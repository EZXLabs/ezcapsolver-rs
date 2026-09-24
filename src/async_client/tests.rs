use std::{sync::Arc, time::Duration};

use mockito::{Matcher, Server};
use serde_json::json;

use super::*;
use crate::{
    ReClassificationSolution, RecaptchaSolution, RecaptchaV2ClassificationTask, RecaptchaV2Task,
    Solved, TaskType,
};

#[tokio::test]
async fn solve_posts_the_expected_requests_and_decodes_the_solution()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut server = Server::new_async().await;
    let create_mock = server
        .mock("POST", "/createTask")
        .match_header(
            "content-type",
            Matcher::Regex("application/json.*".to_owned()),
        )
        .match_body(Matcher::Json(json!({
            "clientKey": "client-key",
            "appId": 42,
            "task": {
                "type": "ReCaptchaV2TaskProxyless",
                "websiteURL": "https://example.com",
                "websiteKey": "site-key",
                "isInvisible": false
            }
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"errorId":0,"requestId":"request-1","taskId":"task-123"}"#)
        .create_async()
        .await;
    let result_mock = server
        .mock("POST", "/getTaskResult")
        .match_body(Matcher::Json(json!({
            "clientKey": "client-key",
            "taskId": "task-123"
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "errorId":0,
                "status":"ready",
                "solution":{
                    "gRecaptchaResponse":"token-value",
                    "sec_ch_ua":"\"Chromium\";v=\"140\"",
                    "user_agent":"Mozilla/5.0"
                }
            }"#,
        )
        .create_async()
        .await;

    let config = ClientConfig::default()
        .with_app_id(42)
        .with_polling(PollingConfig::new(Duration::from_millis(1), 2));
    let client = test_client(config, &server)?;
    let mut task = RecaptchaV2Task {
        website_url: "https://example.com".to_owned(),
        website_key: "site-key".to_owned(),
        ..Default::default()
    };
    task.extra
        .insert("type".to_owned(), json!("IncorrectTaskType"));

    let completed: Solved<RecaptchaSolution> =
        client.solve_recaptcha_v2_task_proxyless(&task).await?;
    assert_eq!(completed.task_id.as_deref(), Some("task-123"));
    assert_eq!(completed.request_id.as_deref(), Some("request-1"));
    assert_eq!(completed.solution.token, "token-value");
    create_mock.assert_async().await;
    result_mock.assert_async().await;
    Ok(())
}

#[tokio::test]
async fn sync_solve_accepts_a_custom_type_without_polling()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut async_server = Server::new_async().await;
    let mut sync_server = Server::new_async().await;
    let async_mock = async_server
        .mock("POST", Matcher::Any)
        .expect(0)
        .create_async()
        .await;
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
        .create_async()
        .await;

    let client = AsyncEzCapSolverClient::with_config(
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
    let solved: Solved<Solution> = client.sync_solve("BrandNewSyncTaskType", &task).await?;

    assert_eq!(solved.task_id, None);
    assert_eq!(solved.request_id.as_deref(), Some("request-sync"));
    assert_eq!(solved.solution.as_value(), Some(&solution));
    assert_eq!(solved.raw, solved.solution);
    sync_mock.assert_async().await;
    async_mock.assert_async().await;
    Ok(())
}

/// The synchronous endpoint assigns and returns a taskId on both the success and
/// the failure path. Dropping it discards the only handle on a billed task.
#[tokio::test]
async fn sync_solve_keeps_the_identifier_the_service_assigned()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut sync_server = Server::new_async().await;
    let sync_mock = sync_server
        .mock("POST", "/createSyncTask")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "errorId": 0,
                "requestId": "request-sync",
                "taskId": "sync-task-1",
                "status": "ready",
                "solution": {"gRecaptchaResponse": "token-value"}
            })
            .to_string(),
        )
        .expect(2)
        .create_async()
        .await;

    let client = AsyncEzCapSolverClient::with_config(
        ClientConfig::default()
            .with_client_key("client-key")
            .with_base_urls(sync_server.url(), sync_server.url()),
    )?;

    // The general entry point.
    let raw: Solved<Solution> = client
        .sync_solve(TaskType::RecaptchaV2TaskProxyless, &json!({}))
        .await?;
    assert_eq!(raw.task_id.as_deref(), Some("sync-task-1"));

    // The typed convenience method goes through decode_sync_result, a second
    // code path.
    let typed = client
        .sync_solve_recaptcha_v2_task_proxyless(&RecaptchaV2Task::default())
        .await?;
    assert_eq!(typed.task_id.as_deref(), Some("sync-task-1"));
    assert_eq!(typed.solution.token, "token-value");

    sync_mock.assert_async().await;
    Ok(())
}

