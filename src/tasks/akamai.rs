use serde::Serialize;

use super::ExtraFields;

/// Akamai Web task.
#[derive(Debug, Clone, Serialize, Default, bon::Builder)]
#[builder(on(String, into))]
#[serde(remote = "Self", rename_all = "camelCase")]
pub struct AkamaiWebTask {
    /// Page URL associated with the Akamai script.
    pub page_url: String,
    /// URL of the Akamai v3 script.
    ///
    /// Most sites change this URL on every request, so it has to be read from
    /// the page rather than hard-coded. It is the URL itself, not the script
    /// the URL serves.
    pub v3_url: String,
    /// Browser user agent.
    pub ua: String,
    /// Browser language.
    pub lang: String,
    /// Current interaction round.
    pub index: i32,
    /// Current `_abck` cookie value.
    #[serde(rename = "abck")]
    #[builder(default)]
    pub abck: String,
    /// Current `bm_sz` cookie value.
    #[serde(rename = "bmsz")]
    #[builder(default)]
    pub bmsz: String,
    /// Base64-encoded Akamai script.
    #[serde(rename = "script_base64")]
    #[builder(default)]
    pub script_base64: String,
    /// Encoded state returned by the previous round.
    #[builder(default)]
    pub encode_data: String,
    /// Additional worker parameters.
    #[serde(skip)]
    #[builder(default)]
    pub extra: ExtraFields,
}

/// Akamai SBSD task.
#[derive(Debug, Clone, Serialize, Default, bon::Builder)]
#[builder(on(String, into))]
#[serde(remote = "Self", rename_all = "camelCase")]
pub struct AkamaiSbsdTask {
    /// Page URL associated with the challenge.
    pub page_url: String,
    /// URL of the SBSD script.
    pub sbsd_url: String,
    /// Existing `bm_so` or equivalent cookie value.
    pub bm_so: String,
    /// Browser user agent.
    pub ua: String,
    /// Browser language.
    pub lang: String,
    /// Base64-encoded SBSD script.
    #[serde(rename = "script_base64")]
    pub script_base64: String,
    /// Additional worker parameters.
    #[serde(skip)]
    #[builder(default)]
    pub extra: ExtraFields,
}
