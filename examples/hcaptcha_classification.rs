//! Demonstrates how to use EzCaptchaSolver to read one or more hCaptcha images.
//!
//! **Task type:** [`HCaptchaClassification`]
//!
//! Every parameter is optional: different classification modules take different
//! input combinations, and the service declares no validation for this type. The
//! solution shape is not confirmed either, so everything arrives in `extra`.
//!
//! For detailed usage instructions, see the
//! [official EzCaptchaSolver documentation](https://docs.ezxlabs.com/docs/captcha/api).

use base64::{Engine, engine::general_purpose::STANDARD};
use ezcapsolver::{AsyncEzCapSolverClient, HcaptchaClassificationTask};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = AsyncEzCapSolverClient::with_client_key(std::env::var("EZCAPTCHA_API_KEY")?)?;

    let image = STANDARD.encode(std::fs::read("examples/fixtures/crosswalks1x1.jpg")?);
    let task_payload = HcaptchaClassificationTask::builder()
        .images(vec![image])
        .question("Please click each image containing a crosswalk")
        .build();

    let solved = client
        .sync_solve_hcaptcha_classification(&task_payload)
        .await?;

    println!("Fields:     {:?}", solved.solution.extra);
    println!("Raw:        {:?}", solved.raw.as_value());

    println!("Balance:    {}", client.balance().await?);
    Ok(())
}