#[tokio::test]
async fn sync_solve_preserves_missing_and_null_solutions()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    for (body, expected) in [
        (r#"{"errorId":0,"status":"ready"}"#, Solution::Missing),
        (
            r#"{"errorId":0,"status":"ready","solution":null}"#,
            Solution::Value(json!(null)),
        ),
    ] {
        let mut server = Server::new_async().await;
        let sync_mock = server
            .mock("POST", "/createSyncTask")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create_async()
            .await;
        let client = test_client(ClientConfig::default(), &server)?;

        let solved = client.sync_solve(TaskType::Hcaptcha, &json!({})).await?;

        assert_eq!(solved.task_id, None);
        assert_eq!(solved.request_id, None);
        assert_eq!(solved.solution, expected);
        assert_eq!(solved.raw, expected);
        sync_mock.assert_async().await;
    }
    Ok(())
}

#[tokio::test]
async fn sync_solve_rejects_non_ready_results()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    for status in ["processing", "error"] {
        let mut server = Server::new_async().await;
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
            .create_async()
            .await;
        let client = test_client(ClientConfig::default(), &server)?;

        let result = client.sync_solve("BrandNewSyncTaskType", &json!({})).await;

        assert!(
            matches!(result, Err(Error::UnexpectedResponse(message)) if message.contains(status))
        );
        sync_mock.assert_async().await;
    }
    Ok(())
}

#[tokio::test]
async fn sync_solve_preserves_api_error_details()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut server = Server::new_async().await;
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
        .create_async()
        .await;
    let client = test_client(ClientConfig::default(), &server)?;

    let result = client.sync_solve("BrandNewSyncTaskType", &json!({})).await;

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
    sync_mock.assert_async().await;
    Ok(())
}

