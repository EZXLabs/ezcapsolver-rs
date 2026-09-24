//! Demonstrates how to use EzCaptchaSolver to clear a DataDome challenge, both steps.
//!
//! **Task type:** [`DataDomeTaskProxyless`]
//!
//! The challenge takes two calls that share one solution model: step one returns
//! the challenge address in `url`, step two the validation instructions. Both run
//! through the synchronous endpoint.
//!
//! `referer` is what the service checks a site allow-list against, so leave it
//! set even though the parameter is not formally required.
//!
//! For detailed usage instructions, see the
//! [official EzCaptchaSolver documentation](https://ezxlabs.com/en/docs/captcha/api).

use ezcapsolver::{AsyncEzCapSolverClient, DataDomeStep, DataDomeTask};

/// The interstitial or slider page, captured and base64-encoded.
const CHALLENGE_HTML_B64: &str = "base64_encoded_challenge_page";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = AsyncEzCapSolverClient::with_client_key(std::env::var("EZCAPTCHA_API_KEY")?)?;

    // Step one: find out where the challenge lives.
    let first = client
        .sync_solve_data_dome_task_proxyless(
            &DataDomeTask::builder()
                .html_b64(CHALLENGE_HTML_B64)
                .step(DataDomeStep::One)
                .referer("https://example.com")
                .build(),
        )
        .await?
        .solution;

    println!("Kind:       {}", first.kind.as_deref().unwrap_or("-"));
    println!("Challenge:  {}", first.url.as_deref().unwrap_or("-"));

    // Fetch that page, capture the slider image, then ask for the validation
    // instructions.
    let second = client
        .sync_solve_data_dome_task_proxyless(
            &DataDomeTask::builder()
                .html_b64(CHALLENGE_HTML_B64)
                .step(DataDomeStep::Two)
                .image("base64_encoded_slider_image")
                .maybe_referer(first.url)
                .build(),
        )
        .await?
        .solution;

    println!("Validate:   {}", second.url.as_deref().unwrap_or("-"));
    println!("Body:       {}", second.body.as_deref().unwrap_or("-"));

    println!("Balance:    {}", client.balance().await?);
    Ok(())
}
