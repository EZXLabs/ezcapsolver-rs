//! This example demonstrates how to use EzCaptchaSolver to solve FunCaptcha
//! (Arkose Labs) challenges.
//!
//! **Task type:** [`FuncaptchaTaskProxyless`]
//!
//! For detailed usage instructions, see the
//! [official EzCaptchaSolver documentation](https://ezxlabs.com/en/docs/captcha/api/funcaptcha).

use ezcapsolver::{AsyncEzCapSolverClient, FunCaptchaTask};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = AsyncEzCapSolverClient::with_client_key(std::env::var("EZCAPTCHA_API_KEY")?)?;

    // Two optional setters are left out: `data` carries the Arkose Labs blob as
    // a JSON string such as `{"blob":"..."}`, and `api_js_subdomain` is only
    // needed when the site serves Arkose Labs from a custom subdomain.
    let task_payload = FunCaptchaTask::builder()
        .website_url("https://example.com")
        .website_key("your_public_key")
        // FunCaptcha uses `protocol://host:port:username:password` rather than
        // the usual proxy format, and a rotating proxy must support sessions.
        .maybe_proxy(std::env::var("EZCAPTCHA_PROXY").ok())
        .build();

    let task = client
        .solve_funcaptcha_task_proxyless(&task_payload)
        .await?;

    println!("Trace ID:   {}", task.request_id.as_deref().unwrap_or("-"));
    println!("Task ID:    {}", task.task_id.as_deref().unwrap_or("-"));
    println!(
        "Token:      {}...",
        task.solution.token.chars().take(64).collect::<String>()
    );

    let balance = client.balance().await?;
    println!("Balance:    {}", balance);
    Ok(())
}
