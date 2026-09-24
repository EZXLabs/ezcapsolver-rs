//! Client configuration builder defaults, conversions, and validation.

use std::time::Duration;

use ezcapsolver::{ClientConfig, ClientConfigBuilder, PollingConfig};

#[test]
fn an_empty_builder_matches_default_configuration() {
    let builder: ClientConfigBuilder = ClientConfig::builder();
    assert_eq!(builder.build(), ClientConfig::default());
}

#[test]
fn builder_settings_match_existing_configuration_methods() {
    let user_agent = "my-app/1.0".to_owned();
    let config = ClientConfig::builder()
        .client_key("test-client-key")
        .timeout(Duration::from_secs(60))
        .sync_timeout(Duration::from_secs(240))
        .polling(PollingConfig::new(Duration::from_secs(5), 24))
        .app_id(42)
        .proxy("http://127.0.0.1:8080".to_owned())
        .user_agent(&user_agent)
        .build();

    let expected = ClientConfig::default()
        .with_client_key("test-client-key")
        .with_timeout(Duration::from_secs(60))
        .with_sync_timeout(Duration::from_secs(240))
        .with_polling(PollingConfig::new(Duration::from_secs(5), 24))
        .with_app_id(42)
        .with_proxy("http://127.0.0.1:8080")
        .with_user_agent(user_agent);
    assert_eq!(config, expected);
}

#[test]
fn optional_setters_preserve_defaults_and_explicit_zero() {
    let client_key: Option<String> = None;
    let proxy: Option<String> = None;
    let user_agent: Option<String> = None;
    let config = ClientConfig::builder()
        .maybe_client_key(client_key)
        .maybe_proxy(proxy)
        .maybe_user_agent(user_agent)
        .maybe_timeout(None)
        .maybe_sync_timeout(None)
        .maybe_polling(None)
        .maybe_app_id(Some(0))
        .build();

    assert_eq!(config, ClientConfig::default().with_app_id(0));

    let config = ClientConfig::builder()
        .maybe_client_key(Some("test-client-key"))
        .maybe_proxy(Some("http://127.0.0.1:8080"))
        .maybe_user_agent(Some("my-app/1.0"))
        .maybe_app_id(None)
        .build();
    assert_eq!(
        config,
        ClientConfig::default()
            .with_client_key("test-client-key")
            .with_proxy("http://127.0.0.1:8080")
            .with_user_agent("my-app/1.0")
    );
}

#[test]
fn builder_configuration_debug_output_redacts_credentials() {
    let config = ClientConfig::builder()
        .client_key("secret-client-key")
        .proxy("http://user:secret-password@127.0.0.1:8080")
        .build();
    let output = format!("{config:?}");

    assert!(!output.contains("secret-client-key"));
    assert!(!output.contains("secret-password"));
    assert!(output.contains("[REDACTED]"));
}

#[cfg(any(feature = "async", feature = "blocking"))]
#[test]
fn clients_accept_builder_configuration() -> ezcapsolver::Result<()> {
    let config = ClientConfig::builder()
        .client_key("test-client-key")
        .timeout(Duration::from_secs(60))
        .app_id(42)
        .build();

    #[cfg(feature = "async")]
    assert_eq!(
        ezcapsolver::AsyncEzCapSolverClient::with_config(config.clone())?.config(),
        &config
    );
    #[cfg(feature = "blocking")]
    assert_eq!(
        ezcapsolver::EzCapSolverClient::with_config(config.clone())?.config(),
        &config
    );
    Ok(())
}

#[cfg(any(feature = "async", feature = "blocking"))]
#[test]
fn invalid_builder_settings_are_rejected_during_client_creation() {
    let configurations = [
        ClientConfig::builder().timeout(Duration::ZERO).build(),
        ClientConfig::builder().sync_timeout(Duration::ZERO).build(),
        ClientConfig::builder()
            .polling(PollingConfig::new(Duration::ZERO, 1))
            .build(),
        ClientConfig::builder()
            .polling(PollingConfig::new(Duration::from_secs(1), 0))
            .build(),
    ];

    // Assert the prefix too: this is the general text-carrying variant, so matching
    // the variant alone would let any failure pass.
    for config in configurations {
        let config = config.with_client_key("test-client-key");
        #[cfg(feature = "async")]
        assert!(matches!(
            ezcapsolver::AsyncEzCapSolverClient::with_config(config.clone()),
            Err(ezcapsolver::Error::Message(message))
                if message.starts_with("invalid client configuration")
        ));
        #[cfg(feature = "blocking")]
        assert!(matches!(
            ezcapsolver::EzCapSolverClient::with_config(config.clone()),
            Err(ezcapsolver::Error::Message(message))
                if message.starts_with("invalid client configuration")
        ));
    }
}

/// Base URLs have to be overridable: without that a private gateway is
/// unreachable, and downstream projects cannot point integration tests at a
/// mock server.
#[test]
fn base_urls_are_configurable_and_default_to_the_service() {
    let defaults = ClientConfig::default();
    assert_eq!(defaults.async_base_url, ezcapsolver::DEFAULT_ASYNC_BASE_URL);
    assert_eq!(defaults.sync_base_url, ezcapsolver::DEFAULT_SYNC_BASE_URL);

    let overridden =
        ClientConfig::default().with_base_urls("http://127.0.0.1:1", "http://127.0.0.1:2");
    assert_eq!(overridden.async_base_url, "http://127.0.0.1:1");
    assert_eq!(overridden.sync_base_url, "http://127.0.0.1:2");

    // An empty string keeps the current value, so one side can be changed alone.
    let one_sided = ClientConfig::default().with_base_urls("http://127.0.0.1:1", "");
    assert_eq!(one_sided.async_base_url, "http://127.0.0.1:1");
    assert_eq!(one_sided.sync_base_url, ezcapsolver::DEFAULT_SYNC_BASE_URL);

    // The builder path sets it too.
    let built = ClientConfig::builder()
        .async_base_url("http://gateway.internal")
        .build();
    assert_eq!(built.async_base_url, "http://gateway.internal");
}

/// Debug shows the base URLs, which is what tells a wrong environment apart, but
/// that must not come at the cost of leaking the credentials.
#[test]
fn debug_output_shows_endpoints_without_leaking_credentials() {
    let rendered = format!(
        "{:?}",
        ClientConfig::default()
            .with_client_key("secret-client-key")
            .with_base_urls("http://gateway.internal", "http://sync.internal")
    );
    assert!(rendered.contains("http://gateway.internal"));
    assert!(rendered.contains("http://sync.internal"));
    assert!(!rendered.contains("secret-client-key"));
}
