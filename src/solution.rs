use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::DeserializeOwned};
use serde_json::Value;

use crate::{Error, Result};

/// Raw solution field from a task response.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Solution {
    /// The response did not include a solution field.
    #[default]
    Missing,
    /// The response included a solution value, including JSON null.
    Value(Value),
}

impl Solution {
    /// Returns true when the response did not include a solution field.
    #[must_use]
    pub const fn is_missing(&self) -> bool {
        matches!(self, Self::Missing)
    }

    /// Returns the raw JSON value when the field was present.
    #[must_use]
    pub const fn as_value(&self) -> Option<&Value> {
        match self {
            Self::Missing => None,
            Self::Value(value) => Some(value),
        }
    }

    /// Consumes the wrapper and returns the raw JSON value when present.
    #[must_use]
    pub fn into_value(self) -> Option<Value> {
        match self {
            Self::Missing => None,
            Self::Value(value) => Some(value),
        }
    }

    /// Deserializes the raw JSON value into a caller-selected type.
    ///
    /// The raw value is borrowed, so only the fields `T` actually reads are
    /// copied and the stored solution remains available afterwards.
    pub fn deserialize<T>(&self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let value = self.as_value().ok_or_else(|| {
            Error::UnexpectedResponse("ready task result does not contain a solution".to_owned())
        })?;
        // Clone only on the failure path: when decoding fails the raw value is the
        // only thing left to diagnose it with, so it must not go down with the
        // TaskResult.
        T::deserialize(value).map_err(|source| Error::SolutionDecode {
            source,
            raw: Box::new(value.clone()),
        })
    }
}

impl Serialize for Solution {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Missing => serializer.serialize_none(),
            Self::Value(value) => value.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Solution {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Value::deserialize(deserializer).map(Self::Value)
    }
}

/// ReCaptcha token solution observed from V2 and V3 workers.
///
/// Only the token is guaranteed. The two header companions are emitted by some
/// worker builds and not others, so they default to an empty string rather than
/// failing the decode — a task that solved and was billed must not be thrown
/// away over a field the caller may not even need.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(remote = "Self")]
pub struct RecaptchaSolution {
    /// Token submitted to the protected website.
    #[serde(rename = "gRecaptchaResponse")]
    pub token: String,
    /// Matching `Sec-CH-UA` request header, empty when the worker omits it.
    #[serde(default)]
    pub sec_ch_ua: String,
    /// Matching user agent, empty when the worker omits it.
    #[serde(default)]
    pub user_agent: String,
    /// Additional worker fields.
    #[serde(flatten, skip_serializing)]
    pub extra: BTreeMap<String, Value>,
}

/// FunCaptcha (Arkose Labs) token solution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(remote = "Self")]
pub struct FunCaptchaSolution {
    /// Token submitted to the protected website.
    pub token: String,
    /// Additional worker fields.
    #[serde(flatten, skip_serializing)]
    pub extra: BTreeMap<String, Value>,
}

/// FunCaptcha classification result, shape not yet confirmed.
///
/// No reliable sample exists yet, so this declares no fields of its own —
/// everything the worker returns lands in [`Self::extra`]. Add fields here as
/// samples confirm them.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(remote = "Self")]
pub struct FunCaptchaClassificationSolution {
    /// Every field the worker returned.
    #[serde(flatten, skip_serializing)]
    pub extra: BTreeMap<String, Value>,
}

/// HCaptcha classification result, shape not yet confirmed.
///
/// No reliable sample exists yet, so this declares no fields of its own —
/// everything the worker returns lands in [`Self::extra`]. Add fields here as
/// samples confirm them.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(remote = "Self")]
pub struct HcaptchaClassificationSolution {
    /// Every field the worker returned.
    #[serde(flatten, skip_serializing)]
    pub extra: BTreeMap<String, Value>,
}