#[tokio::test]
async fn specialized_sync_solver_returns_a_classification_struct()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut server = Server::new_async().await;
    let sync_mock = server
        .mock("POST", "/createSyncTask")
        .match_body(Matcher::Json(json!({
            "clientKey": "client-key",
            "task": {
                "type": "ReCaptchaV2Classification",
                "image": "aW1n",
                "question": "question",
                "size": 1
            }
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "errorId": 0,
                "status": "ready",
                "solution": {"type": "single", "hasObject": false, "confidence": 0.9}
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = test_client(ClientConfig::default(), &server)?;
    let task = RecaptchaV2ClassificationTask {
        image: "aW1n".to_owned(),
        question: "question".to_owned(),
        size: 1,
        ..Default::default()
    };
    let solution: ReClassificationSolution = client
        .sync_solve_recaptcha_v2_classification(&task)
        .await?
        .solution;

    assert_eq!(solution.r#type, "single");
    assert!(solution.is_single());
    assert!(!solution.is_multi());
    assert!(!solution.has_object);
    assert!(solution.objects.is_empty());
    assert_eq!(solution.extra["confidence"], json!(0.9));
    sync_mock.assert_async().await;
    Ok(())
}

/// The mode a type documents does not restrict which methods exist: the
/// `sync_`-prefixed one sends a polling type to `/createSyncTask`, and it is the
/// endpoint that changes, not the parameters.
#[tokio::test]
async fn sync_prefixed_method_sends_a_polling_type_to_the_synchronous_endpoint()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut server = Server::new_async().await;
    let sync_mock = server
        .mock("POST", "/createSyncTask")
        .match_body(Matcher::Json(json!({
            "clientKey": "client-key",
            "task": {
                "type": "ReCaptchaV2TaskProxyless",
                "websiteURL": "https://example.com",
                "websiteKey": "site-key",
                "isInvisible": false
            }
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "errorId": 0,
                "status": "ready",
                "solution": {
                    "gRecaptchaResponse": "token-value",
                    "sec_ch_ua": "Chromium",
                    "user_agent": "Mozilla/5.0",
                    "aFieldAddedLater": 7
                }
            })
            .to_string(),
        )
        .create_async()
        .await;

    let client = test_client(ClientConfig::default(), &server)?;
    let task = RecaptchaV2Task {
        website_url: "https://example.com".to_owned(),
        website_key: "site-key".to_owned(),
        ..Default::default()
    };
    let solved = client.sync_solve_recaptcha_v2_task_proxyless(&task).await?;

    assert_eq!(solved.solution.token, "token-value");
    // This endpoint assigns no task id, so the field has to be None rather than a
    // placeholder empty string.
    assert!(solved.task_id.is_none());
    // Unmodelled fields stay recoverable from raw.
    assert_eq!(
        solved
            .raw
            .as_value()
            .and_then(|v| v.get("aFieldAddedLater")),
        Some(&json!(7))
    );
    sync_mock.assert_async().await;
    Ok(())
}

/// The mirror of the above: the unprefixed method creates and polls a type the
/// catalog documents as synchronous.
#[tokio::test]
async fn unprefixed_method_polls_even_a_type_documented_as_synchronous()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut server = Server::new_async().await;
    let create_mock = server
        .mock("POST", "/createTask")
        .match_body(Matcher::PartialJson(
            json!({"task": {"type": "ReCaptchaV2Classification"}}),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!({"errorId": 0, "taskId": "task-77"}).to_string())
        .create_async()
        .await;
    let result_mock = server
        .mock("POST", "/getTaskResult")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "errorId": 0,
                "status": "ready",
                "solution": {"type": "multi", "objects": [1, 5]}
            })
            .to_string(),
        )
        .create_async()
        .await;

    let config =
        ClientConfig::default().with_polling(PollingConfig::new(Duration::from_millis(1), 2));
    let client = test_client(config, &server)?;
    let task = RecaptchaV2ClassificationTask {
        image: "aW1n".to_owned(),
        question: "question".to_owned(),
        ..Default::default()
    };
    let solved = client.solve_recaptcha_v2_classification(&task).await?;

    assert!(solved.solution.is_multi());
    assert_eq!(solved.solution.objects, vec![1, 5]);
    assert_eq!(solved.task_id.as_deref(), Some("task-77"));
    create_mock.assert_async().await;
    result_mock.assert_async().await;
    Ok(())
}

#[tokio::test]
async fn create_task_accepts_dynamic_json_parameters()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut server = Server::new_async().await;
    let create_mock = server
        .mock("POST", "/createTask")
        .match_body(Matcher::Json(json!({
            "clientKey": "client-key",
            "task": {
                "type": "HCaptcha",
                "websiteURL": "https://example.com",
                "websiteKey": "site-key",
                "futureWorkerOption": true
            }
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"errorId":0,"requestId":"request-dynamic","taskId":"task-dynamic"}"#)
        .create_async()
        .await;

    let client = test_client(ClientConfig::default(), &server)?;
    let params = json!({
        "websiteURL": "https://example.com",
        "websiteKey": "site-key",
        "futureWorkerOption": true
    });

    let created = client.create_task(TaskType::Hcaptcha, &params).await?;

    assert_eq!(created.task_id, "task-dynamic");
    assert_eq!(created.meta.request_id.as_deref(), Some("request-dynamic"));
    create_mock.assert_async().await;
    Ok(())
}

#[tokio::test]
async fn balance_returns_the_value_the_service_reported()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    // Balances carry at most four decimal places, and the API sends them as a
    // JSON number. Every value in that range must survive unchanged.
    for raw in [
        "0",
        "250",
        "0.0001",
        "12.3456",
        "1234.5678",
        "9999999999.9999",
    ] {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/getBalance")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(format!(r#"{{"errorId":0,"balance":{raw}}}"#))
            .create_async()
            .await;

        let client = test_client(ClientConfig::default(), &server)?;
        let balance = client.balance().await?;

        assert_eq!(balance.to_string(), raw, "balance {raw} changed");
        mock.assert_async().await;
    }
    Ok(())
}

#[tokio::test]
async fn validation_errors_are_visible_in_the_error_message()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut server = Server::new_async().await;
    let create_mock = server
        .mock("POST", "/createTask")
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "errorId":1,
                "errorCode":"ERROR_REQUEST_PARAMETERS",
                "errorDescription":"Invalid parameters",
                "errors":{"task.websiteKey":"must not be blank"}
            }"#,
        )
        .create_async()
        .await;

    let client = test_client(ClientConfig::default(), &server)?;
    let error = client
        .create_task(TaskType::Hcaptcha, &json!({}))
        .await
        .err()
        .ok_or_else(|| std::io::Error::other("expected an API error"))?;

    // Displaying the error alone must name the offending field.
    let message = error.to_string();
    assert!(message.contains("ERROR_REQUEST_PARAMETERS"), "{message}");
    assert!(message.contains("task.websiteKey"), "{message}");
    assert!(message.contains("must not be blank"), "{message}");
    create_mock.assert_async().await;
    Ok(())
}

