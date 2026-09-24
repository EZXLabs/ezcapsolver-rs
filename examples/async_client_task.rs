//! Demonstrates how to build the asynchronous EzCaptchaSolver client in full,
//! then solve one task with it.
//!
//! Every setting below is optional — `AsyncEzCapSolverClient::new()` reads
//! `EZCAPTCHA_API_KEY` and takes the defaults for the rest. They are all spelled
//! out here so the knobs are visible in one place.
//!
//! For the task types themselves, see the per-type examples in this directory.

use std::time::Duration;

use ezcapsolver::{AsyncEzCapSolverClient, ClientConfig, PollingConfig, RecaptchaV2Task};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = ClientConfig::builder()
        // Without this the client falls back to `EZCAPTCHA_API_KEY`.
        .client_key(std::env::var("EZCAPTCHA_API_KEY")?)
        .user_agent("my-app/1.0")
        // Two separate budgets, and they have to stay separate. Asynchronous
        // calls only enqueue and poll, so 30 seconds is already generous.
        // Synchronous calls block until a worker answers, and the service
        // grants some task types three minutes — sharing the shorter budget
        // would abort a call that has already been billed, and a call that
        // times out never receives the task ID
        // to recover the result with.
        .timeout(Duration::from_secs(30))
        .sync_timeout(Duration::from_mins(3))
        // How long to wait for an asynchronous task: 3s x 50 is two and a half minutes.
        .polling(PollingConfig::new(Duration::from_secs(3), 50))
        // The SDK's own egress proxy, unrelated to a task's `proxy` parameter,
        // which is what the worker uses to reach the protected site.
        .maybe_proxy(std::env::var("EZCAPTCHA_PROXY").ok())
        .build();

    let client = AsyncEzCapSolverClient::with_config(config)?;

    let task_payload = RecaptchaV2Task::builder()
        .website_url("https://www.google.com/recaptcha/api2/demo")
        .website_key("6Le-wvkSAAAAAPBMRTvw0Q4Muexq9bi0DJwx_mJ-")
        .build();

    let task = client
        .solve_recaptcha_v2_task_proxyless(&task_payload)
        .await?;

    println!("Trace ID:   {}", task.request_id.as_deref().unwrap_or("-"));
    println!("Task ID:    {}", task.task_id.as_deref().unwrap_or("-"));
    println!("User-Agent: {}", task.solution.user_agent);
    println!(
        "Token:      {}...",
        task.solution.token.chars().take(64).collect::<String>()
    );

    let balance = client.balance().await?;
    println!("Balance:    {balance}");
    Ok(())
}
