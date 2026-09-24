//! Rust SDK for the EzCaptchaSolver task API.
//!
//! The crate exposes task and response models without requiring an HTTP client
//! feature. The default feature set enables both clients: [`EzCapSolverClient`]
//! blocks the calling thread, [`AsyncEzCapSolverClient`] runs on Tokio. The names
//! match the other language SDKs, where the plain name is the synchronous
//! client and the `Async` prefix marks the asynchronous one.

// `doc_cfg` is nightly-only and is enabled only for the docs.rs build, which
// passes `--cfg docsrs`. Stable builds never see this attribute.
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "async")]
mod async_client;
#[cfg(feature = "blocking")]
mod blocking_client;
mod config;
mod error;
#[cfg(any(feature = "async", feature = "blocking"))]
mod request;
mod response;
mod solution;
#[cfg(any(feature = "async", feature = "blocking"))]
mod task_solvers;
mod task_type;
mod tasks;
#[cfg(any(feature = "async", feature = "blocking"))]
mod transport;
mod wire;

#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
pub use async_client::AsyncEzCapSolverClient;
#[cfg(feature = "blocking")]
#[cfg_attr(docsrs, doc(cfg(feature = "blocking")))]
pub use blocking_client::EzCapSolverClient;
pub use config::{
    ClientConfig, ClientConfigBuilder, DEFAULT_ASYNC_BASE_URL, DEFAULT_CLIENT_KEY_ENV,
    DEFAULT_MAX_POLL_ATTEMPTS, DEFAULT_POLL_INTERVAL, DEFAULT_SYNC_BASE_URL, DEFAULT_SYNC_TIMEOUT,
    DEFAULT_TIMEOUT, PollingConfig,
};
pub use error::{Error, EzError, Result};
pub use response::{CreateTaskResponse, ResponseMeta, Solved, TaskResult, TaskStatus};
pub use solution::{
    AkamaiSbsdSolution, AkamaiWebSolution, Cloudflare5sSolution, CloudflareTurnstileSolution,
    DataDomeSolution, FunCaptchaClassificationSolution, FunCaptchaSolution,
    HcaptchaClassificationSolution, HcaptchaSolution, IncapsulaSolution, PerimeterXSolution,
    ReClassificationSolution, RecaptchaSolution, Solution, TlsForwardSolution,
};
pub use task_type::{TaskMode, TaskType, TaskTypeName};
pub use tasks::*;
