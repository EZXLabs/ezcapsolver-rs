//! This example demonstrates how to use EzCaptchaSolver to generate an Incapsula
//! Reese84 sensor payload.
//!
//! **Task type:** [`IncapsulaTaskProxyless`]
//!
//! Incapsula runs through the synchronous API, so the solver returns the
//! solution directly instead of polling for it.
//!
//! For detailed usage instructions, see the
//! [official EzCaptchaSolver documentation](https://ezxlabs.com/en/docs/captcha/api/incapsula).

use ezcapsolver::{AsyncEzCapSolverClient, IncapsulaTask};

/// Full source of the Reese84 sensor script, fetched from `script_url`.
const SCRIPT: &str = "";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = AsyncEzCapSolverClient::with_client_key(std::env::var("EZCAPTCHA_API_KEY")?)?;

    let task_payload = IncapsulaTask::builder()
        .script(SCRIPT)
        .script_url("https://example.com/xxxxx?d=example.com")
        .page_url("https://example.com")
        // Must match the Accept-Language header used by the browser flow.
        .accept_language("en-US,en;q=0.9")
        // Only Chrome 147, 148 and 149 are supported; the major version is all
        // the service reads out of this.
        .ua("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
             (KHTML, like Gecko) Chrome/149.0.0.0 Safari/537.36")
        // Supplying a proxy makes the worker submit the payload and return the
        // cookie; without one it only generates the payload.
        .maybe_proxy(std::env::var("EZCAPTCHA_PROXY").ok())
        // Only the sites with PoW challenges enabled need this one.
        .maybe_pow(std::env::var("EZCAPTCHA_INCAPSULA_POW").ok())
        .build();

    let solution = client
        .sync_solve_incapsula_task_proxyless(&task_payload)
        .await?
        .solution;

    let status = solution
        .status
        .map_or_else(|| String::from("-"), |code| code.to_string());
    println!("Status:     {status}");
    // `data` is the JSON string to POST to the Reese84 endpoint.
    println!(
        "Data:       {}...",
        solution.data.chars().take(64).collect::<String>()
    );

    let balance = client.balance().await?;
    println!("Balance:    {}", balance);
    Ok(())
}
