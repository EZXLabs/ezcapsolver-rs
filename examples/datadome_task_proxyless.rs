//! Demonstrates how to use EzCaptchaSolver to clear a DataDome challenge step by step.
//!
//! **Task type:** [`DataDomeTaskProxyless`]
//!
//! The three steps share one request and one solution model, and all run
//! through the synchronous endpoint. Between them you fetch the pages yourself:
//!
//! 1. Step one turns the intercepted page into the challenge address in `url`.
//! 2. Fetch that page and its challenge script, then ask step two whether the
//!    service already caches the script, by its hash.
//! 3. Step three returns the submission in `url` and `body`, and how long to
//!    wait before sending it in `submit_after_ms`. Submitting it from the
//!    challenge page brings back the cookie.
//!
//! `referer` is what the service checks a site allow-list against, so leave it
//! set even though the parameter is not formally required.
//!
//! For detailed usage instructions, see the
//! [official EzCaptchaSolver documentation](https://docs.ezxlabs.com/docs/captcha/api).

use ezcapsolver::{AsyncEzCapSolverClient, DataDomeStep, DataDomeTask};

/// The browser the pages were fetched with.
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
                          (KHTML, like Gecko) Chrome/153.0.0.0 Safari/537.36";
/// The page DataDome intercepted, captured and base64-encoded.
const INTERCEPTED_HTML_B64: &str = "base64_encoded_intercepted_page";
/// The challenge page step one points at, captured and base64-encoded.
const CHALLENGE_HTML_B64: &str = "base64_encoded_challenge_page";
/// MD5 of the challenge script without its first-line version banner.
const SCRIPT_HASH: &str = "md5_of_the_challenge_script";
/// The challenge script itself, base64-encoded.
const SCRIPT_B64: &str = "base64_encoded_challenge_script";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = AsyncEzCapSolverClient::with_client_key("YOUR_API_KEY")?;

    // Step one: find out where the challenge lives.
    let first = client
        .sync_solve_data_dome_task_proxyless(
            &DataDomeTask::builder()
                .step(DataDomeStep::One)
                .html_b64(INTERCEPTED_HTML_B64)
                .referer("https://example.com")
                .user_agent(USER_AGENT)
                .build(),
        )
        .await?
        .solution;
    let challenge_url = first.url.unwrap_or_default();
    println!("Challenge:  {challenge_url}");

    // Fetch that page and its script, then ask whether the service caches it.
    let cached = client
        .sync_solve_data_dome_task_proxyless(
            &DataDomeTask::builder()
                .step(DataDomeStep::Two)
                .hash(SCRIPT_HASH)
                .build(),
        )
        .await?
        .solution
        .status
        .unwrap_or(false);
    println!("Cached:     {cached}");

    // Step three: the submission. A cached script only needs its hash.
    let (script_hash, script_b64) = if cached {
        (Some(SCRIPT_HASH), None)
    } else {
        (None, Some(SCRIPT_B64))
    };
    let third = client
        .sync_solve_data_dome_task_proxyless(
            &DataDomeTask::builder()
                .step(DataDomeStep::Three)
                .html_b64(CHALLENGE_HTML_B64)
                .referer(challenge_url)
                .user_agent(USER_AGENT)
                .maybe_external_js_hash(script_hash)
                .maybe_external_js_base64(script_b64)
                .build(),
        )
        .await?
        .solution;

    println!("Kind:       {}", third.kind.as_deref().unwrap_or("-"));
    println!("Submit to:  {}", third.url.as_deref().unwrap_or("-"));
    println!("Body:       {}", third.body.as_deref().unwrap_or("-"));
    println!("Wait (ms):  {}", third.submit_after_ms.unwrap_or_default());

    println!("Balance:    {}", client.balance().await?);
    Ok(())
}
