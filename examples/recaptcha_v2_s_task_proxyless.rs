//! Demonstrates how to use EzCaptchaSolver to solve a reCAPTCHA V2 task carrying the
//! challenge-bound `s` parameter.
//!
//! **Task type:** [`ReCaptchaV2STaskProxyless`]
//!
//! Supplying `s` routes the task to the high-score IPv4 queue. The service does
//! not actually require it, so this type behaves like the plain V2 one when the
//! parameter is left unset.
//!
//! For detailed usage instructions, see the
//! [official EzCaptchaSolver documentation](https://ezxlabs.com/en/docs/captcha/api/recaptcha-v2).

use ezcapsolver::{AsyncEzCapSolverClient, RecaptchaV2Task};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = AsyncEzCapSolverClient::with_client_key(std::env::var("EZCAPTCHA_API_KEY")?)?;

    let task_payload = RecaptchaV2Task::builder()
        .website_url("https://example.com")
        .website_key("6Lc_your_site_key")
        // Read off the page that issued the challenge.
        .s("value-from-the-page")
        .build();

    let solved = client
        .solve_recaptcha_v2_s_task_proxyless(&task_payload)
        .await?;

    println!("Task id:    {}", solved.task_id.as_deref().unwrap_or("-"));
    println!("Token:      {}", solved.solution.token);

    println!("Balance:    {}", client.balance().await?);
    Ok(())
}
