use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::BufRead;
use std::io::BufReader;
use std::process::{Command, Stdio};
use std::sync::LazyLock;

use opentelemetry::global::BoxedSpan;
use opentelemetry::trace::{Span, Tracer};
use opentelemetry::KeyValue;
use opentelemetry_semantic_conventions as semconv;
use regex::Regex;

fn normalize_backend_name(be: &str) -> String {
    // capture for vmod-dynamic backends to extract the name as they are in format <vcl_name>(<ip>:<port>)
    static RE_BE_VMOD_DYNAMIC: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?<backend>.*)\((?<server>.*)\)").unwrap());
    if let Some(caps) = RE_BE_VMOD_DYNAMIC.captures(be) {
        return caps["backend"].to_string();
    }

    be.to_string()
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VarnishTimelineItem {
    pub name: String,
    pub timestamp: f64,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VarnishTxBackend {
    pub name: String,
    pub r_addr: String,
    pub r_port: i64,
    pub conn_reused: Option<bool>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VarnishTxClient {
    pub r_addr: String,
    pub r_port: i64,
    pub conn_reused: Option<bool>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VarnishTxLink {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub reason: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VarnishTxReq {
    pub method: String,
    pub proto: String,
    pub hdr_bytes: i64,
    pub body_bytes: i64,
    pub headers: HashMap<String, Vec<String>>,
    pub url: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VarnishTxResp {
    pub proto: String,
    pub hdr_bytes: i64,
    pub body_bytes: i64,
    pub headers: HashMap<String, Vec<String>>,
    pub status: i64,
    pub reason: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VarnishTx {
    pub id: String,
    pub handling: String,
    pub side: String,
    pub vcl: Option<String>,
    pub storage: Option<String>,
    pub error: Option<String>,
    pub logs: Option<Vec<String>>,
    pub links: Option<Vec<VarnishTxLink>>,
    pub backend: Option<VarnishTxBackend>,
    pub client: Option<VarnishTxClient>,
    pub req: VarnishTxReq,
    pub resp: VarnishTxResp,
    pub timeline: Vec<VarnishTimelineItem>,
}

impl VarnishTx {
    /// Returns the timestamp of the first timeline item.
    pub fn get_start_time(&self) -> std::time::SystemTime {
        if let Some(t) = self.timeline.first() {
            std::time::UNIX_EPOCH + std::time::Duration::from_secs_f64(t.timestamp)
        } else {
            std::time::SystemTime::now()
        }
    }

    /// Returns the timestamp of the last timeline item.
    pub fn get_end_time(&self) -> std::time::SystemTime {
        if let Some(t) = self.timeline.last() {
            std::time::UNIX_EPOCH + std::time::Duration::from_secs_f64(t.timestamp)
        } else {
            std::time::SystemTime::now()
        }
    }

    /// Updates a span with attributes from this transaction
    pub fn update_span(&self, mut span: BoxedSpan) -> BoxedSpan {
        span.update_name(format!("Varnish {} {}", self.side, self.handling));

        span.set_attribute(KeyValue::new(
            semconv::trace::URL_FULL,
            self.req.url.clone(),
        ));
        span.set_attribute(KeyValue::new(semconv::trace::NETWORK_PROTOCOL_NAME, "http"));
        span.set_attribute(KeyValue::new(
            semconv::trace::HTTP_REQUEST_METHOD,
            self.req.method.clone(),
        ));
        span.set_attribute(KeyValue::new(
            semconv::trace::HTTP_REQUEST_SIZE,
            self.req.body_bytes + self.req.hdr_bytes,
        ));
        span.set_attribute(KeyValue::new(
            semconv::trace::HTTP_RESPONSE_BODY_SIZE,
            self.resp.body_bytes,
        ));
        span.set_attribute(KeyValue::new(
            semconv::trace::HTTP_RESPONSE_SIZE,
            self.resp.body_bytes + self.resp.hdr_bytes,
        ));
        span.set_attribute(KeyValue::new(
            semconv::trace::HTTP_RESPONSE_STATUS_CODE,
            self.resp.status,
        ));

        span.set_attribute(KeyValue::new(
            varnishotel_semconv::VARNISH_VCL,
            self.vcl.clone().unwrap_or_default(),
        ));
        span.set_attribute(KeyValue::new(
            varnishotel_semconv::VARNISH_SIDE,
            self.side.clone(),
        ));
        span.set_attribute(KeyValue::new(
            varnishotel_semconv::VARNISH_VXID,
            self.id.clone(),
        ));

        if let Some(b) = &self.backend {
            span.set_attribute(KeyValue::new(
                semconv::trace::NETWORK_PEER_ADDRESS,
                b.r_addr.clone(),
            ));
            span.set_attribute(KeyValue::new(semconv::trace::NETWORK_PEER_PORT, b.r_port));

            span.set_attribute(KeyValue::new(
                varnishotel_semconv::VARNISH_BACKEND_NAME,
                b.name.clone(),
            ));
            span.set_attribute(KeyValue::new(
                varnishotel_semconv::VARNISH_BACKEND_CONN_REUSED,
                b.conn_reused.unwrap_or_default(),
            ));

            span.update_name(format!(
                "Varnish to {} fetch",
                normalize_backend_name(&b.name)
            ));
        }

        if let Some(c) = &self.client {
            span.set_attribute(KeyValue::new(
                semconv::trace::NETWORK_PEER_ADDRESS,
                c.r_addr.clone(),
            ));
            span.set_attribute(KeyValue::new(semconv::trace::NETWORK_PEER_PORT, c.r_port));
        }

        for event in self.timeline.clone() {
            let event_name = event.name;
            let event_ts =
                std::time::UNIX_EPOCH + std::time::Duration::from_secs_f64(event.timestamp);

            span.add_event_with_timestamp(event_name, event_ts, vec![]);
        }

        if let Some(err) = &self.error {
            span.set_status(opentelemetry::trace::Status::error(err.to_string()));
        }

        span
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct VarnishRequest(Vec<VarnishTx>);

impl VarnishRequest {
    pub fn get_req_top(&self) -> Option<&VarnishTx> {
        self.0.first()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// A wrapper around [`varnishlog-json`](https://github.com/varnish/varnishlog-json).
#[derive(Debug, Default)]
pub struct VarnishlogReceiver {}

impl VarnishlogReceiver {
    pub fn new() -> Self {
        Self {}
    }

    /// Stream output of varnishlog-json and convert the output to traces.
    /// Requires [`varnishlogjson`] to be available on the `$PATH`.
    pub async fn execute(&self, scope: opentelemetry::InstrumentationScope) -> anyhow::Result<()> {
        let cmd_args = vec!["-b", "-c", "-E", "-g", "request"];
        let mut cmd_binding = Command::new("varnishlogjson");
        let mut cmd = cmd_binding
            .args(cmd_args)
            .stdout(Stdio::piped())
            .spawn()
            .inspect_err(|e| tracing::error!("failed to run varnishlogjson: {e}"))?;

        tracing::debug!("running varnishlogjson as [{:?}]", cmd);

        let tracer = opentelemetry::global::tracer_with_scope(scope);

        {
            let stdout = cmd.stdout.as_mut().unwrap();
            let stdout_reader = BufReader::new(stdout);
            let stdout_lines = stdout_reader.lines();

            for line in stdout_lines {
                match line {
                    Ok(s) => {
                        let req =
                            serde_json::from_str::<VarnishRequest>(&s.clone()).unwrap_or_default();

                        if req.is_empty() {
                            tracing::warn!("found empty request. skipping");
                            continue;
                        }

                        let req_top = req.get_req_top().unwrap();
                        let req_top_start = req_top.get_start_time();
                        let req_top_end = req_top.get_end_time();

                        let mut sp_top = tracer
                            .span_builder("Varnish request processing")
                            .with_kind(opentelemetry::trace::SpanKind::Server)
                            .with_start_time(req_top_start)
                            .with_end_time(req_top_end)
                            .start(&tracer);

                        sp_top = req_top.update_span(sp_top);
                        sp_top.update_name("Varnish request processing");

                        let sp_top_active = opentelemetry::trace::mark_span_as_active(sp_top);

                        for req_sub in &req.0[1..] {
                            let req_sub_start = req_sub.get_start_time();
                            let req_sub_end = req_sub.get_end_time();

                            {
                                let mut sp_sub = tracer
                                    .span_builder(format!(
                                        "Varnish {} {}",
                                        req_sub.side, req_sub.handling,
                                    ))
                                    .with_kind(opentelemetry::trace::SpanKind::Internal)
                                    .with_start_time(req_sub_start)
                                    .with_end_time(req_sub_end)
                                    .start(&tracer);

                                sp_sub = req_sub.update_span(sp_sub);

                                let _sp_sub_active =
                                    opentelemetry::trace::mark_span_as_active(sp_sub);
                            }
                        }

                        drop(sp_top_active);
                    }
                    Err(err) => tracing::error!("failed to read line from stdout: {err}"),
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_backend_name() {
        assert_eq!(normalize_backend_name("default"), "default");
        assert_eq!(normalize_backend_name("dyn_dir(127.0.0.1:80)"), "dyn_dir");
    }
}
