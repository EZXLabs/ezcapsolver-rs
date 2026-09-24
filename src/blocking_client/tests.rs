use std::{
    io::Write,
    sync::{Arc, Mutex},
    time::Duration,
};

use mockito::{Matcher, Server};
use serde_json::json;
use tracing_subscriber::fmt::MakeWriter;

use super::*;
use crate::{ReClassificationSolution, RecaptchaV2ClassificationTask, RecaptchaV2Task, TaskType};

/// Pins the process-wide maximum log level at `TRACE`.
///
/// `tracing` keeps that maximum in a global, and rebuilds it whenever any
/// thread registers a callsite. A scoped subscriber only exists on the thread
/// that installed it, so a rebuild triggered by a test running in parallel
/// recomputes the maximum as `OFF` and this test silently stops recording
/// halfway through. Installing a global subscriber that discards its output
/// keeps the maximum at `TRACE` regardless of which thread rebuilds it.
fn pin_global_trace_level() {
    static ONCE: std::sync::Once = std::sync::Once::new();

    ONCE.call_once(|| {
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::TRACE)
            .with_writer(std::io::sink)
            .finish();
        let _ = tracing::subscriber::set_global_default(subscriber);
    });
}

/// In-memory log sink used to assert on emitted trace output.
#[derive(Clone, Default)]
struct CapturedLogs(Arc<Mutex<Vec<u8>>>);

impl CapturedLogs {
    fn contents(&self) -> std::result::Result<String, Box<dyn std::error::Error>> {
        let buffer = self.0.lock().map_err(|_| "log buffer was poisoned")?;
        Ok(String::from_utf8_lossy(&buffer).into_owned())
    }
}

impl Write for CapturedLogs {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0
            .lock()
            .map_err(|_| std::io::Error::other("log buffer was poisoned"))?
            .extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'writer> MakeWriter<'writer> for CapturedLogs {
    type Writer = Self;

    fn make_writer(&'writer self) -> Self::Writer {
        self.clone()
    }
}

#[test]
fn client_can_be_created_with_an_explicit_key() -> Result<()> {
    let client = EzCapSolverClient::with_client_key("client-key")?;

    assert_eq!(client.client_key, "client-key");
    Ok(())
}

/// Every endpoint carries the client key in the `X-API-Key` header, on
/// top of the `clientKey` body field. It sends no `X-Request-Id`: the
/// gateway assigns the correlation id and the SDK only reads it back.
#[test]
fn every_request_carries_the_client_key_header()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut server = Server::new();
    let ready = r#"{"errorId":0,"status":"ready","solution":{"token":"token-value"}}"#;
    let mocks: Vec<_> = [
        ("/createTask", r#"{"errorId":0,"taskId":"task-123"}"#),
        ("/getTaskResult", ready),
        ("/createSyncTask", ready),
        ("/getBalance", r#"{"errorId":0,"balance":1.5}"#),
    ]
    .into_iter()
    .map(|(path, body)| {
        server
            .mock("POST", path)
            .match_header("X-API-Key", "client-key")
            .match_header("X-Request-Id", Matcher::Missing)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body)
            .expect(1)
            .create()
    })
    .collect();

    let client = EzCapSolverClient::with_config(
        ClientConfig::default()
            .with_client_key("client-key")
            .with_polling(PollingConfig::new(Duration::from_millis(1), 1))
            .with_base_urls(server.url(), server.url()),
    )?;
    client.solve("CustomTask", &json!({}))?;
    client.sync_solve("CustomTask", &json!({}))?;
    client.balance()?;

    for mock in mocks {
        mock.assert();
    }
    Ok(())
}

/// A key that cannot travel as a header value fails when the client is built,
/// not on every request afterwards.
#[test]
fn a_client_key_that_is_not_a_valid_header_value_is_rejected() {
    let error = EzCapSolverClient::with_client_key("client\nkey").err();

    assert!(matches!(
        error,
        Some(Error::Message(message)) if message.contains("invalid characters")
    ));
}

#[test]
fn trace_logging_records_the_exchange_without_leaking_credentials()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    pin_global_trace_level();

