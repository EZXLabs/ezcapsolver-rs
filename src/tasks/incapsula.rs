use serde::Serialize;

use super::ExtraFields;

/// Incapsula Reese84 task.
#[derive(Debug, Clone, Serialize, Default, bon::Builder)]
#[builder(on(String, into))]
#[serde(remote = "Self", rename_all = "camelCase")]
pub struct IncapsulaTask {
    /// Full Reese84 sensor script source.
    #[builder(default)]
    pub script: String,
    /// URL of the sensor script.
    #[builder(default)]
    pub script_url: String,
    /// URL of the page executing the sensor script.
    #[builder(default)]
    pub page_url: String,
    /// Accept-Language header used by the browser flow.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accept_language: Option<String>,
    /// Browser user agent.
    #[builder(default)]
    pub ua: String,
    /// Optional proxy used by the worker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<String>,
    /// Proof-of-work data, required by sites that have PoW challenges enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pow: Option<String>,
    /// Additional worker parameters.
    #[serde(skip)]
    #[builder(default)]
    pub extra: ExtraFields,
}
