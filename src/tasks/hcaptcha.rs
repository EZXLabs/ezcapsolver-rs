use serde::Serialize;

use super::ExtraFields;

/// HCaptcha token task.
#[derive(Debug, Clone, Serialize, Default, bon::Builder)]
#[builder(on(String, into))]
#[serde(remote = "Self", rename_all = "camelCase")]
pub struct HcaptchaTask {
    /// URL of the page containing the challenge.
    #[serde(rename = "websiteURL")]
    pub website_url: String,
    /// HCaptcha site key.
    pub website_key: String,
    /// Browser language. Only `en-US` is supported at the moment.
    pub lang: String,
    /// Optional proxy forwarded to the worker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<String>,
    /// Whether the challenge runs without a visible checkbox.
    ///
    /// True when the site shows no hCaptcha checkbox, false when it does.
    pub invisible: bool,
    /// Optional `rqdata` value, required by the sites that publish one.
    ///
    /// The wire name is all lowercase here, unlike the Cloudflare types, whose
    /// equivalent field is `rqData`. The two are not interchangeable.
    #[serde(rename = "rqdata", skip_serializing_if = "Option::is_none")]
    pub rq_data: Option<String>,
    /// Additional worker parameters.
    #[serde(skip)]
    #[builder(default)]
    pub extra: ExtraFields,
}

/// HCaptcha image classification task.
#[derive(Debug, Clone, Serialize, Default, bon::Builder)]
#[builder(on(String, into))]
#[serde(remote = "Self", rename_all = "camelCase")]
pub struct HcaptchaClassificationTask {
    /// Optional single image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    /// Optional image collection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,
    /// Optional anchor collection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchors: Option<Vec<String>>,
    /// Optional classification question.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub question: Option<String>,
    /// Optional classification module.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
    /// Additional worker parameters.
    #[serde(skip)]
    #[builder(default)]
    pub extra: ExtraFields,
}
