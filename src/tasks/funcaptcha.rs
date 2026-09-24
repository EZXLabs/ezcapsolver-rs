use serde::Serialize;

use super::ExtraFields;

/// FunCaptcha token task.
#[derive(Debug, Clone, Serialize, Default, bon::Builder)]
#[builder(on(String, into))]
#[serde(remote = "Self", rename_all = "camelCase")]
pub struct FunCaptchaTask {
    /// URL of the page containing the challenge.
    #[serde(rename = "websiteURL")]
    pub website_url: String,
    /// FunCaptcha public key.
    pub website_key: String,
    /// Optional Arkose Labs blob data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    /// Optional Arkose Labs API subdomain.
    #[serde(
        rename = "funcaptchaApiJSSubdomain",
        skip_serializing_if = "Option::is_none"
    )]
    pub api_js_subdomain: Option<String>,
    /// Optional worker proxy.
    ///
    /// FunCaptcha is the only task type using the `FUN` proxy format —
    /// `protocol://host:port:username:password`, with the credentials appended
    /// rather than placed before the host. Every other type takes
    /// `protocol://username:password@host:port`.
    ///
    /// Either way the service requires both a username and a password: an
    /// unauthenticated proxy is rejected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<String>,
    /// Whether the supplied proxy is in mainland China.
    #[builder(default)]
    pub cn: bool,
    /// Additional worker parameters.
    #[serde(skip)]
    #[builder(default)]
    pub extra: ExtraFields,
}

/// FunCaptcha image classification task.
#[derive(Debug, Clone, Serialize, Default, bon::Builder)]
#[builder(on(String, into))]
#[serde(remote = "Self", rename_all = "camelCase")]
pub struct FunCaptchaClassificationTask {
    /// Base64-encoded challenge image.
    pub image: String,
    /// Classification question.
    pub question: String,
    /// Additional worker parameters.
    #[serde(skip)]
    #[builder(default)]
    pub extra: ExtraFields,
}
