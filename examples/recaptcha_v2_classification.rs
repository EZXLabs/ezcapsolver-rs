//! Demonstrates how to use EzCaptchaSolver to solve reCAPTCHA Image recognition task
//!
//! **Task type:** [`ReCaptchaV2Classification`]
//!
//! For detailed usage instructions, see the
//! [official EzCaptchaSolver documentation](https://docs.ezxlabs.com/docs/captcha/api/recaptcha-v2-classification).

use base64::{Engine, engine::general_purpose::STANDARD};
use ezcapsolver::{AsyncEzCapSolverClient, RecaptchaV2ClassificationTask};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = AsyncEzCapSolverClient::with_client_key(std::env::var("EZCAPTCHA_API_KEY")?)?;

    let image = STANDARD.encode(std::fs::read("examples/fixtures/crosswalks3x3.jpg")?);
    let task_payload = RecaptchaV2ClassificationTask::builder()
        .image(image)
        .question("/m/014xcs")
        .size(3)
        .build();

    let solution = client
        .sync_solve_recaptcha_v2_classification(&task_payload)
        .await?
        .solution;

    if solution.is_multi() {
        println!("Multi objects: {:?}", solution.objects);
    } else if solution.is_single() {
        println!("Single hasObject: {}", solution.has_object);
    } else {
        println!("Type: {}, extra: {:?}", solution.r#type, solution.extra);
    }

    let balance = client.balance().await?;
    println!("Balance:    {}", balance);
    Ok(())
}