    let mut server = Server::new();
    let create_mock = server
        .mock("POST", "/createTask")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"errorId":0,"taskId":"task-123"}"#)
        .create();

    let logs = CapturedLogs::default();
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_ansi(false)
        .with_writer(logs.clone())
        .finish();

    let client = EzCapSolverClient::with_config(
        ClientConfig::default()
            .with_client_key("super-secret-key")
            .with_base_urls(server.url(), server.url()),
    )?;
    let task = RecaptchaV2Task {
        website_url: "https://example.com".to_owned(),
        website_key: "site-key".to_owned(),
        proxy: Some("http://user:hunter2@127.0.0.1:8080".to_owned()),
        ..Default::default()
    };

    tracing::subscriber::with_default(subscriber, || {
        client.create_task(TaskType::RecaptchaV2TaskProxyless, &task)
    })?;

    let output = logs.contents()?;
    assert!(
        output.contains("sending API request"),
        "the request body was never traced"
    );
    assert!(
        output.contains("received API response"),
        "the response was never traced"
    );
    assert!(
        output.contains("websiteKey"),
        "tracing must still show the task parameters"
    );
    assert!(
        !output.contains("super-secret-key"),
        "the client key leaked into the logs"
    );
    assert!(
        !output.contains("hunter2"),
        "proxy credentials leaked into the logs"
    );
    assert!(output.contains("[REDACTED]"));
    create_mock.assert();
    Ok(())
}

#[test]
fn sync_solve_accepts_a_custom_type_without_polling()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut async_server = Server::new();
    let mut sync_server = Server::new();
    let async_mock = async_server.mock("POST", Matcher::Any).expect(0).create();
    let solution = json!({"token": "token-value", "futureField": {"items": [1, null, true]}});
    let sync_mock = sync_server
        .mock("POST", "/createSyncTask")
        .match_body(Matcher::Json(json!({
            "clientKey": "client-key",
            "appId": 42,
            "task": {
                "type": "BrandNewSyncTaskType",
                "websiteURL": "https://example.com",
                "futureParam": {"enabled": true}
            }
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "errorId": 0,
                "status": "ready",
                "requestId": "request-sync",
                "solution": solution
            })
            .to_string(),
        )
        .expect(1)
        .create();

    let client = EzCapSolverClient::with_config(
        ClientConfig::default()
            .with_client_key("client-key")
            .with_app_id(42)
            .with_base_urls(async_server.url(), sync_server.url()),
    )?;
    let task = json!({
        "type": "IncorrectTaskType",
        "websiteURL": "https://example.com",
        "futureParam": {"enabled": true}
    });
    let solved: Solved<Solution> = client.sync_solve("BrandNewSyncTaskType", &task)?;

    assert_eq!(solved.task_id, None);
    assert_eq!(solved.request_id.as_deref(), Some("request-sync"));
    assert_eq!(solved.solution.as_value(), Some(&solution));
    assert_eq!(solved.raw, solved.solution);
    sync_mock.assert();
    async_mock.assert();
    Ok(())
}

#[test]
fn sync_solve_preserves_missing_and_null_solutions()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    for (body, expected) in [
        (r#"{"errorId":0,"status":"ready"}"#, Solution::Missing),
        (
            r#"{"errorId":0,"status":"ready","solution":null}"#,
            Solution::Value(json!(null)),
        ),
    ] {
        let mut server = Server::new();
        let sync_mock = server
            .mock("POST", "/createSyncTask")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create();
        let client = EzCapSolverClient::with_config(
            ClientConfig::default()
                .with_client_key("client-key")
                .with_base_urls(server.url(), server.url()),
        )?;

        let solved = client.sync_solve(TaskType::Hcaptcha, &json!({}))?;

        assert_eq!(solved.task_id, None);
        assert_eq!(solved.request_id, None);
        assert_eq!(solved.solution, expected);
        assert_eq!(solved.raw, expected);
        sync_mock.assert();
    }
    Ok(())
}

#[test]
fn sync_solve_rejects_non_ready_results() -> std::result::Result<(), Box<dyn std::error::Error>> {
    for status in ["processing", "error"] {
        let mut server = Server::new();
        let sync_mock = server
            .mock("POST", "/createSyncTask")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "errorId": 0,
                    "status": status,
                    "solution": {"token": "not-ready"}
                })
                .to_string(),
            )
            .create();
        let client = EzCapSolverClient::with_config(
            ClientConfig::default()
                .with_client_key("client-key")
                .with_base_urls(server.url(), server.url()),
        )?;

        let result = client.sync_solve("BrandNewSyncTaskType", &json!({}));

        assert!(
            matches!(result, Err(Error::UnexpectedResponse(message)) if message.contains(status))
        );
        sync_mock.assert();
    }
    Ok(())
}

