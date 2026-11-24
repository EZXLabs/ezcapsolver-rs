//! The DataDome challenge workflow end to end, against one local server standing in
//! for the protected site, DataDome, and the EzCaptchaSolver API.
#![cfg(feature = "datadome")]

use base64::{Engine as _, engine::general_purpose::STANDARD};
use ezcapsolver::{
    AsyncEzCapSolverClient, ClientConfig, Error,
    ext::{DataDomeChallenge, DataDomeCookie, DataDomeOutcome},
};
use mockito::{Matcher, Mock, Server, ServerGuard};
use serde_json::{Value, json};

type TestResult = Result<(), Box<dyn std::error::Error>>;

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/153.0.0.0 Safari/537.36";
const INTERCEPTED_PAGE: &str = "<html><script>var dd={'rt':'c','cid':'abc'}</script></html>";
const SLIDER_SCRIPT: &str = "/** DataDome v1.34.0 96eb15b */\n!function A(){}();\nvar g=1;";
const INTERSTITIAL_SCRIPT: &str = "/* DataDome v1.33.1 d5110df */\n;(function(){var A={};})()";
const RAW_COOKIE: &str =
    "datadome=abc123; Max-Age=31536000; Domain=.example.com; Path=/; Secure; SameSite=Lax";

/// Requires the flow's future to be `Send`, so callers can spawn it.
fn assert_send<T: Send>(future: T) -> T {
    future
}

/// A client whose API calls go to the local server.
fn client(server: &ServerGuard) -> ezcapsolver::Result<AsyncEzCapSolverClient> {
    AsyncEzCapSolverClient::with_config(
        ClientConfig::default()
            .with_client_key("client-key")
            .with_base_urls(server.url(), server.url()),
    )
}

/// A challenge run against the local server's `/signup` page.
fn challenge(server: &ServerGuard) -> DataDomeChallenge {
    DataDomeChallenge::builder()
        .target_url(format!("{}/signup", server.url()))
        .user_agent(USER_AGENT)
        .build()
}

/// Answers one DataDome step on the synchronous endpoint.
async fn api_step(server: &mut ServerGuard, task: Matcher, solution: Value) -> Mock {
    server
        .mock("POST", "/createSyncTask")
        .match_body(task)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(json!({"errorId": 0, "status": "ready", "solution": solution}).to_string())
        .expect(1)
        .create_async()
        .await
}

/// Matches a DataDome task with the given step and fields.
fn task(fields: Value) -> Matcher {
    let mut task = json!({"type": "DataDomeTaskProxyless"});
    if let (Some(task), Some(fields)) = (task.as_object_mut(), fields.as_object()) {
        task.extend(fields.clone());
    }
    Matcher::PartialJson(json!({"task": task}))
}

/// Serves the intercepted target page.
async fn intercepted_target(server: &mut ServerGuard) -> Mock {
    server
        .mock("GET", "/signup")
        .match_header("user-agent", USER_AGENT)
        .with_status(403)
        .with_body(INTERCEPTED_PAGE)
        .expect(1)
        .create_async()
        .await
}

/// Answers a submission with the cookie.
fn cookie_verdict() -> String {
    json!({"view": "redirect", "cookie": RAW_COOKIE}).to_string()
}

/// The cookie every passing run returns.
fn expected_cookie() -> DataDomeCookie {
    DataDomeCookie {
        name: "datadome".to_owned(),
        value: "abc123".to_owned(),
        domain: ".example.com".to_owned(),
        raw: RAW_COOKIE.to_owned(),
    }
}

