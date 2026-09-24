use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::Value;

use super::ExtraFields;

/// DataDome challenge step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Default)]
pub enum DataDomeStep {
    /// Fetch the challenge URL.
    #[serde(rename = "1")]
    #[default]
    One,
    /// Produce validation instructions.
    #[serde(rename = "2")]
    Two,
}

/// DataDome tags JavaScript mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DataDomeJsType {
    /// Challenge mode.
    #[default]
    Ch,
    /// Legacy or external mode.
    Le,
}

/// DataDome challenge task.
#[derive(Debug, Clone, Serialize, Default, bon::Builder)]
#[builder(on(String, into))]
#[serde(remote = "Self")]
pub struct DataDomeTask {
    /// Base64-encoded challenge HTML.
    pub html_b64: String,
    /// Challenge workflow step.
    #[builder(default)]
    pub step: DataDomeStep,
    /// Optional base64-encoded image.
    #[builder(default)]
    pub image: String,
    /// Optional page or challenge URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referer: Option<String>,
    /// Optional parent page URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_url: Option<String>,
    /// Optional equipment identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub equipment: Option<String>,
    /// Additional worker parameters.
    #[serde(skip)]
    #[builder(default)]
    pub extra: ExtraFields,
}

/// Lowest packet counter the service accepts.
///
/// `bpc` is validated as one or greater, so zero — what a derived `Default`
/// would produce — is rejected outright.
const MIN_PACKET_COUNTER: u64 = 1;

/// DataDome tags task.
#[derive(Debug, Clone, Serialize, bon::Builder)]
#[builder(on(String, into))]
#[serde(remote = "Self")]
pub struct DataDomeTagsTask {
    /// DataDome JavaScript key.
    pub ddk: String,
    /// DataDome JavaScript mode.
    #[builder(default)]
    pub jstype: DataDomeJsType,
    /// Session identifier. An empty string is valid.
    #[builder(default)]
    pub cid: String,
    /// One-based packet counter.
    ///
    /// `ch` mode fixes it at one; `le` mode starts at two and counts up.
    pub bpc: u64,
    /// Current page URL.
    pub referer: String,
    /// Browser user agent.
    pub ua: String,
    /// External business fields forwarded to the worker.
    #[builder(default)]
    pub fields: BTreeMap<String, Value>,
    /// Additional worker parameters.
    #[serde(skip)]
    #[builder(default)]
    pub extra: ExtraFields,
}

impl Default for DataDomeTagsTask {
    fn default() -> Self {
        Self {
            ddk: String::new(),
            jstype: DataDomeJsType::default(),
            cid: String::new(),
            bpc: MIN_PACKET_COUNTER,
            referer: String::new(),
            ua: String::new(),
            fields: BTreeMap::new(),
            extra: ExtraFields::new(),
        }
    }
}
