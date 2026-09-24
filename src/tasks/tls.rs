use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::Value;

use super::ExtraFields;

/// HTTP method accepted by TLS forwarding tasks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum TlsHttpMethod {
    /// GET request.
    #[default]
    Get,
    /// POST request.
    Post,
    /// PUT request.
    Put,
    /// DELETE request.
    Delete,
    /// PATCH request.
    Patch,
}

/// TLS forwarding task.
#[derive(Debug, Clone, Serialize, Default, bon::Builder)]
#[builder(on(String, into))]
#[serde(remote = "Self")]
pub struct TlsForwardTask {
    /// Worker TLS fingerprint identifier.
    pub tls_type: String,
    /// Proxy used for the upstream request.
    pub proxy: String,
    /// Upstream HTTP method.
    #[builder(default)]
    pub method: TlsHttpMethod,
    /// Upstream URL.
    pub url: String,
    /// Optional upstream request headers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<BTreeMap<String, Value>>,
    /// Optional serialized header order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers_order: Option<String>,
    /// Optional upstream request cookies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cookies: Option<BTreeMap<String, Value>>,
    /// Optional upstream request body.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<Value>,
    /// Setting it to true means the body is base64-encoded.
    #[serde(default)]
    #[builder(default = false)]
    pub body_raw: bool,
    /// Additional worker parameters.
    #[serde(skip)]
    #[builder(default)]
    pub extra: ExtraFields,
}