#[test]
fn sync_solve_preserves_api_error_details() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut server = Server::new();
    let sync_mock = server
        .mock("POST", "/createSyncTask")
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "errorId": 1,
                "errorCode": "ERROR_REQUEST_PARAMETERS",
                "errorDescription": "Invalid parameters",
                "requestId": "request-sync-error",
                "errors": {"task.websiteURL": "must not be blank"}
            })
            .to_string(),
        )
        .create();
    let client = EzCapSolverClient::with_config(
        ClientConfig::default()
            .with_client_key("client-key")
            .with_base_urls(server.url(), server.url()),
    )?;

    let result = client.sync_solve("BrandNewSyncTaskType", &json!({}));

    let Err(Error::EzError(api_error)) = result else {
        return Err(std::io::Error::other("expected a structured API error").into());
    };
    assert_eq!(
        api_error.error_code.as_deref(),
        Some("ERROR_REQUEST_PARAMETERS")
    );
    assert_eq!(
        api_error.error_description.as_deref(),
        Some("Invalid parameters")
    );
    assert_eq!(api_error.request_id.as_deref(), Some("request-sync-error"));
    assert_eq!(api_error.task_id, None);
    assert_eq!(api_error.http_status, 400);
    assert_eq!(
        api_error.errors.get("task.websiteURL").map(String::as_str),
        Some("must not be blank")
    );
    sync_mock.assert();
    Ok(())
}

#[test]
fn specialized_sync_solver_returns_a_decoded_solution()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut server = Server::new();
    let sync_mock = server
        .mock("POST", "/createSyncTask")
        .match_body(Matcher::Json(json!({
            "clientKey": "client-key",
            "task": {
                "type": "ReCaptchaV2Classification",
                "image": "aW1n",
                "question": "/m/0k4j",
                "size": 4
            }
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "errorId":0,
                "status":"ready",
                "solution":{"type":"multi","objects":[0,3,7]}
            }"#,
        )
        .create();

    let client = EzCapSolverClient::with_config(
        ClientConfig::default()
            .with_client_key("client-key")
            .with_base_urls(server.url(), server.url()),
    )?;
    let task = RecaptchaV2ClassificationTask {
        image: "aW1n".to_owned(),
        question: "/m/0k4j".to_owned(),
        ..Default::default()
    };

    let solution: ReClassificationSolution = client
        .sync_solve_recaptcha_v2_classification(&task)?
        .solution;
    assert!(solution.is_multi());
    assert!(!solution.is_single());
    assert_eq!(solution.r#type, "multi");
    assert_eq!(solution.objects, vec![0, 3, 7]);
    sync_mock.assert();
    Ok(())
}

#[test]
fn a_synchronous_call_uses_the_synchronous_timeout()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    // The synchronous endpoint blocks until the worker answers, which the service
    // allows up to three minutes. The asynchronous endpoint's 30 seconds would cut
    // the call off while the service was still working, and a timed-out call is
    // billed yet returns no taskId to recover the result with.
    let client = EzCapSolverClient::with_config(
        ClientConfig::default()
            .with_client_key("client-key")
            .with_timeout(Duration::from_secs(1)),
    )?;

    assert_eq!(client.config().timeout, Duration::from_secs(1));
    assert_eq!(client.config().sync_timeout, Duration::from_secs(240));
    Ok(())
}

/// The two clients carry their own copy of the polling loop, so the throttling
/// retry has to be proven on both. A task refused by the rate limiter is still
/// queued, and it has already been billed.
#[test]
fn a_throttled_poll_is_retried_instead_of_failing_the_task()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut server = Server::new();
    let throttled = server
        .mock("POST", "/getTaskResult")
        .with_status(429)
        .with_header("content-type", "application/json")
        .with_body(r#"{"errorId":1,"errorCode":"ERROR_REQUEST_BANNED"}"#)
        .expect(1)
        .create();
    let ready = server
        .mock("POST", "/getTaskResult")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"errorId":0,"status":"ready","solution":{"token":"t"}}"#)
        .expect(1)
        .create();

    let client = EzCapSolverClient::with_config(
        ClientConfig::default()
            .with_client_key("client-key")
            .with_base_urls(server.url(), server.url()),
    )?;
    let result =
        client.wait_for_result_with("task-123", PollingConfig::new(Duration::from_millis(1), 3))?;

    assert_eq!(result.status, TaskStatus::Ready);
    throttled.assert();
    ready.assert();
    Ok(())
}
