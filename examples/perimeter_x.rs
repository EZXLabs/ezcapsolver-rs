//! This example demonstrates how to use EzCaptchaSolver to solve PerimeterX
//! (Press & Hold) challenges.
//!
//! **Task type:** [`PerimeterX`]
//!
//! PerimeterX has no stable solution schema, so the solver hands back the raw
//! [`serde_json::Value`]. What the protected site checks are the cookies it
//! carries, such as `_px3`, `_pxvid` and `_pxde`.
//!
//! For detailed usage instructions, see the
//! [official EzCaptchaSolver documentation](https://ezxlabs.com/en/docs/captcha/api/perimeterx).

use ezcapsolver::{AsyncEzCapSolverClient, PerimeterXTask};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = AsyncEzCapSolverClient::with_client_key(std::env::var("EZCAPTCHA_API_KEY")?)?;

    let task_payload = PerimeterXTask::builder()
        // PerimeterX application identifier, which starts with `PX`.
        .website_key("PXxxxxxxxx")
        // The field is `invisible`, with no `is` prefix, unlike the reCAPTCHA
        // types. Enabling it changes how the task is priced.
        .invisible(false)
        .build();

    let task = client.solve_perimeter_x(&task_payload).await?;

    println!("Trace ID:   {}", task.request_id.as_deref().unwrap_or("-"));
    println!("Task ID:    {}", task.task_id.as_deref().unwrap_or("-"));
    println!("Solution:   {:#?}", task.solution);

    let balance = client.balance().await?;
    println!("Balance:    {}", balance);
    Ok(())
}
