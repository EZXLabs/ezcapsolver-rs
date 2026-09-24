use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::Value;

use super::ExtraFields;

/// Cloudflare five-second challenge task.
#[derive(Debug, Clone, Serialize, Default, bon::Builder)]
#[builder(on(String, into))]
#[serde(remote = "Self", rename_all = "camelCase")]
pub struct Cloudflare5sTask {
    /// URL protected by the challenge.
    #[serde(rename = "websiteURL")]
    pub website_url: String,
    /// Proxy used by the challenge worker. Required for this type, unlike most
    /// others.
    ///
    /// Format is `protocol://username:password@host:port` with protocol one of
    /// `http`, `https` or `socks5`. Both credentials are required — the service
    /// rejects an unauthenticated proxy — and the host may not be a private
    /// address.
    pub proxy: String,
    /// Optional challenge request data.
    #[serde(rename = "rqData", skip_serializing_if = "Option::is_none")]
    pub rq_data: Option<BTreeMap<String, Value>>,
    /// Additional worker parameters.
    #[serde(skip)]
    #[builder(default)]
    pub extra: ExtraFields,
}

/// Cloudflare Turnstile task.
#[derive(Debug, Clone, Serialize, Default, bon::Builder)]
#[builder(on(String, into))]
#[serde(remote = "Self", rename_all = "camelCase")]
pub struct CloudflareTurnstileTask {
    /// URL containing the Turnstile widget.
    #[serde(rename = "websiteURL")]
    pub website_url: String,
    /// Turnstile site key.
    pub website_key: String,
    /// Optional worker proxy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<String>,
    /// Optional Turnstile metadata.
    #[serde(rename = "rqData", skip_serializing_if = "Option::is_none")]
    pub rq_data: Option<BTreeMap<String, Value>>,
    /// Additional worker parameters.
    #[serde(skip)]
    #[builder(default)]
    pub extra: ExtraFields,
}
