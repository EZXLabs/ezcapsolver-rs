//! Demonstrates a fully customized task workflow using dynamic JSON and manual polling.
//!
//! `AsyncEzCapSolverClient::solve` already creates a task and waits for it. This example
//! uses `create_task` and `get_task_result` directly, so the payload and the
//! polling policy stay under caller control.

use std::time::Duration;

use anyhow::anyhow;
use ezcapsolver::{AsyncEzCapSolverClient, TaskType::RecaptchaV2TaskProxyless};
use serde_json::{Value, json};

const POLL_INTERVAL: Duration = Duration::from_secs(5);
const MAX_ATTEMPTS: usize = 24;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = AsyncEzCapSolverClient::new()?;

    let payload = json!({
        "websiteURL": "https://www.google.com/recaptcha/api2/demo",
        "websiteKey": "6Le-wvkSAAAAAPBMRTvw0Q4Muexq9bi0DJwx_mJ-",
        "isInvisible": false
    });

    let task_id = client
        .create_task(RecaptchaV2TaskProxyless, &payload)
        .await?
        .task_id;
    println!("Task [{task_id}] created");

    for attempt in 1..=MAX_ATTEMPTS {
        tokio::time::sleep(POLL_INTERVAL).await;
        let result = client.get_task_result(&task_id).await?;

        if result.is_ready() {
            let solution: Value = result.deserialize_solution()?;
            println!("Task [{task_id}] solved: {solution:#?}");
            return Ok(());
        }
        println!(
            "Task [{task_id}] {} ({attempt}/{MAX_ATTEMPTS})",
            result.status.as_str()
        );
    }
    Err(anyhow!(
        "Task [{task_id}] did not finish within {MAX_ATTEMPTS} attempts"
    ))
}