#[tokio::test]
async fn an_empty_error_code_does_not_turn_success_into_failure()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut server = Server::new_async().await;
    let create_mock = server
        .mock("POST", "/createTask")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"errorId":0,"errorCode":"","errorDescription":"","taskId":"task-ok"}"#)
        .create_async()
        .await;

    let client = test_client(ClientConfig::default(), &server)?;
    let created = client.create_task(TaskType::Hcaptcha, &json!({})).await?;

    assert_eq!(created.task_id, "task-ok");
    create_mock.assert_async().await;
    Ok(())
}

#[tokio::test]
async fn specialized_solver_preserves_task_api_error_details()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut server = Server::new_async().await;
    let create_mock = server
        .mock("POST", "/createTask")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"errorId":0,"requestId":"request-create","taskId":"task-123"}"#)
        .create_async()
        .await;
    let result_mock = server
        .mock("POST", "/getTaskResult")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "errorId":1,
                "status":"error",
                "errorCode":"ERROR_CAPTCHA_UNSOLVABLE",
                "errorDescription":"Captcha could not be solved",
                "requestId":"request-3"
            }"#,
        )
        .create_async()
        .await;

    let config =
        ClientConfig::default().with_polling(PollingConfig::new(Duration::from_millis(1), 2));
    let client = test_client(config, &server)?;
    let task = RecaptchaV2Task {
        website_url: "https://example.com".to_owned(),
        website_key: "site-key".to_owned(),
        ..Default::default()
    };
    let error = client
        .solve_recaptcha_v2_task_proxyless(&task)
        .await
        .unwrap_err();

    let Error::EzError(api_error) = error else {
        return Err(std::io::Error::other("expected a structured API error").into());
    };
    assert_eq!(
        api_error.error_code.as_deref(),
        Some("ERROR_CAPTCHA_UNSOLVABLE")
    );
    assert_eq!(
        api_error.error_description.as_deref(),
        Some("Captcha could not be solved")
    );
    assert_eq!(api_error.task_id.as_deref(), Some("task-123"));
    assert_eq!(api_error.request_id.as_deref(), Some("request-create"));
    create_mock.assert_async().await;
    result_mock.assert_async().await;
    Ok(())
}

#[tokio::test]
async fn api_errors_are_preserved_for_non_success_http_statuses()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut server = Server::new_async().await;
    let balance_mock = server
        .mock("POST", "/getBalance")
        .match_body(Matcher::Json(json!({"clientKey": "client-key"})))
        .with_status(429)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "errorId":1,
                "errorCode":"ERROR_RATE_LIMIT",
                "errorDescription":"Too many requests",
                "requestId":"request-2"
            }"#,
        )
        .create_async()
        .await;

    let client = test_client(ClientConfig::default(), &server)?;
    let error = client
        .balance()
        .await
        .err()
        .ok_or_else(|| std::io::Error::other("expected an API error"))?;

    let Error::EzError(api_error) = error else {
        return Err(std::io::Error::other("expected a structured API error").into());
    };
    assert_eq!(api_error.error_code.as_deref(), Some("ERROR_RATE_LIMIT"));
    assert_eq!(
        api_error.error_description.as_deref(),
        Some("Too many requests")
    );
    assert_eq!(api_error.task_id, None);
    assert_eq!(api_error.request_id.as_deref(), Some("request-2"));
    assert_eq!(api_error.http_status, 429);
    balance_mock.assert_async().await;
    Ok(())
}

