//! This example demonstrates how to use EzCaptchaSolver to solve Akamai Web challenges.
//!
//! **Task type:** [`AkamaiWEBTaskProxyless`]
//!
//! Akamai Web is a multi-round protocol: each solved payload is posted back to
//! `v3Url`, and the fresh `_abck` cookie from that response feeds the next
//! round. Clearing the challenge takes up to eight rounds.
//!
//! For detailed usage instructions, see the
//! [official EzCaptchaSolver documentation](https://ezxlabs.com/en/docs/captcha/api/akamai).

use ezcapsolver::{AkamaiWebTask, AsyncEzCapSolverClient};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = AsyncEzCapSolverClient::with_client_key(std::env::var("EZCAPTCHA_API_KEY")?)?;

    let task_payload = AkamaiWebTask::builder()
        .page_url("https://example.com")
        // Most sites change this URL on every request, so read it from the page
        // rather than hard-coding it.
        .v3_url("https://example.com/v3/...")
        // Chrome only. `lang` must match both the accept-language header sent
        // to the site and the region the proxy exits from.
        .ua("Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/131.0.0.0")
        .lang("en-GB")
        // The first round is index 0 and leaves `encode_data` unset.
        .index(0)
        .abck("_abck_cookie_value")
        .bmsz("bm_sz_cookie_value")
        // Read from network traffic, then base64-encoded. Only the first round
        // sends it; later rounds send an empty string.
        .script_base64("base64_encoded_v3_script")
        .build();

    let solution = client
        .sync_solve_akamai_web_task_proxyless(&task_payload)
        .await?
        .solution;

    println!(
        "Payload:    {}...",
        solution.payload.chars().take(64).collect::<String>()
    );
    println!(
        "Encodedata: {}...",
        solution.encodedata.chars().take(64).collect::<String>()
    );

    // Next round: POST `payload` to `v3Url`, read the new `_abck` cookie from
    // that response, then solve again with `index` incremented, `abck` set to
    // the new cookie, `encode_data` set to `solution.encodedata`, and an empty
    // `script_base64`. Repeat until `_abck` is accepted.

    let balance = client.balance().await?;
    println!("Balance:    {}", balance);
    Ok(())
}
