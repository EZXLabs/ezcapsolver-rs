//! Demonstrates how to use EzCaptchaSolver to read one FunCaptcha image.
//!
//! **Task type:** [`FunCaptchaClassification`]
//!
//! The solution shape for this type is not confirmed, so
//! [`FunCaptchaClassificationSolution`] declares no fields of its own and
//! everything the worker returns arrives in `extra`. The untouched JSON is on
//! `Solved::raw`.
//!
//! For detailed usage instructions, see the
//! [official EzCaptchaSolver documentation](https://docs.ezxlabs.com/docs/captcha/api).

use base64::{Engine, engine::general_purpose::STANDARD};
use ezcapsolver::{AsyncEzCapSolverClient, FunCaptchaClassificationTask};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = AsyncEzCapSolverClient::with_client_key(std::env::var("EZCAPTCHA_API_KEY")?)?;

    let image = STANDARD.encode(std::fs::read("examples/fixtures/crosswalks1x1.jpg")?);
    let task_payload = FunCaptchaClassificationTask::builder()
        .image(image)
        .question("Pick the image that is the right way up")
        .build();

    let solved = client
        .sync_solve_funcaptcha_classification(&task_payload)
        .await?;

    println!("Fields:     {:?}", solved.solution.extra);
    println!("Raw:        {:?}", solved.raw.as_value());

    println!("Balance:    {}", client.balance().await?);
    Ok(())
}
