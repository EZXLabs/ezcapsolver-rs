//! Extensions built on top of the task types.
//!
//! The clients map one call to one task. An extension runs a whole workflow
//! around one or more task types instead — fetching pages, calling several
//! steps, and submitting the answer — so the caller only supplies the inputs a
//! person would. Workflow methods end in `_workflow` and are added to the
//! clients; their inputs and results live here.
//!
//! Each extension sits behind a feature of its own, since a workflow usually
//! needs dependencies the task API does not:
//!
//! | Feature | Workflow |
//! | --- | --- |
//! | `datadome` | [`AsyncEzCapSolverClient::solve_data_dome_workflow`] |
//!
//! Extensions are new, and their API may still change.
//!
//! [`AsyncEzCapSolverClient::solve_data_dome_workflow`]: crate::AsyncEzCapSolverClient::solve_data_dome_workflow

#[cfg(feature = "datadome")]
mod datadome;

#[cfg(feature = "datadome")]
#[cfg_attr(docsrs, doc(cfg(feature = "datadome")))]
pub use datadome::{DataDomeChallenge, DataDomeChallengeBuilder, DataDomeCookie, DataDomeOutcome};
