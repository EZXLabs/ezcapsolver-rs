//! Demonstrates how to use EzCaptchaSolver to report a DataDome fingerprint on the
//! normal browsing path, in exchange for a cookie.
//!
//! **Task type:** [`DataDomeTagsTaskProxyless`]
//!
//! This is a different worker from [`DataDomeTaskProxyless`]: that one answers a
//! challenge after an interception, this one reports tags before any challenge
//! appears. Note the field naming — this model is flat lowercase where the
//! challenge model is snake_case.
//!
//! For detailed usage instructions, see the
//! [official EzCaptchaSolver documentation](https://docs.ezxlabs.com/docs/captcha/api).

use std::collections::BTreeMap;

use ezcapsolver::{AsyncEzCapSolverClient, DataDomeJsType, DataDomeTagsTask};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = AsyncEzCapSolverClient::with_client_key(std::env::var("EZCAPTCHA_API_KEY")?)?;

    let task_payload = DataDomeTagsTask::builder()
        // Read from the site's inline snippet as `window.ddjskey`.
        .ddk("ABCDEF0123456789ABCDEF0123456789")
        // `Ch` fixes the packet counter at 1; `Le` starts at 2 and counts up.
        .jstype(DataDomeJsType::Ch)
        .bpc(1)
        .cid("")
        .referer("https://example.com/product/1")
        .ua("Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/131.0.0.0")
        // External-key sites derive the call stack and fonts from this.
        .fields(BTreeMap::from([(
            "tags_url".to_owned(),
            serde_json::json!("https://example.com/tags.js"),
        )]))
        .build();

    let solved = client
        .sync_solve_data_dome_tags_task_proxyless(&task_payload)
        .await?;

    println!(
        "Kind:       {}",
        solved.solution.kind.as_deref().unwrap_or("-")
    );
    println!("Raw:        {:?}", solved.raw.as_value());

    println!("Balance:    {}", client.balance().await?);
    Ok(())
}
