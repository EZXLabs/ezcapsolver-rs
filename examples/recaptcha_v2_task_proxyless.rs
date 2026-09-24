//! Demonstrates how to use EzCaptchaSolver to solve reCAPTCHA v2 challenges,
//! with SDK logging turned on.
//!
//! **Task type:** [`ReCaptchaV2TaskProxyless`]
//!
//! The subscriber below shows every SDK log level:
//!
//! - `INFO`: task created, task completed
//! - `DEBUG`: request lifecycle and each polling attempt
//! - `TRACE`: full request bodies, response status, duration, and response
//!   bodies, with `clientKey` and `proxy` values replaced by `[REDACTED]`
//!
//! For detailed usage instructions, see the
//! [official EzCaptchaSolver documentation](https://docs.ezxlabs.com/docs/captcha/api/recaptcha-v2).

use std::time::Duration;

use ezcapsolver::{AsyncEzCapSolverClient, ClientConfig, PollingConfig, RecaptchaV2Task};
use tracing::Level;
use tracing_subscriber::{filter::Targets, prelude::*};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Scope the filter to this crate. A global TRACE level would also enable
    // the HTTP stack's own trace output and bury the SDK logs. Lower this to
    // `Level::DEBUG` for the lifecycle without bodies, or `Level::INFO` for
    // task events only.
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(Targets::new().with_target("ezcapsolver", Level::DEBUG))
        .init();

    // Build the client
    let config = ClientConfig::builder()
        .timeout(Duration::from_secs(30))
        .polling(PollingConfig::new(Duration::from_secs(5), 24))
        .build();
    let client = AsyncEzCapSolverClient::with_config(config)?;

    // Build task parameters
    let task_payload = RecaptchaV2Task::builder()
        .website_url("https://www.google.com/recaptcha/api2/demo")
        .website_key("6Le-wvkSAAAAAPBMRTvw0Q4Muexq9bi0DJwx_mJ-")
        .build();
    // Solve task
    match client
        .solve_recaptcha_v2_task_proxyless(&task_payload)
        .await
    {
        Ok(task) => {
            println!("Trace ID:   {}", task.request_id.as_deref().unwrap_or("-"));
            println!("Task ID:    {}", task.task_id.as_deref().unwrap_or("-"));
            println!("User-Agent: {}", task.solution.user_agent);
            println!(
                "Token:      {}...",
                task.solution.token.chars().take(64).collect::<String>()
            );
        }
        Err(ezcapsolver::Error::EzError(e)) => {
            println!(
                "EzCaptchaSolver error, task_id: {:?}, request_id: {:?}, errorCode: {:?}, errorDescription: {:?}, errors: {:?}",
                e.task_id, e.request_id, e.error_code, e.error_description, e.errors
            )
        }
        Err(e) => {
            eprintln!("solve error: {:?}", e)
        }
    }

    Ok(())
}
