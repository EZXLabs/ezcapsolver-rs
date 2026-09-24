//! Demonstrates how to turn on SDK logging and handle every error it can raise.
//!
//! The SDK emits through `tracing`; the application picks the subscriber and the
//! filter. Nothing is rendered until one is installed.
//!
//! | Level | Events |
//! | --- | --- |
//! | `INFO` | Task created, task solved |
//! | `DEBUG` | The request lifecycle and every polling attempt |
//! | `TRACE` | One line per request and per response, with the body truncated |
//!
//! Request bodies are rendered with `clientKey` and `proxy` replaced by
//! `[REDACTED]` at any nesting depth, and are not rendered at all below `TRACE`.

use ezcapsolver::{AsyncEzCapSolverClient, Error, RecaptchaV2Task};
use tracing::Level;
use tracing_subscriber::{filter::Targets, prelude::*};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Scope the level to this crate: a global DEBUG also turns on the HTTP
    // stack's own output and buries these lines.
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(Targets::new().with_target("ezcapsolver", Level::DEBUG))
        .init();

    let client = AsyncEzCapSolverClient::with_client_key(std::env::var("EZCAPTCHA_API_KEY")?)?;
    let task = RecaptchaV2Task::builder()
        .website_url("https://www.google.com/recaptcha/api2/demo")
        .website_key("6Le-wvkSAAAAAPBMRTvw0Q4Muexq9bi0DJwx_mJ-")
        .build();

    let outcome = client.solve_recaptcha_v2_task_proxyless(&task).await;

    // Ask this first. Whether a billed task is still recoverable matters more
    // than what broke, because it decides whether you pay again — and it is one
    // question regardless of which variant happens to carry the id.
    if let Err(error) = &outcome
        && let Some(task_id) = error.task_id()
    {
        println!("Recover:    wait_for_result({task_id}) — do not create a second task");
    }

    match outcome {
        Ok(solved) => println!("Token:      {}", solved.solution.token),

        // The service refused the request. Code, description and HTTP status
        // are all on the error; the two helpers say what to do about it.
        Err(Error::EzError(api)) => {
            println!("API error:  {api}");
            if api.is_authentication_error() {
                // The service counts these per key and bans after thirty in a
                // minute. Stop rather than back off.
                println!("Action:     stop, do not retry");
            } else if api.is_terminal() {
                println!("Action:     fix the request, retrying changes nothing");
            } else if api.is_rate_limited() {
                // Throttling refuses the query, not the task. wait_for_result
                // already polls through one of these on its own.
                println!("Action:     throttled, ask again later");
            } else {
                println!("Action:     may be transient, retrying is the caller's call");
            }
        }

        // The budget ran out, but the task may still finish. The service holds
        // its result for five minutes after creation.
        Err(Error::PollingExhausted { attempts, .. }) => {
            println!("Timed out:  gave up after {attempts} attempts");
        }

        // Same situation, different cause: the task was created and billed, but
        // the wait broke off — a dropped connection, a gateway, a response that
        // did not match the contract. The underlying failure stays reachable,
        // which is what tells a timeout apart from a DNS failure.
        Err(Error::WaitInterrupted { source, .. }) => {
            println!("Interrupted: {source}");
        }

        // The worker returned a shape this release does not model. The raw value
        // travels with the error, so nothing is lost.
        Err(Error::SolutionDecode { raw, .. }) => println!("Bad shape:  {raw}"),

        // Network-level failure: DNS, TCP, TLS, or a timeout.
        Err(Error::Request(error)) => println!("Network:    {error}"),

        Err(other) => println!("Other:      {other}"),
    }

    Ok(())
}