/// Cloudflare Turnstile solution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(remote = "Self")]
pub struct CloudflareTurnstileSolution {
    /// Token submitted to the protected website.
    pub token: String,
    /// Request headers to replay together with the token.
    #[serde(default)]
    pub header: BTreeMap<String, String>,
    /// Additional worker fields.
    #[serde(flatten, skip_serializing)]
    pub extra: BTreeMap<String, Value>,
}

/// Cloudflare five-second challenge solution.
///
/// Every field describes one part of the browser state the worker ended up
/// with; replaying the headers and cookies against the protected site is what
/// actually clears the challenge.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(remote = "Self", rename_all = "camelCase")]
pub struct Cloudflare5sSolution {
    /// Request headers to replay against the protected site.
    #[serde(default)]
    pub header: BTreeMap<String, String>,
    /// Clearance cookies returned by the worker.
    #[serde(default)]
    pub cookies: BTreeMap<String, String>,
    /// Browser fingerprint used by the worker, such as `chrome149`.
    #[serde(default)]
    pub tls_version: String,
    /// Challenge page body, empty when the worker captured none.
    #[serde(default)]
    pub body: String,
    /// Turnstile token embedded in the challenge, empty when the flow did not
    /// produce one.
    #[serde(default)]
    pub s_token: String,
    /// Additional worker fields.
    #[serde(flatten, skip_serializing)]
    pub extra: BTreeMap<String, Value>,
}

/// HCaptcha solution observed from current workers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(remote = "Self")]
pub struct HcaptchaSolution {
    /// Generated HCaptcha pass identifier.
    #[serde(rename = "generated_pass_UUID")]
    pub generated_pass_uuid: String,
    /// User agent associated with the result.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ua: Option<String>,
    /// lang
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    /// Additional worker fields.
    #[serde(flatten, skip_serializing)]
    pub extra: BTreeMap<String, Value>,
}

/// PerimeterX solution.
///
/// The worker returns the clearance cookies as top-level fields, not nested
/// under a `cookies` object. Field names carry a leading underscore on the
/// wire; the Rust fields drop it because a leading underscore means
/// "intentionally unused" here.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(remote = "Self")]
pub struct PerimeterXSolution {
    /// `_px3` clearance cookie, the value that actually passes the check.
    #[serde(rename = "_px3")]
    pub px3: String,
    /// `_pxvid` visitor identifier.
    #[serde(rename = "_pxvid", default)]
    pub pxvid: String,
    /// `_pxde` data-enrichment cookie.
    #[serde(rename = "_pxde", default)]
    pub pxde: String,
    /// Additional worker fields.
    #[serde(flatten, skip_serializing)]
    pub extra: BTreeMap<String, Value>,
}

/// Akamai Web solution used in multi-round flows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(remote = "Self")]
pub struct AkamaiWebSolution {
    /// Sensor payload for the current round.
    pub payload: String,
    /// Encoded state passed to the next round.
    ///
    /// Sent back as `encodeData` in the following round. A round that ends the
    /// flow carries none, so its absence is not a decoding failure.
    #[serde(default)]
    pub encodedata: String,
    /// Additional worker fields.
    #[serde(flatten, skip_serializing)]
    pub extra: BTreeMap<String, Value>,
}

/// Akamai SBSD solution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(remote = "Self")]
pub struct AkamaiSbsdSolution {
    /// Base64-encoded payload returned by the worker.
    pub payload: String,
    /// `bm_lso_time` value produced alongside the payload.
    ///
    /// Optional: not every response carries one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bm_lso_time: Option<String>,
    /// Additional worker fields.
    #[serde(flatten, skip_serializing)]
    pub extra: BTreeMap<String, Value>,
}

/// TLS forwarding response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(remote = "Self")]
pub struct TlsForwardSolution {
    /// Worker response status.
    #[serde(default)]
    pub status: u16,
    /// Upstream HTTP status code.
    #[serde(default)]
    pub code: u16,
    /// Upstream response headers.
    #[serde(default)]
    pub headers: BTreeMap<String, Value>,
    /// Upstream response cookies.
    #[serde(default)]
    pub cookies: HashMap<String, Value>,
    /// Upstream response body.
    #[serde(default)]
    pub body: String,
    /// Additional worker fields.
    #[serde(flatten, skip_serializing)]
    pub extra: BTreeMap<String, Value>,
}

