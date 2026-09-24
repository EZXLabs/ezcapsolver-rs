use serde::Serialize;

use super::ExtraFields;

/// PerimeterX task.
#[derive(Debug, Clone, Serialize, Default, bon::Builder)]
#[builder(on(String, into))]
#[serde(remote = "Self", rename_all = "camelCase")]
pub struct PerimeterXTask {
    /// PerimeterX application identifier.
    pub website_key: String,
    /// Whether the challenge uses invisible mode.
    #[builder(default)]
    pub invisible: bool,
    /// Additional worker parameters.
    #[serde(skip)]
    #[builder(default)]
    pub extra: ExtraFields,
}
