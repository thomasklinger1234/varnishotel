use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
}