/// ReCaptcha V2 image classification result.
///
/// Unknown type values are preserved for the caller to interpret. Missing
/// fields use their defaults: an empty type, `false`, and an empty object list.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(remote = "Self", default, rename_all = "camelCase")]
pub struct ReClassificationSolution {
    /// Worker result type, usually `multi` or `single`.
    pub r#type: String,
    /// Whether a single image contains the requested object.
    pub has_object: bool,
    /// Zero-based cell indexes to select.
    pub objects: Vec<usize>,
    /// Additional worker fields.
    #[serde(flatten, skip_serializing)]
    pub extra: BTreeMap<String, Value>,
}

impl ReClassificationSolution {
    /// Returns true when the worker reported a multi-cell grid result.
    #[must_use]
    pub fn is_multi(&self) -> bool {
        self.r#type == "multi"
    }

    /// Returns true when the worker reported a single-image result.
    #[must_use]
    pub fn is_single(&self) -> bool {
        self.r#type == "single"
    }
}

/// DataDome solution, returned by both challenge steps.
///
/// Step 1 carries the challenge URL in `url`; step 2 carries the validation
/// instructions. Earlier workers returned step 1 as a bare string — it is an
/// object now, so both steps decode into this one type.
///
/// Shared by `DataDomeTaskProxyless` and `DataDomeTagsTaskProxyless`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(remote = "Self")]
pub struct DataDomeSolution {
    /// Challenge kind, such as `slider` or `interstitial`.
    #[serde(default)]
    pub kind: Option<String>,
    /// Challenge URL on step 1, validation endpoint on step 2.
    #[serde(default)]
    pub url: Option<String>,
    /// Optional validation request body.
    #[serde(default)]
    pub body: Option<String>,
    /// Additional worker fields.
    #[serde(flatten, skip_serializing)]
    pub extra: BTreeMap<String, Value>,
}

/// Incapsula worker response envelope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(remote = "Self")]
pub struct IncapsulaSolution {
    /// Inner worker status code.
    #[serde(default)]
    pub status: Option<u16>,
    /// JSON string posted to the Incapsula sensor endpoint.
    ///
    /// This is stringified JSON and is submitted verbatim; decoding it here
    /// would change what the caller has to send.
    pub data: String,
    /// Additional worker fields.
    #[serde(flatten, skip_serializing)]
    pub extra: BTreeMap<String, Value>,
}

/// Gives a result model both halves of its serde handling.
///
/// Decoding keeps the derived behaviour: `extra` is flattened, so a field this
/// release does not declare is collected instead of dropped. Encoding is the
/// half that needs rewriting — a flattened map is written after the declared
/// fields and would emit a colliding key a second time, which reads back as
/// whichever copy the parser keeps. `skip_serializing` takes `extra` out of the
/// derived encoder so the merge can put it back under the declared fields.
///
/// `#[serde(remote = "Self")]` keeps both derived implementations reachable as
/// inherent functions, so neither call below recurses.
macro_rules! impl_solution_serde {
    ($($ty:ident),+ $(,)?) => {$(
        impl Serialize for $ty {
            fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                crate::wire::merge_extra(|s| <$ty>::serialize(self, s), &self.extra, serializer)
            }
        }

        impl<'de> Deserialize<'de> for $ty {
            fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                <$ty>::deserialize(deserializer)
            }
        }
    )+};
}

impl_solution_serde!(
    AkamaiSbsdSolution,
    AkamaiWebSolution,
    Cloudflare5sSolution,
    CloudflareTurnstileSolution,
    DataDomeSolution,
    FunCaptchaClassificationSolution,
    FunCaptchaSolution,
    HcaptchaClassificationSolution,
    HcaptchaSolution,
    IncapsulaSolution,
    PerimeterXSolution,
    ReClassificationSolution,
    RecaptchaSolution,
    TlsForwardSolution,
);
