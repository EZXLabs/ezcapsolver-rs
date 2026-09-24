//! This example demonstrates how to use EzCaptchaSolver to quickly solve the Cloudflare turnstile challenge.
//!
//! **Task type:** [`CloudFlareTurnstileTask`]
//!
//! For detailed usage instructions, see the
//! [official EzCaptchaSolver documentation](https://ezxlabs.com/en/docs/captcha/api/turnstile).

use ezcapsolver::{AsyncEzCapSolverClient, CloudflareTurnstileTask};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = AsyncEzCapSolverClient::with_client_key(std::env::var("EZCAPTCHA_API_KEY")?)?;

    // `rq_data` is left out here. When a site uses it, send it as an object:
    // the service stringifies it itself on the way to the worker, so encoding
    // it yourself sends it twice over.
    let task_payload = CloudflareTurnstileTask::builder()
        .website_url("https://example.com")
        .website_key("0x4AAAAAAA...")
        .maybe_proxy(std::env::var("EZCAPTCHA_PROXY").ok())
        .build();

    let task = client
        .solve_cloudflare_turnstile_task(&task_payload)
        .await?;

    println!("Trace ID:   {}", task.request_id.as_deref().unwrap_or("-"));
    println!("Task ID:    {}", task.task_id.as_deref().unwrap_or("-"));
    println!(
        "Token:      {}...",
        task.solution.token.chars().take(64).collect::<String>()
    );
    for (name, value) in &task.solution.header {
        println!("Header:     {name}: {value}");
    }

    let balance = client.balance().await?;
    println!("Balance:    {}", balance);
    Ok(())
}
