//! This example demonstrates how to use EzCaptchaSolver to quickly solve the Cloudflare 5-second challenge.
//!
//! **Task type:** [`CloudFlare5STask`]
//!
//! For detailed usage instructions, see the
//! [official EzCaptchaSolver documentation](https://ezxlabs.com/en/docs/captcha/api/cloudflare-5s).

use ezcapsolver::{AsyncEzCapSolverClient, Cloudflare5sTask};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = AsyncEzCapSolverClient::with_client_key(std::env::var("EZCAPTCHA_API_KEY")?)?;

    // A proxy is mandatory for this type: the clearance is bound to the IP that
    // obtained it, so it has to be the same IP you then browse from.
    let task_payload = Cloudflare5sTask::builder()
        .website_url("https://example.com")
        .proxy(std::env::var("EZCAPTCHA_PROXY")?)
        .build();

    let task = client.solve_cloudflare_5s_task(&task_payload).await?;

    println!("Trace ID:   {}", task.request_id.as_deref().unwrap_or("-"));
    println!("Task ID:    {}", task.task_id.as_deref().unwrap_or("-"));
    println!("TLS:        {}", task.solution.tls_version);
    for (name, value) in &task.solution.header {
        println!("Header:     {name}: {value}");
    }
    for (name, value) in &task.solution.cookies {
        println!("Cookie:     {name}={value}");
    }

    let balance = client.balance().await?;
    println!("Balance:    {}", balance);
    Ok(())
}
