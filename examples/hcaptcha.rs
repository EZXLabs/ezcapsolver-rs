//! This example demonstrates how to use EzCaptchaSolver to solve HCaptcha challenges.
//!
//! **Task type:** [`HCaptcha`]
//!
//! For detailed usage instructions, see the
//! [official EzCaptchaSolver documentation](https://docs.ezxlabs.com/docs/captcha/api/hcaptcha).

use ezcapsolver::{AsyncEzCapSolverClient, HcaptchaTask};
use serde_json::Value;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = AsyncEzCapSolverClient::with_client_key(std::env::var("EZCAPTCHA_API_KEY")?)?;

    let task_payload = HcaptchaTask::builder()
        .website_url("https://accounts.hcaptcha.com/demo")
        .website_key("a5f74b19-9e45-40e0-b45d-47ff91b7a6c2")
        .lang("en-US")
        // False because the demo page shows a checkbox. Sites that hide it
        // need true here.
        .invisible(false)
        .maybe_proxy(std::env::var("EZCAPTCHA_PROXY").ok())
        // Only the sites that publish an rqdata value need this one.
        .maybe_rq_data(std::env::var("EZCAPTCHA_HCAPTCHA_RQDATA").ok())
        .build();

    let task = client.solve_hcaptcha(&task_payload).await?;

    println!("Trace ID:   {}", task.request_id.as_deref().unwrap_or("-"));
    println!("Task ID:    {}", task.task_id.as_deref().unwrap_or("-"));
    println!("Pass UUID:  {}", task.solution.generated_pass_uuid);
    // Every request carrying the token must send this exact user agent.
    println!("User-Agent: {}", task.solution.ua.as_deref().unwrap_or("-"));
    if let Some(context_id) = task.solution.extra.get("contextId").and_then(Value::as_str) {
        println!("Context ID: {context_id}");
    }

    let balance = client.balance().await?;
    println!("Balance:    {}", balance);
    Ok(())
}
