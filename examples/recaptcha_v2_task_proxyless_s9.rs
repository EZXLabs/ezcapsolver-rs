//! Demonstrates how to use EzCaptchaSolver to solve reCAPTCHA v2 challenges
//! with a minimum score of 0.9.
//!
//! **Task type:** [`ReCaptchaV2TaskProxylessS9`]
//!
//! For detailed usage instructions, see the
//! [official EzCaptchaSolver documentation](https://ezxlabs.com/en/docs/captcha/api/recaptcha-v2).

use ezcapsolver::{AsyncEzCapSolverClient, RecaptchaV2Task};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = AsyncEzCapSolverClient::with_client_key(std::env::var("EZCAPTCHA_API_KEY")?)?;

    // Same parameters as the plain V2 type; the difference is the queue.
    let task_payload = RecaptchaV2Task::builder()
        .website_url("https://example.com")
        .website_key("your_site_key")
        .build();

    let task = client
        .solve_recaptcha_v2_task_proxyless_s9(&task_payload)
        .await?;

    println!("Trace ID:   {}", task.request_id.as_deref().unwrap_or("-"));
    println!("Task ID:    {}", task.task_id.as_deref().unwrap_or("-"));
    println!("User-Agent: {}", task.solution.user_agent);
    println!(
        "Token:      {}...",
        task.solution.token.chars().take(64).collect::<String>()
    );

    let balance = client.balance().await?;
    println!("Balance:    {}", balance);
    Ok(())
}
