//! Demonstrates how to use EzCaptchaSolver to solve reCAPTCHA v3 Enterprise
//! challenges.
//!
//! **Task type:** [`ReCaptchaV3EnterpriseTaskProxyless`]
//!
//! For detailed usage instructions, see the
//! [official EzCaptchaSolver documentation](https://docs.ezxlabs.com/docs/captcha/api/recaptcha-v3).

use ezcapsolver::{AsyncEzCapSolverClient, RecaptchaV3Task};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = AsyncEzCapSolverClient::with_client_key(std::env::var("EZCAPTCHA_API_KEY")?)?;

    let task_payload = RecaptchaV3Task::builder()
        .website_url("https://example.com")
        .website_key("your_enterprise_key")
        // Must match the action the page passes to `grecaptcha.execute`.
        .page_action("checkout")
        // `is_invisible` is left alone: V3 defaults it to true.
        .build();

    let task = client
        .solve_recaptcha_v3_enterprise_task_proxyless(&task_payload)
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
