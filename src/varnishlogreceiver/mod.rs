use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::BufRead;
use std::io::BufReader;
use std::process::{Command, Stdio};

use opentelemetry::trace::Tracer;

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

                        if req.len() == 0 {
                            tracing::warn!("found empty request. skipping");
                            continue;
                        }

                        let req_top = req.get_req_top().unwrap();
                        let req_top_start = req_top.get_start_time();
                        let req_top_end = req_top.get_end_time();

                        let sp_top = tracer
                            .span_builder("Varnish request processing")
                            .with_kind(opentelemetry::trace::SpanKind::Server)
                            .with_start_time(req_top_start)
                            .with_end_time(req_top_end)
                            .start(&tracer);

                        let sp_top_active = opentelemetry::trace::mark_span_as_active(sp_top);

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
}
