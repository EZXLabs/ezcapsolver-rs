//! This example demonstrates how to use a specific TLS fingerprint to forward HTTP requests.
//!
//! **Task type:** [`TlsTask`]
//!
//! For detailed usage instructions, see the
//! [official EzCaptchaSolver documentation](https://ezxlabs.com/en/docs/captcha/api/tls-forward).

use std::time::Duration;

use ezcapsolver::{
    AsyncEzCapSolverClient, ClientConfig, PollingConfig, TlsForwardTask, TlsHttpMethod::Get,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build the client
    let config = ClientConfig::builder()
        .timeout(Duration::from_secs(30))
        .polling(PollingConfig::new(Duration::from_secs(3), 10))
        .build();
    let client = AsyncEzCapSolverClient::with_config(config)?;
    // Build task parameters
    let task_payload = TlsForwardTask::builder()
        .tls_type("chrome146")
        .proxy(std::env::var("EZCAPTCHA_PROXY")?)
        .method(Get)
        .url("https://postman-echo.com/get")
        .build();

    // Solve task
    match client.sync_solve_tls_task(&task_payload).await {
        Ok(solution) => {
            println!("solve success: {:?}", solution);
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
