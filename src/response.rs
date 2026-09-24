use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};

use crate::{Result, Solution};

/// Envelope metadata included in every API response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseMeta {
    /// On success it is 0, on failure it is 1.
    pub error_id: i32,
    /// Request Tracking ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Error code
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    /// Error message
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_description: Option<String>,
}

/// Response returned after an asynchronous task is created.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskResponse {
    /// Shared response metadata.
    #[serde(flatten)]
    pub meta: ResponseMeta,
    /// Identifier used to query the task result.
    pub task_id: String,
}

/// Current state of a task.
///
/// The service reports exactly these three states, so this enum is closed and
/// callers can match it exhaustively without a wildcard arm. A status outside
/// this set is rejected during deserialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskStatus {
    /// The task is still being processed.
    Processing,
    /// The task completed successfully.
    Ready,
    /// The task failed.
    Error,
}

impl TaskStatus {
    /// Returns the wire representation of the status.
    #[must_use]
    pub const fn as_str(&self) -> &str {
        match self {
            Self::Processing => "processing",
            Self::Ready => "ready",
            Self::Error => "error",
        }
    }
}

impl Serialize for TaskStatus {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for TaskStatus {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "processing" => Ok(Self::Processing),
            "ready" => Ok(Self::Ready),
            "error" => Ok(Self::Error),
            other => Err(D::Error::unknown_variant(
                other,
                &["processing", "ready", "error"],
            )),
        }
    }
}

/// Response returned by task result queries and synchronous task creation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskResult {
    /// Shared response metadata.
    #[serde(flatten)]
    pub meta: ResponseMeta,
    /// Current task status.
    ///
    /// This field is required. The service may omit `status` on request-level
    /// failures such as an invalid key or an unknown task ID, but those carry
    /// an error envelope that is rejected before this type is decoded. A
    /// success envelope without a status therefore violates the API contract,
    /// and reporting it as [`Error::UnexpectedResponse`] with the raw body is
    /// more useful than silently substituting a placeholder status.
    ///
    /// [`Error::UnexpectedResponse`]: crate::Error::UnexpectedResponse
    pub status: TaskStatus,
    /// Identifier assigned by the synchronous task endpoint.
    ///
    /// `/createSyncTask` returns one on both the success and the failure path;
    /// `/getTaskResult` does not echo it back, so this is `None` there.
    ///
    /// The name is spelled out because this struct has no `rename_all`: the
    /// other fields already match the wire, and a blanket rule would have to be
    /// re-checked against every one of them.
    #[serde(rename = "taskId", default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    /// Raw task solution, preserving a missing field separately from JSON null.
    #[serde(default, skip_serializing_if = "Solution::is_missing")]
    pub solution: Solution,
}

impl TaskResult {
    /// Returns true when the task is still processing.
    #[must_use]
    pub fn is_processing(&self) -> bool {
        self.status == TaskStatus::Processing
    }

    /// Returns true when the task completed successfully.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.status == TaskStatus::Ready
    }

    /// Deserializes the raw solution into a caller-selected type.
    pub fn deserialize_solution<T>(&self) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        self.solution.deserialize()
    }
}

/// Completed task with its identifiers and solution.
///
/// This is an SDK-side aggregate rather than a wire type, so it is serializable
/// for logging and storage but is never decoded from an API response.
///
/// Both execution modes return this shape, [`task_id`] included.
///
/// [`task_id`]: Solved::task_id
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Solved<T> {
    /// Identifier the service assigned to the task.
    ///
    /// Both endpoints supply one: the asynchronous path returns it from task
    /// creation, and the synchronous path returns it alongside the result.
    /// `None` only when the service omitted it.
    pub task_id: Option<String>,
    /// Request identifier returned by task creation.
    pub request_id: Option<String>,
    /// Decoded or raw task solution.
    pub solution: T,
    /// The untouched solution JSON.
    ///
    /// Whatever the typed model in [`solution`] does not declare stays
    /// recoverable here, so a worker that starts returning a new field never
    /// costs the caller data.
    ///
    /// [`solution`]: Solved::solution
    pub raw: Solution,
}

/// Response returned by the balance endpoint.
///
/// Internal: the balance is exposed directly as an `f64`, so this envelope
/// never reaches callers.
///
/// Only the two clients use it, so it is gated alongside them: without
/// that, a build with neither client enabled reports it as dead code.
#[cfg(any(feature = "async", feature = "blocking"))]
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub(crate) struct BalanceResponse {
    /// Shared response metadata.
    #[serde(flatten)]
    #[allow(dead_code, reason = "decoded from the envelope but not read")]
    pub meta: ResponseMeta,
    /// Balance returned by the service.
    ///
    /// The API sends a JSON number with at most four decimal places, which
    /// `f64` round-trips exactly. The value is for display only, so exact
    /// decimal arithmetic is not required here.
    pub balance: f64,
}