#[tokio::test]
async fn polling_stops_after_the_configured_attempt_limit()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut server = Server::new_async().await;
    let result_mock = server
        .mock("POST", "/getTaskResult")
        .match_body(Matcher::Json(json!({
            "clientKey": "client-key",
            "taskId": "task-123"
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"errorId":0,"status":"processing"}"#)
        .expect(2)
        .create_async()
        .await;

    let client = test_client(ClientConfig::default(), &server)?;
    let error = client
        .wait_for_result_with("task-123", PollingConfig::new(Duration::from_millis(1), 2))
        .await
        .err()
        .ok_or_else(|| std::io::Error::other("expected polling to be exhausted"))?;

    let Error::PollingExhausted {
        task_id, attempts, ..
    } = error
    else {
        return Err(std::io::Error::other("expected a polling exhaustion error").into());
    };
    assert_eq!(task_id, "task-123");
    assert_eq!(attempts, 2);
    result_mock.assert_async().await;
    Ok(())
}

/// A throttled poll refuses the query, not the task: the task is still queued
/// and has already been billed, so failing the wait would throw away a result
/// that was about to arrive.
#[tokio::test]
async fn a_throttled_poll_is_retried_instead_of_failing_the_task()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut server = Server::new_async().await;
    let throttled = server
        .mock("POST", "/getTaskResult")
        .with_status(429)
        .with_header("content-type", "application/json")
        .with_body(r#"{"errorId":1,"errorCode":"ERROR_REQUEST_LIMIT"}"#)
        .expect(1)
        .create_async()
        .await;
    let ready = server
        .mock("POST", "/getTaskResult")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"errorId":0,"status":"ready","solution":{"token":"t"}}"#)
        .expect(1)
        .create_async()
        .await;

    let client = test_client(ClientConfig::default(), &server)?;
    let result = client
        .wait_for_result_with("task-123", PollingConfig::new(Duration::from_millis(1), 3))
        .await?;

    assert_eq!(result.status, TaskStatus::Ready);
    throttled.assert_async().await;
    ready.assert_async().await;
    Ok(())
}

/// Only the throttling codes are retried. Every other API error is the answer
/// to the poll, so the wait has to end on it rather than burn the budget.
#[tokio::test]
async fn a_non_throttling_error_ends_the_wait_at_once()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut server = Server::new_async().await;
    let result_mock = server
        .mock("POST", "/getTaskResult")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(r#"{"errorId":1,"errorCode":"ERROR_TASK_NOT_EXIST"}"#)
        .expect(1)
        .create_async()
        .await;

    let client = test_client(ClientConfig::default(), &server)?;
    let error = client
        .wait_for_result_with("task-123", PollingConfig::new(Duration::from_millis(1), 5))
        .await
        .err()
        .ok_or_else(|| std::io::Error::other("expected the wait to fail"))?;

    let Error::EzError(api_error) = error else {
        return Err(std::io::Error::other("expected a structured API error").into());
    };
    assert_eq!(
        api_error.error_code.as_deref(),
        Some("ERROR_TASK_NOT_EXIST")
    );
    result_mock.assert_async().await;
    Ok(())
}

#[tokio::test]
async fn the_first_result_query_waits_for_one_polling_interval()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut server = Server::new_async().await;
    let result_mock = server
        .mock("POST", "/getTaskResult")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"errorId":0,"status":"ready","solution":{"gRecaptchaResponse":"t"}}"#)
        .create_async()
        .await;

    let client = test_client(ClientConfig::default(), &server)?;
    let polling = PollingConfig::new(Duration::from_millis(150), 1);

    let started = std::time::Instant::now();
    let result = client.wait_for_result_with("task-123", polling).await?;
    let elapsed = started.elapsed();

    assert!(result.is_ready());
    assert!(
        elapsed >= Duration::from_millis(150),
        "the first query ran after {elapsed:?}, expected it to wait one 150ms interval"
    );
    result_mock.assert_async().await;
    Ok(())
}

#[test]
fn invalid_client_configuration_is_rejected() {
    let blank_key = AsyncEzCapSolverClient::with_client_key(" ").err();
    assert!(matches!(
        blank_key,
        Some(Error::Message(message)) if message.contains("client key")
    ));

    let zero_timeout = ClientConfig::default().with_timeout(Duration::from_secs(0));
    let invalid_timeout =
        AsyncEzCapSolverClient::with_config(zero_timeout.with_client_key("client-key")).err();
    assert!(matches!(
        invalid_timeout,
        Some(Error::Message(message)) if message.contains("timeout")
    ));
}

