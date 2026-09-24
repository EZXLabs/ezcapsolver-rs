use std::{fmt, time::Duration};

#[cfg(any(feature = "async", feature = "blocking"))]
use crate::{Error, Result};

/// Default base URL for asynchronous task operations and balance queries.
pub const DEFAULT_ASYNC_BASE_URL: &str = "https://api.ez-captcha.com";

/// Default base URL for synchronous task operations.
pub const DEFAULT_SYNC_BASE_URL: &str = "https://sync.ez-captcha.com";

/// Default environment variable used to load the client key.
pub const DEFAULT_CLIENT_KEY_ENV: &str = "EZCAPTCHA_API_KEY";

/// Default timeout for asynchronous endpoints and balance queries.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Default timeout for the synchronous task endpoint.
///
/// The service grants its slowest synchronous task types a 180-second worker
/// deadline. Matching that exactly would leave no margin: a worker that uses
/// its full budget would be cut off client-side, and a synchronous task that
/// times out cannot be recovered. This adds one minute of headroom.
pub const DEFAULT_SYNC_TIMEOUT: Duration = Duration::from_secs(240);

/// Default delay between two result queries.
pub const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(3);

/// Default number of result queries before polling gives up.
pub const DEFAULT_MAX_POLL_ATTEMPTS: usize = 50;

/// Polling settings used while waiting for an asynchronous task.
///
/// Construct this with [`Self::new`] or [`Self::default`]; it is
/// `#[non_exhaustive]` so that new settings can be added without a breaking
/// change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct PollingConfig {
    /// Delay applied before every result query, including the first one.
    ///
    /// A freshly created task is still queued for a worker, so querying
    /// immediately after creation almost always reports `processing`. Waiting
    /// one interval first avoids that wasted request.
    pub interval: Duration,
    /// Maximum number of result queries before returning a timeout error.
    pub max_attempts: usize,
}

impl Default for PollingConfig {
    fn default() -> Self {
        Self {
            interval: DEFAULT_POLL_INTERVAL,
            max_attempts: DEFAULT_MAX_POLL_ATTEMPTS,
        }
    }
}

impl PollingConfig {
    /// Creates polling settings with the supplied interval and attempt limit.
    #[must_use]
    pub const fn new(interval: Duration, max_attempts: usize) -> Self {
        Self {
            interval,
            max_attempts,
        }
    }

    #[cfg(any(feature = "async", feature = "blocking"))]
    pub(crate) fn validate(self) -> Result<()> {
        if self.interval.is_zero() {
            return Err(Error::config("polling interval must be greater than zero"));
        }
        if self.max_attempts == 0 {
            return Err(Error::config(
                "maximum polling attempts must be greater than zero",
            ));
        }
        Ok(())
    }
}

/// Configuration shared by the asynchronous and blocking clients.
///
/// All builder settings are optional and use the same defaults as
/// [`Self::default`]. The client resolves the key and validates the settings
/// when it is created.
///
/// ```
/// use std::time::Duration;
/// use ezcapsolver::ClientConfig;
///
/// let config = ClientConfig::builder()
///     .timeout(Duration::from_secs(60))
///     .user_agent("my-app/1.0")
///     .build();
/// assert_eq!(config.timeout, Duration::from_secs(60));
/// assert_eq!(config.sync_timeout, Duration::from_secs(240));
/// ```
#[derive(Clone, PartialEq, Eq, bon::Builder)]
#[builder(on(String, into))]
pub struct ClientConfig {
    /// Optional key; when unset, the client reads `EZCAPTCHA_API_KEY`.
    client_key: Option<String>,
    /// Base URL for asynchronous task operations and balance queries.
    ///
    /// Override it to reach a private gateway, or to point the client at a test
    /// server. The service splits the two deployments, so they cannot share one
    /// host — see [`Self::sync_base_url`].
    #[builder(default = DEFAULT_ASYNC_BASE_URL.to_owned())]
    pub async_base_url: String,
    /// Base URL for synchronous task operations.
    #[builder(default = DEFAULT_SYNC_BASE_URL.to_owned())]
    pub sync_base_url: String,
    /// Timeout applied to asynchronous endpoints and balance queries.
    ///
    /// These are millisecond-scale enqueue and lookup calls, so a short value
    /// keeps a network fault distinguishable from a slow service.
    #[builder(default = DEFAULT_TIMEOUT)]
    pub timeout: Duration,
    /// Timeout applied to the synchronous task endpoint.
    ///
    /// A synchronous call blocks until the worker answers, and the service
    /// grants some task types a three-minute worker deadline. Sharing the
    /// asynchronous timeout would abort a call the service is still billing
    /// for, and a call that times out never receives the task ID it would take
    /// to recover the result.
    #[builder(default = DEFAULT_SYNC_TIMEOUT)]
    pub sync_timeout: Duration,
    /// Polling settings for asynchronous task completion.
    #[builder(default)]
    pub polling: PollingConfig,
    /// Optional developer application identifier.
    pub app_id: Option<i32>,
    /// Optional proxy used by the SDK to reach the EzCaptchaSolver API.
    pub proxy: Option<String>,
    /// User agent sent with every request.
    #[builder(default = default_user_agent())]
    pub user_agent: String,
}

/// Default user agent identifying this SDK and its version.
fn default_user_agent() -> String {
    concat!("ezcapsolver-rs/", env!("CARGO_PKG_VERSION")).to_owned()
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            client_key: None,
            async_base_url: DEFAULT_ASYNC_BASE_URL.to_owned(),
            sync_base_url: DEFAULT_SYNC_BASE_URL.to_owned(),
            timeout: DEFAULT_TIMEOUT,
            sync_timeout: DEFAULT_SYNC_TIMEOUT,
            polling: PollingConfig::default(),
            app_id: None,
            proxy: None,
            user_agent: default_user_agent(),
        }
    }
}

