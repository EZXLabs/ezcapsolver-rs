//! The DataDome challenge workflow.
//!
//! [`workflow`] drives the flow through the network. The other submodules hold
//! every step that only reads or builds text — page classification, challenge
//! script extraction, request headers, and the submission verdict — so each one
//! can be tested without a server.

use std::fmt;

mod browser;
mod page;
mod verdict;
mod workflow;

/// `Accept-Language` sent when the caller does not choose one.
const DEFAULT_ACCEPT_LANGUAGE: &str = "en-US,en;q=0.9";

/// A page behind DataDome, and the browser identity to clear its challenge with.
///
/// Every browser-side request of one run goes out through [`Self::proxy`] with
/// [`Self::user_agent`]. Keep using both for the requests that carry the cookie
/// the run returns.
#[derive(Clone, bon::Builder)]
#[builder(on(String, into))]
pub struct DataDomeChallenge {
    /// Page on the protected site.
    ///
    /// It is fetched first, and the flow only continues when DataDome
    /// intercepts it.
    pub target_url: String,
    /// Browser `User-Agent` sent with every browser-side request.
    ///
    /// Its `Chrome/<major>` version also selects the TLS fingerprint, so it
    /// must name a Chrome release with a fingerprint preset: 131, or 144 to 158.
    pub user_agent: String,
    /// Proxy for every browser-side request, such as `http://user:pass@host:port`.
    ///
    /// Unset means a direct connection. Calls to the EzCaptchaSolver API do not
    /// use it; they follow [`ClientConfig::proxy`](crate::ClientConfig::proxy).
    pub proxy: Option<String>,
    /// `Accept-Language` sent with every browser-side request.
    #[builder(default = DEFAULT_ACCEPT_LANGUAGE.to_owned())]
    pub accept_language: String,
}

/// Keeps proxy credentials out of logs, the way `ClientConfig` does.
impl fmt::Debug for DataDomeChallenge {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DataDomeChallenge")
            .field("target_url", &self.target_url)
            .field("user_agent", &self.user_agent)
            .field("proxy", &self.proxy.as_ref().map(|_| "[REDACTED]"))
            .field("accept_language", &self.accept_language)
            .finish()
    }
}

/// How a DataDome challenge run ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataDomeOutcome {
    /// DataDome let the target page through, so no cookie was needed.
    NotTriggered,
    /// The challenge was cleared, and DataDome issued this cookie.
    Passed(DataDomeCookie),
}

/// The cookie DataDome issues once its challenge is cleared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataDomeCookie {
    /// Cookie name, normally `datadome`.
    pub name: String,
    /// Cookie value.
    pub value: String,
    /// Domain the cookie applies to.
    ///
    /// Taken from the `Domain` attribute, or built from the last two labels of
    /// the target host, such as `.example.com`, when DataDome omits it.
    pub domain: String,
    /// The cookie exactly as DataDome returned it, attributes included.
    pub raw: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_builder_fills_the_default_accept_language() {
        let challenge = DataDomeChallenge::builder()
            .target_url("https://example.com")
            .user_agent("Chrome/153.0.0.0")
            .build();
        assert_eq!(challenge.accept_language, DEFAULT_ACCEPT_LANGUAGE);
        assert_eq!(challenge.proxy, None);
    }

    #[test]
    fn debug_output_redacts_the_proxy() {
        let challenge = DataDomeChallenge::builder()
            .target_url("https://example.com")
            .user_agent("Chrome/153.0.0.0")
            .proxy("http://user:secret@proxy.example.com:8080")
            .build();
        let rendered = format!("{challenge:?}");
        assert!(!rendered.contains("secret"), "{rendered}");
        assert!(rendered.contains("[REDACTED]"), "{rendered}");
    }
}
