//! This example demonstrates how to use EzCaptchaSolver to solve Akamai SBSD challenges.
//!
//! **Task type:** [`AkamaiSBSDTaskProxyless`]
//!
//! SBSD runs through the synchronous API, so the solver returns the solution
//! directly instead of polling for it.
//!
//! For detailed usage instructions, see the
//! [official EzCaptchaSolver documentation](https://ezxlabs.com/en/docs/captcha/api/akamai-sbsd).

use ezcapsolver::{AkamaiSbsdTask, AsyncEzCapSolverClient};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = AsyncEzCapSolverClient::with_client_key(std::env::var("EZCAPTCHA_API_KEY")?)?;

    // Every value below is collected from the protected page before solving:
    // `sbsd_url` comes from its `<script src="/sbsd/...?v=...">` tag,
    // `script_base64` is that script base64-encoded, and `bm_so` is the
    // `bm_so` cookie, falling back to `sbsd_o`.
    let task_payload = AkamaiSbsdTask::builder()
        .page_url("https://example.com")
        .sbsd_url("https://example.com/sbsd/xxxxx?v=xxx")
        .bm_so("bm_so_cookie_value")
        .ua(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
             (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
        )
        .lang("en-US")
        .script_base64("base64_encoded_sbsd_script")
        .build();

    let solution = client
        .sync_solve_akamai_sbsd_task_proxyless(&task_payload)
        .await?
        .solution;

    println!(
        "Payload:    {}...",
        solution.payload.chars().take(64).collect::<String>()
    );

    // Decode the payload from base64, then POST it as `{"body": <decoded>}` to
    // the SBSD endpoint, which is `sbsd_url` without its query string. Keep the
    // cookies that response sets for every subsequent request.

    let balance = client.balance().await?;
    println!("Balance:    {}", balance);
    Ok(())
}