impl ClientConfig {
    /// Sets the client key instead of loading it from the default environment variable.
    #[must_use]
    pub fn with_client_key(mut self, value: impl Into<String>) -> Self {
        self.client_key = Some(value.into());
        self
    }

    /// Points the client at different hosts, for a self-hosted deployment or a
    /// test server. An empty string keeps the current value.
    #[must_use]
    pub fn with_base_urls(
        mut self,
        async_base_url: impl Into<String>,
        sync_base_url: impl Into<String>,
    ) -> Self {
        let async_base_url = async_base_url.into();
        if !async_base_url.is_empty() {
            self.async_base_url = async_base_url;
        }
        let sync_base_url = sync_base_url.into();
        if !sync_base_url.is_empty() {
            self.sync_base_url = sync_base_url;
        }
        self
    }

    /// Sets the timeout for asynchronous endpoints and balance queries.
    #[must_use]
    pub const fn with_timeout(mut self, value: Duration) -> Self {
        self.timeout = value;
        self
    }

    /// Sets the timeout for the synchronous task endpoint.
    #[must_use]
    pub const fn with_sync_timeout(mut self, value: Duration) -> Self {
        self.sync_timeout = value;
        self
    }

    /// Sets asynchronous task polling options.
    #[must_use]
    pub const fn with_polling(mut self, value: PollingConfig) -> Self {
        self.polling = value;
        self
    }

    /// Sets the optional developer application identifier.
    #[must_use]
    pub const fn with_app_id(mut self, value: i32) -> Self {
        self.app_id = Some(value);
        self
    }

    /// Sets the proxy used by the SDK HTTP client.
    #[must_use]
    pub fn with_proxy(mut self, value: impl Into<String>) -> Self {
        self.proxy = Some(value.into());
        self
    }

    /// Overrides the user agent sent with every request.
    #[must_use]
    pub fn with_user_agent(mut self, value: impl Into<String>) -> Self {
        self.user_agent = value.into();
        self
    }

    #[cfg(any(feature = "async", feature = "blocking"))]
    pub(crate) fn validate(&self) -> Result<()> {
        if self.async_base_url.trim().is_empty() || self.sync_base_url.trim().is_empty() {
            return Err(Error::config("base URLs must not be blank"));
        }
        if self.timeout.is_zero() {
            return Err(Error::config("timeout must be greater than zero"));
        }
        if self.sync_timeout.is_zero() {
            return Err(Error::config("sync timeout must be greater than zero"));
        }
        self.polling.validate()
    }

    #[cfg(any(feature = "async", feature = "blocking"))]
    pub(crate) fn resolve_client_key(&self) -> Result<String> {
        let client_key = match &self.client_key {
            Some(client_key) => client_key.clone(),
            None => std::env::var(DEFAULT_CLIENT_KEY_ENV).map_err(|source| {
                Error::config(format!(
                    "client key environment variable `{DEFAULT_CLIENT_KEY_ENV}` is unavailable: {source}"
                ))
            })?,
        };
        if client_key.trim().is_empty() {
            return Err(Error::config("client key must not be blank"));
        }
        Ok(client_key)
    }
}

impl fmt::Debug for ClientConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClientConfig")
            .field(
                "client_key",
                &self.client_key.as_ref().map(|_| "[REDACTED]"),
            )
            .field("async_base_url", &self.async_base_url)
            .field("sync_base_url", &self.sync_base_url)
            .field("timeout", &self.timeout)
            .field("sync_timeout", &self.sync_timeout)
            .field("polling", &self.polling)
            .field("app_id", &self.app_id)
            .field("proxy", &self.proxy.as_ref().map(|_| "[REDACTED]"))
            .field("user_agent", &self.user_agent)
            .finish()
    }
}

impl From<String> for ClientConfig {
    fn from(client_key: String) -> Self {
        Self::default().with_client_key(client_key)
    }
}

impl From<&str> for ClientConfig {
    fn from(client_key: &str) -> Self {
        Self::from(client_key.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synchronous_and_asynchronous_timeouts_have_distinct_defaults() {
        let config = ClientConfig::default();

        // The synchronous endpoint blocks until the worker answers, which the
        // service allows up to three minutes; the asynchronous one only enqueues
        // and polls. One shared value would have to be wrong for one of them.
        assert_eq!(config.timeout, Duration::from_secs(30));
        // Strictly greater than the service's 180s worker deadline, or a worker
        // that uses its full budget gets cut off by the client.
        assert!(config.sync_timeout > Duration::from_secs(180));
        assert_eq!(config.sync_timeout, Duration::from_secs(240));
        assert_ne!(config.timeout, config.sync_timeout);
    }

    #[test]
    fn polling_defaults_match_the_specified_budget() {
        let polling = PollingConfig::default();

        assert_eq!(polling.interval, Duration::from_secs(3));
        assert_eq!(polling.max_attempts, 50);
    }

    #[test]
    fn debug_output_redacts_credentials() {
        let config = ClientConfig::default()
            .with_client_key("secret-client-key")
            .with_proxy("http://user:password@127.0.0.1:8080");
        let output = format!("{config:?}");

        assert!(!output.contains("secret-client-key"));
        assert!(!output.contains("password"));
        assert!(output.contains("[REDACTED]"));
    }
}
