//! Demonstrates how to use EzCaptchaSolver to clear a DataDome challenge in one
//! call and get the cookie it issues.
//!
//! **Task type:** [`DataDomeTaskProxyless`], steps 1 to 3, run inside the call.
//!
//! Needs the `datadome` feature:
//!
//! ```text
//! cargo run --example datadome_workflow --features datadome
//! ```
//!
//! The call fetches the page, locates and loads the challenge, reads its script,
//! asks the service for the answer, and submits it. Browser-side requests go
//! through the proxy with the Chrome TLS fingerprint the user agent names, so
//! keep using both for the requests that carry the cookie.
//!
//! For detailed usage instructions, see the
//! [official EzCaptchaSolver documentation](https://docs.ezxlabs.com/docs/captcha/api).

use ezcapsolver::{
    AsyncEzCapSolverClient,
    ext::{DataDomeChallenge, DataDomeOutcome},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = AsyncEzCapSolverClient::with_client_key("YOUR_API_KEY")?;

    let challenge = DataDomeChallenge::builder()
        .target_url("https://example.com/signup")
        .user_agent(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
             (KHTML, like Gecko) Chrome/153.0.0.0 Safari/537.36",
        )
        .maybe_proxy(std::env::var("EZCAPTCHA_PROXY").ok())
        .build();

    match client.solve_data_dome_workflow(&challenge).await? {
        DataDomeOutcome::Passed(cookie) => {
            println!("Cookie:   {}={}", cookie.name, cookie.value);
            println!("Domain:   {}", cookie.domain);
        }
        DataDomeOutcome::NotTriggered => println!("DataDome did not intercept the page"),
    }

    println!("Balance:  {}", client.balance().await?);
    Ok(())
}