#[tokio::test]
async fn a_slider_run_sends_an_uncached_script_in_full() -> TestResult {
    let mut server = Server::new_async().await;
    let base = server.url();
    let target = intercepted_target(&mut server).await;
    let step_one = api_step(
        &mut server,
        task(json!({
            "step": "1",
            "html_b64": STANDARD.encode(INTERCEPTED_PAGE),
            "referer": format!("{base}/signup"),
            "user_agent": USER_AGENT
        })),
        json!({"url": format!("{base}/captcha/?initialCid=abc")}),
    )
    .await;
    let challenge_page = format!("<html><script>var ddm={{}};\n{SLIDER_SCRIPT}</script></html>");
    let frame = server
        .mock("GET", "/captcha/")
        .match_query(Matcher::Any)
        .match_header("referer", format!("{base}/signup").as_str())
        .with_body(&challenge_page)
        .expect(1)
        .create_async()
        .await;
    let step_two = api_step(
        &mut server,
        task(json!({"step": "2"})),
        json!({"status": false}),
    )
    .await;
    let step_three = api_step(
        &mut server,
        task(json!({
            "step": "3",
            "html_b64": STANDARD.encode(&challenge_page),
            "referer": format!("{base}/captcha/?initialCid=abc"),
            "external_js_base64": STANDARD.encode(SLIDER_SCRIPT)
        })),
        json!({
            "url": format!("{base}/captcha/check?cid=abc"),
            "kind": "slider",
            "submit_after_ms": 1
        }),
    )
    .await;
    let submission = server
        .mock("GET", "/captcha/check")
        .match_query(Matcher::Any)
        .with_body(cookie_verdict())
        .expect(1)
        .create_async()
        .await;

    let client = client(&server)?;
    let outcome = assert_send(client.solve_data_dome_workflow(&challenge(&server))).await?;

    assert_eq!(outcome, DataDomeOutcome::Passed(expected_cookie()));
    for mock in [target, step_one, frame, step_two, step_three, submission] {
        mock.assert_async().await;
    }
    Ok(())
}

#[tokio::test]
async fn an_interstitial_run_sends_only_the_hash_of_a_cached_script() -> TestResult {
    let mut server = Server::new_async().await;
    let base = server.url();
    let target = intercepted_target(&mut server).await;
    let step_one = api_step(
        &mut server,
        task(json!({"step": "1"})),
        json!({"url": format!("{base}/interstitial/?initialCid=abc")}),
    )
    .await;
    let challenge_page = format!(
        "<html><script>var ddm={{}};\n{INTERSTITIAL_SCRIPT};\nvar glue='cid=abc';</script></html>"
    );
    let frame = server
        .mock("GET", "/interstitial/")
        .match_query(Matcher::Any)
        // The interstitial frame only gets the origin of the page.
        .match_header("referer", base.as_str())
        .with_body(&challenge_page)
        .expect(1)
        .create_async()
        .await;
    let step_two = api_step(
        &mut server,
        task(json!({"step": "2"})),
        json!({"status": true}),
    )
    .await;
    let step_three = api_step(
        &mut server,
        Matcher::AllOf(vec![
            task(json!({"step": "3"})),
            Matcher::Regex(r#""external_js_hash":"[0-9a-f]{32}""#.to_owned()),
        ]),
        json!({"url": format!("{base}/interstitial/check"), "body": "payload=1"}),
    )
    .await;
    let submission = server
        .mock("POST", "/interstitial/check")
        .match_header(
            "content-type",
            "application/x-www-form-urlencoded; charset=UTF-8",
        )
        .match_body("payload=1")
        .with_body(cookie_verdict())
        .expect(1)
        .create_async()
        .await;

    let outcome = client(&server)?
        .solve_data_dome_workflow(&challenge(&server))
        .await?;

    assert_eq!(outcome, DataDomeOutcome::Passed(expected_cookie()));
    for mock in [target, step_one, frame, step_two, step_three, submission] {
        mock.assert_async().await;
    }
    Ok(())
}

#[tokio::test]
async fn a_page_datadome_lets_through_costs_no_task() -> TestResult {
    let mut server = Server::new_async().await;
    let target = server
        .mock("GET", "/signup")
        .with_body("<html>welcome</html>")
        .expect(1)
        .create_async()
        .await;
    let api = server
        .mock("POST", "/createSyncTask")
        .expect(0)
        .create_async()
        .await;

    let outcome = client(&server)?
        .solve_data_dome_workflow(&challenge(&server))
        .await?;

    assert_eq!(outcome, DataDomeOutcome::NotTriggered);
    target.assert_async().await;
    api.assert_async().await;
    Ok(())
}

#[tokio::test]
async fn a_block_page_is_reported_as_a_ban() -> TestResult {
    let mut server = Server::new_async().await;
    let _target = server
        .mock("GET", "/signup")
        .with_status(403)
        .with_body("<html><script>var dd={'t':'bv'}</script>hardblock</html>")
        .create_async()
        .await;
    let api = server
        .mock("POST", "/createSyncTask")
        .expect(0)
        .create_async()
        .await;

    let result = client(&server)?
        .solve_data_dome_workflow(&challenge(&server))
        .await;

    assert!(
        matches!(result, Err(Error::DataDomeBlocked(_))),
        "{result:?}"
    );
    api.assert_async().await;
    Ok(())
}