#[test]
fn client_can_be_created_with_an_explicit_key() -> Result<()> {
    let client = AsyncEzCapSolverClient::with_client_key("client-key")?;

    assert_eq!(client.client_key, "client-key");
    Ok(())
}

/// Every endpoint carries the client key in the `X-API-Key` header, on
/// top of the `clientKey` body field. It sends no `X-Request-Id`: the
/// gateway assigns the correlation id and the SDK only reads it back.
#[tokio::test]
async fn every_request_carries_the_client_key_header()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut server = Server::new_async().await;
    let ready = r#"{"errorId":0,"status":"ready","solution":{"token":"token-value"}}"#;
    let mut mocks = Vec::new();
    for (path, body) in [
        ("/createTask", r#"{"errorId":0,"taskId":"task-123"}"#),
        ("/getTaskResult", ready),
        ("/createSyncTask", ready),
        ("/getBalance", r#"{"errorId":0,"balance":1.5}"#),
    ] {
        let mock = server
            .mock("POST", path)
            .match_header("X-API-Key", "client-key")
            .match_header("X-Request-Id", Matcher::Missing)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body)
            .expect(1)
            .create_async()
            .await;
        mocks.push(mock);
    }

    let client = test_client(
        ClientConfig::default().with_polling(PollingConfig::new(Duration::from_millis(1), 1)),
        &server,
    )?;
    client.solve("CustomTask", &json!({})).await?;
    client.sync_solve("CustomTask", &json!({})).await?;
    client.balance().await?;

    for mock in mocks {
        mock.assert_async().await;
    }
    Ok(())
}

/// A key that cannot travel as a header value fails when the client is built,
/// not on every request afterwards.
#[test]
fn a_client_key_that_is_not_a_valid_header_value_is_rejected() {
    let error = AsyncEzCapSolverClient::with_client_key("client\nkey").err();

    assert!(matches!(
        error,
        Some(Error::Message(message)) if message.contains("invalid characters")
    ));
}

fn test_client(
    config: ClientConfig,
    server: &mockito::ServerGuard,
) -> Result<AsyncEzCapSolverClient> {
    AsyncEzCapSolverClient::with_config(
        config
            .with_client_key("client-key")
            .with_base_urls(server.url(), server.url()),
    )
}

/// Supplying an HTTP client is what makes reusing a connection pool, installing
/// middleware, or changing the TLS configuration possible.
#[tokio::test]
async fn a_supplied_http_client_is_used_as_is()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/getBalance")
        .match_header("user-agent", "caller-supplied/9.9")
        // The supplied client has no default headers, so the key header must
        // still be added per request.
        .match_header("X-API-Key", "client-key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"errorId":0,"balance":12.5}"#)
        .expect(1)
        .create_async()
        .await;

    // The configured user_agent is deliberately something else: the supplied
    // client brings its own header, which must not be overwritten.
    let client = AsyncEzCapSolverClient::with_http_client(
        ClientConfig::default()
            .with_client_key("client-key")
            .with_user_agent("ignored-because-the-client-brings-its-own")
            .with_base_urls(server.url(), server.url()),
        reqwest::Client::builder()
            .user_agent("caller-supplied/9.9")
            .build()?,
    )?;

    assert_eq!(client.balance().await?, 12.5);
    mock.assert_async().await;
    Ok(())
}

/// One client instance serving many concurrent tasks.
///
/// Every method takes `&self` and the client holds no interior mutable state, so
/// sharing needs nothing beyond an `Arc`. `reqwest::Client` is itself `Arc`-backed,
/// so neither cloning nor sharing duplicates the connection pool.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn one_client_serves_many_concurrent_callers()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    const CALLERS: usize = 64;

    // A compile-time proof of everything sharing across threads requires.
    const fn assert_shareable<T: Send + Sync + Clone + 'static>() {}
    assert_shareable::<AsyncEzCapSolverClient>();
    assert_shareable::<crate::EzCapSolverClient>();

    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/getBalance")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"errorId":0,"balance":12.5}"#)
        .expect(CALLERS)
        .create_async()
        .await;

    let client = Arc::new(AsyncEzCapSolverClient::with_config(
        ClientConfig::default()
            .with_client_key("client-key")
            .with_base_urls(server.url(), server.url()),
    )?);

    let mut tasks = tokio::task::JoinSet::new();
    for _ in 0..CALLERS {
        let client = Arc::clone(&client);
        tasks.spawn(async move { client.balance().await });
    }

    let mut completed = 0;
    while let Some(joined) = tasks.join_next().await {
        assert_eq!(joined??, 12.5);
        completed += 1;
    }

    assert_eq!(completed, CALLERS);
    mock.assert_async().await;
    Ok(())
}

