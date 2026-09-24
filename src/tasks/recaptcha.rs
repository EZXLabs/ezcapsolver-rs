use serde::Serialize;

use super::ExtraFields;

/// Invisible mode the service assumes for V3 when the parameter is omitted.
///
/// V2 defaults to `false`, which is the Rust default for `bool`; V3 does not,
/// so this value has to be stated in both the builder and `Default`.
const V3_DEFAULT_IS_INVISIBLE: bool = true;

/// Grid size the service assumes when the parameter is omitted.
const DEFAULT_CLASSIFICATION_SIZE: i32 = 4;

/// Request parameters shared by ReCaptcha V2 task variants.
///
/// Build with string literals or owned strings; optional setters wrap values
/// in `Some`, while `maybe_*` setters accept an `Option` directly.
///
/// ```
/// use ezcapsolver::RecaptchaV2Task;
///
/// let task = RecaptchaV2Task::builder()
///     .website_url("https://example.com")
///     .website_key("site-key")
///     .s("challenge-data")
///     .build();
/// assert_eq!(task.s.as_deref(), Some("challenge-data"));
/// assert!(!task.is_invisible);
/// ```
///
/// Required fields must be set before calling `build()`.
///
/// ```compile_fail
/// use ezcapsolver::RecaptchaV2Task;
///
/// let task = RecaptchaV2Task::builder()
///     .website_url("https://example.com")
///     .build();
/// ```
#[derive(Debug, Clone, Serialize, Default, bon::Builder)]
#[builder(on(String, into))]
#[serde(remote = "Self", rename_all = "camelCase")]
pub struct RecaptchaV2Task {
    /// URL of the page containing the challenge.
    #[serde(rename = "websiteURL")]
    pub website_url: String,
    /// ReCaptcha site key.
    pub website_key: String,
    /// Whether the challenge uses invisible mode.
    #[serde(rename = "isInvisible")]
    #[builder(default)]
    pub is_invisible: bool,
    /// Optional security anchor parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sa: Option<String>,
    /// Optional challenge-bound `s` parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub s: Option<String>,
    /// Optional page title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website_title: Option<String>,
    /// Optional proxy forwarded to the worker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<String>,
    /// Additional worker parameters.
    #[serde(skip)]
    #[builder(default)]
    pub extra: ExtraFields,
}

/// Request parameters shared by ReCaptcha V3 task variants.
#[derive(Debug, Clone, Serialize, bon::Builder)]
#[builder(on(String, into))]
#[serde(remote = "Self", rename_all = "camelCase")]
pub struct RecaptchaV3Task {
    /// URL of the page containing the challenge.
    #[serde(rename = "websiteURL")]
    pub website_url: String,
    /// ReCaptcha site key.
    pub website_key: String,
    /// Whether the challenge uses invisible mode.
    #[serde(rename = "isInvisible")]
    #[builder(default = V3_DEFAULT_IS_INVISIBLE)]
    pub is_invisible: bool,
    /// Optional action configured by the protected page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_action: Option<String>,
    /// Optional page title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website_title: Option<String>,
    /// Optional website-specific check field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_field: Option<String>,
    /// Optional proxy forwarded to the worker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<String>,
    /// Additional worker parameters.
    #[serde(skip)]
    #[builder(default)]
    pub extra: ExtraFields,
}

impl Default for RecaptchaV3Task {
    fn default() -> Self {
        Self {
            website_url: String::new(),
            website_key: String::new(),
            is_invisible: V3_DEFAULT_IS_INVISIBLE,
            page_action: None,
            website_title: None,
            check_field: None,
            proxy: None,
            extra: ExtraFields::new(),
        }
    }
}

/// ReCaptcha V2 image classification request.
#[derive(Debug, Clone, Serialize, bon::Builder)]
#[builder(on(String, into))]
#[serde(remote = "Self", rename_all = "camelCase")]
pub struct RecaptchaV2ClassificationTask {
    /// Base64-encoded challenge image.
    pub image: String,
    /// Object identifier or classification question.
    pub question: String,
    /// Grid size documented by the service.
    #[builder(default = DEFAULT_CLASSIFICATION_SIZE)]
    pub size: i32,
    /// Additional worker parameters.
    #[serde(skip)]
    #[builder(default)]
    pub extra: ExtraFields,
}

impl Default for RecaptchaV2ClassificationTask {
    fn default() -> Self {
        Self {
            image: String::new(),
            question: String::new(),
            size: DEFAULT_CLASSIFICATION_SIZE,
            extra: ExtraFields::new(),
        }
    }
}
