//! Demonstrates how to use EzCaptchaSolver to solve a reCAPTCHA V3 Enterprise task on
//! the high-score queue.
//!
//! **Task type:** [`ReCaptchaV3EnterpriseTaskProxylessS9`]
//!
//! Note the spelling: this is the one type in the family whose wire name opens
//! with `Recaptcha` rather than `ReCaptcha`. The SDK sends the service's own
//! spelling rather than "correcting" it.
//!
//! For detailed usage instructions, see the
//! [official EzCaptchaSolver documentation](https://docs.ezxlabs.com/docs/captcha/api/recaptcha-v3).

use ezcapsolver::{AsyncEzCapSolverClient, RecaptchaV3Task};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = AsyncEzCapSolverClient::with_client_key(std::env::var("EZCAPTCHA_API_KEY")?)?;

    let task_payload = RecaptchaV3Task::builder()
        .website_url("https://example.com")
        .website_key("6Lc_your_site_key")
        // Has to match the action the page passes to grecaptcha.execute,
        // otherwise the site-side check fails.
        .page_action("login")
        .build();

    let solved = client
        .solve_recaptcha_v3_enterprise_task_proxyless_s9(&task_payload)
        .await?;

    println!("Task id:    {}", solved.task_id.as_deref().unwrap_or("-"));
    println!("Token:      {}", solved.solution.token);

    println!("Balance:    {}", client.balance().await?);
    Ok(())
}