#[tokio::test]
async fn a_wait_failure_carries_the_identifier_of_the_billed_task()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut server = Server::new_async().await;
    let create_mock = server
        .mock("POST", "/createTask")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"errorId":0,"requestId":"request-1","taskId":"task-123"}"#)
        .create_async()
        .await;
    // A success envelope with no status violates the contract. The task has been
    // created and billed by now, and a caller using the one-shot solve has never
    // seen its id, so an error that does not carry it writes the charge off.
    let result_mock = server
        .mock("POST", "/getTaskResult")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"errorId":0}"#)
        .create_async()
        .await;

    let client = test_client(
        ClientConfig::default().with_polling(PollingConfig::new(Duration::from_millis(1), 3)),
        &server,
    )?;
    let error = client
        .solve(
            TaskType::RecaptchaV2TaskProxyless,
            &RecaptchaV2Task::default(),
        )
        .await
        .err()
        .ok_or_else(|| std::io::Error::other("expected the wait to fail"))?;

    let Error::WaitInterrupted {
        task_id,
        request_id,
        source,
    } = error
    else {
        return Err(std::io::Error::other("expected the task identifier to survive").into());
    };
    assert_eq!(task_id, "task-123");
    assert_eq!(request_id.as_deref(), Some("request-1"));
    assert!(
        matches!(*source, Error::UnexpectedResponse(_)),
        "the underlying failure must stay reachable"
    );

    create_mock.assert_async().await;
    result_mock.assert_async().await;
    Ok(())
}

#[tokio::test]
async fn an_exhausted_budget_is_not_wrapped_a_second_time()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut server = Server::new_async().await;
    let create_mock = server
        .mock("POST", "/createTask")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"errorId":0,"taskId":"task-123"}"#)
        .create_async()
        .await;
    let result_mock = server
        .mock("POST", "/getTaskResult")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"errorId":0,"status":"processing"}"#)
        .expect(2)
        .create_async()
        .await;

    let client = test_client(
        ClientConfig::default().with_polling(PollingConfig::new(Duration::from_millis(1), 2)),
        &server,
    )?;
    let error = client
        .solve(
            TaskType::RecaptchaV2TaskProxyless,
            &RecaptchaV2Task::default(),
        )
        .await
        .err()
        .ok_or_else(|| std::io::Error::other("expected polling to be exhausted"))?;

    // This variant carries a task_id already, so wrapping it would only make a
    // caller unwrap one more layer to read the same value.
    assert!(
        matches!(error, Error::PollingExhausted { ref task_id, .. } if task_id == "task-123"),
        "got {error:?}"
    );

    create_mock.assert_async().await;
    result_mock.assert_async().await;
    Ok(())
}

#[tokio::test]
async fn an_api_error_while_waiting_keeps_its_own_identifier_field()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut server = Server::new_async().await;
    let create_mock = server
        .mock("POST", "/createTask")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"errorId":0,"requestId":"request-1","taskId":"task-123"}"#)
        .create_async()
        .await;
    let result_mock = server
        .mock("POST", "/getTaskResult")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(r#"{"errorId":1,"errorCode":"ERROR_TASK_NOT_EXIST"}"#)
        .create_async()
        .await;

    let client = test_client(
        ClientConfig::default().with_polling(PollingConfig::new(Duration::from_millis(1), 3)),
        &server,
    )?;
    let error = client
        .solve(
            TaskType::RecaptchaV2TaskProxyless,
            &RecaptchaV2Task::default(),
        )
        .await
        .err()
        .ok_or_else(|| std::io::Error::other("expected an API error"))?;

    // A business error has a task_id field of its own, so it is filled in rather
    // than wrapped; wrapping would make an existing match miss.
    let Error::EzError(api_error) = error else {
        return Err(std::io::Error::other("a business error must not be wrapped").into());
    };
    assert_eq!(api_error.task_id.as_deref(), Some("task-123"));
    assert_eq!(api_error.request_id.as_deref(), Some("request-1"));

    create_mock.assert_async().await;
    result_mock.assert_async().await;
    Ok(())
}
