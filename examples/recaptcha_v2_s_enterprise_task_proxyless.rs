//! Demonstrates how to use EzCaptchaSolver to solve reCAPTCHA v2 Enterprise
//! challenges that carry the `s` parameter.
//!
//! **Task type:** [`ReCaptchaV2SEnterpriseTaskProxyless`]
//!
//! For detailed usage instructions, see the
//! [official EzCaptchaSolver documentation](https://ezxlabs.com/en/docs/captcha/api/recaptcha-v2).

use ezcapsolver::{AsyncEzCapSolverClient, RecaptchaV2Task};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = AsyncEzCapSolverClient::with_client_key(std::env::var("EZCAPTCHA_API_KEY")?)?;
    let task_payload = RecaptchaV2Task::builder()
        .website_url("https://example.com")
        .website_key("your_enterprise_key")
        // The per-session `s` value read from the widget's configuration.
        // Despite the type name, the service does not actually require it.
        .s("the_s_value_from_the_page")
        .build();

    let task = client
        .solve_recaptcha_v2_s_enterprise_task_proxyless(&task_payload)
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
