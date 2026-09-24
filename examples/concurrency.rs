//! Demonstrates how to drive many tasks through one EzCaptchaSolver client.
//!
//! A client holds a connection pool and is cheap to clone — its inner state is
//! shared, so cloning does not duplicate the pool. Building one per task only
//! wastes connections.
//!
//! Both clients are safe to share: [`AsyncEzCapSolverClient`] across tasks, and
//! `EzCapSolverClient` across threads.

use ezcapsolver::{AsyncEzCapSolverClient, RecaptchaV2Task};

/// The sites to solve for, in parallel.
const SITES: [(&str, &str); 3] = [
    ("https://a.example.com", "6Lc_site_key_a"),
    ("https://b.example.com", "6Lc_site_key_b"),
    ("https://c.example.com", "6Lc_site_key_c"),
];

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = AsyncEzCapSolverClient::with_client_key(std::env::var("EZCAPTCHA_API_KEY")?)?;

    let mut running = tokio::task::JoinSet::new();
    for (url, key) in SITES {
        // Cloning shares the pool rather than opening a second one.
        let client = client.clone();
        running.spawn(async move {
            let task = RecaptchaV2Task::builder()
                .website_url(url)
                .website_key(key)
                .build();
            (url, client.solve_recaptcha_v2_task_proxyless(&task).await)
        });
    }

    // One failure must not take the other tasks down with it: they are billed
    // whether or not this process keeps waiting for them.
    while let Some(joined) = running.join_next().await {
        match joined? {
            (url, Ok(solved)) => println!("{url}: {}", solved.solution.token),
            (url, Err(error)) => eprintln!("{url}: {error}"),
        }
    }

    println!("Balance:    {}", client.balance().await?);
    Ok(())
}
