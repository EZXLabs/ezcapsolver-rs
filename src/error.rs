use std::{collections::BTreeMap, fmt, time::Duration};

use serde_json::Value;

/// Result type returned by this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Structured error returned by the EzCaptchaSolver API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EzError {
    /// Optional server request identifier.
    pub request_id: Option<String>,
    /// Optional task identifier for errors returned after task creation.
    pub task_id: Option<String>,
    /// Stable machine-readable error code.
    pub error_code: Option<String>,
    /// Human-readable error description.
    pub error_description: Option<String>,
    /// Field-level validation errors.
    pub errors: BTreeMap<String, String>,
    /// HTTP status code that carried the API error.
    pub http_status: u16,
}

/// Codes that feed the service's per-key ban counter.
///
/// `/createTask` bans a key for three minutes after thirty of these within one
/// minute; `/getTaskResult` bans it for one minute after thirty of the first
/// two. Retrying is therefore worse than useless — it is what triggers the ban.
const AUTHENTICATION_ERROR_CODES: &[&str] = &[
    "ERROR_KEY_DOES_NOT_EXIST",
    "ERROR_KEY_NOT_AVAILABLE",
    "ERROR_ZERO_BALANCE",
];

/// Codes for which resending the identical request yields the identical failure.
///
/// The service's remaining codes are left out on purpose: an internal error, a
/// rate limit, a ban, and the two synchronous worker faults
/// (`ERROR_SERVICE_UNAVAILABLE`, `ERROR_SERVICE_TIMEOUT`) can all clear on their
/// own.
const TERMINAL_ERROR_CODES: &[&str] = &[
    "ERROR_CONTENT_TYPE_ERROR",
    "ERROR_KEY_DOES_NOT_EXIST",
    "ERROR_KEY_NOT_AVAILABLE",
    "ERROR_NOT_FOUND",
    "ERROR_PACKAGE_NOT_EXIST",
    "ERROR_PACKAGE_TASK_TYPE_NOT_SUPPORTED",
    "ERROR_REQUEST_METHOD",
    "ERROR_REQUEST_PARAMETERS",
    "ERROR_REQUEST_PROXY_MISSING",
    "ERROR_SUBSCRIPTION_EXPIRED",
    "ERROR_TASK_NOT_EXIST",
    "ERROR_TASK_TYPE_NOT_ALLOWED",
    "ERROR_TASK_TYPE_NOT_AVAILABLE",
    "ERROR_TASK_TYPE_NOT_SUPPORTED",
    "ERROR_WEBSITE_NOT_ALLOWED",
    "ERROR_ZERO_BALANCE",
];

/// The service's two throttling codes, both HTTP 429.
///
/// Neither says anything about a task: the query is refused before the service
/// looks it up, so the task keeps running and the next poll can still find it.
/// `/getTaskResult` counts only the first two authentication codes toward its
/// ban counter, so polling through a refusal does not dig the hole deeper.
const RATE_LIMITED_ERROR_CODES: &[&str] = &["ERROR_REQUEST_LIMIT", "ERROR_REQUEST_BANNED"];

impl EzError {
    /// Returns true when this is a credential or balance problem.
    ///
    /// A caller that retries one of these is not merely wasting a call: the
    /// service counts them per key, and thirty within a minute earn a
    /// three-minute ban. Stop rather than back off.
    #[must_use]
    pub fn is_authentication_error(&self) -> bool {
        self.error_code
            .as_deref()
            .is_some_and(|code| AUTHENTICATION_ERROR_CODES.contains(&code))
    }

    /// Returns true when resending the identical request would fail identically.
    ///
    /// An unrecognised code returns `false`, because a code this release has not
    /// seen may well be transient and the SDK should not talk a caller out of a
    /// retry that would have worked. `false` is not a promise that retrying is
    /// safe — the retry policy stays with the caller.
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        self.error_code
            .as_deref()
            .is_some_and(|code| TERMINAL_ERROR_CODES.contains(&code))
    }

    /// Returns true when the service refused the request for throttling rather
    /// than for anything about the request itself.
    ///
    /// Both codes are transient by construction: a rate limit resets with its
    /// window and a ban expires on its own. The asynchronous polling loop
    /// treats one as a skipped attempt instead of a failed task, and a caller
    /// polling by hand with `get_task_result` should do the same.
    ///
    /// This is narrower than the complement of [`Self::is_terminal`], which is
    /// false for every unrecognised code as well — including the worker codes
    /// that report a genuinely failed task.
    #[must_use]
    pub fn is_rate_limited(&self) -> bool {
        self.error_code
            .as_deref()
            .is_some_and(|code| RATE_LIMITED_ERROR_CODES.contains(&code))
    }
}

impl fmt::Display for EzError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = self.error_code.as_deref().unwrap_or("UNKNOWN_API_ERROR");
        let description = self
            .error_description
            .as_deref()
            .unwrap_or("The API returned an unspecified error");
        write!(
            formatter,
            "{code}: {description} (HTTP {})",
            self.http_status
        )?;
        // Validation failures are only actionable with the offending fields, so
        // they belong in the message rather than only in `errors`.
        for (index, (field, message)) in self.errors.iter().enumerate() {
            let separator = if index == 0 { "; " } else { ", " };
            write!(formatter, "{separator}{field}: {message}")?;
        }
        Ok(())
    }
}

impl std::error::Error for EzError {}

/// Errors produced by request construction, HTTP transport, API responses, and polling.
///
/// This enum is `#[non_exhaustive]`: new variants may be added in future minor
/// releases, so downstream matches must include a wildcard arm.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// A failure whose only useful detail is its description.
    ///
    /// An invalid client configuration arrives here today. Anything a caller
    /// would react to programmatically gets a variant of its own instead, so
    /// this one is meant to be printed rather than matched on.
    #[error("{0}")]
    Message(String),

    /// The HTTP request failed.
    ///
    /// Wraps the underlying [`reqwest::Error`], whose `is_timeout`,
    /// `is_connect`, and `status` methods classify the failure.
    #[cfg(any(feature = "async", feature = "blocking"))]
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    /// The response could not be used, either because it did not decode or
    /// because it violated the API contract.
    ///
    /// A ready result carrying no solution field lands here too: the task was
    /// billed and the answer is gone, which is a broken contract rather than a
    /// category of its own. [`Solution::is_missing`] still tells that case
    /// apart from a solution that was present and null.
    ///
    /// [`Solution::is_missing`]: crate::Solution::is_missing
    #[error("unexpected API response: {0}")]
    UnexpectedResponse(String),

    /// The API reported an error.
    ///
    /// This covers both a structured business error and any non-success HTTP
    /// status, so [`EzError::http_status`] is always available here.
    #[error(transparent)]
    EzError(Box<EzError>),

    /// Polling ended before the task reached a terminal state.
    #[error(
        "task `{task_id}` did not complete after {attempts} polling attempts at {interval:?} intervals"
    )]
    PollingExhausted {
        /// Task identifier.
        task_id: String,
        /// Number of completed result queries.
        attempts: usize,
        /// Delay configured between queries.
        interval: Duration,
    },

    /// The task was created and billed, but waiting for its result failed.
    ///
    /// `solve` creates the task internally, so the identifier never reaches the
    /// caller on its own. A failure that dropped it would leave a paid-for
    /// result unreachable — hence this variant, whose whole purpose is to carry
    /// the identifier out.
    ///
    /// Recover by waiting on the same task again rather than creating a second
    /// one: the service holds a result for five minutes after creation, and a
    /// new task is billed again. Errors that already carry the identifier
    /// themselves are left alone, so a business error still arrives as
    /// [`Self::EzError`] and an exhausted budget as [`Self::PollingExhausted`].
    #[error("task `{task_id}` was created but waiting for its result failed: {source}")]
    WaitInterrupted {
        /// Identifier of the task that was created and billed.
        task_id: String,
        /// Request identifier returned by task creation.
        request_id: Option<String>,
        /// The failure that interrupted the wait.
        #[source]
        source: Box<Error>,
    },

    /// A raw solution could not be converted into the requested type.
    ///
    /// The raw value travels with the error: a decode failure means the worker
    /// returned a shape this release does not model, and that shape is exactly
    /// what is needed to diagnose it. The convenience methods consume the task
    /// result, so this is the only place it survives.
    #[error("failed to decode task solution: {source}; raw solution: {}", preview_solution(.raw))]
    SolutionDecode {
        /// JSON decoding error.
        #[source]
        source: serde_json::Error,
        /// Raw solution value that failed to decode, preserved in full.
        raw: Box<Value>,
    },
}

/// Renders a raw solution for an error message, truncated so an oversized
/// worker response cannot bury the rest of the message. [`Error::SolutionDecode`]
/// still carries the untruncated value.
fn preview_solution(value: &Value) -> String {
    const MAX_CHARS: usize = 512;

    let rendered = value.to_string();
    let mut chars = rendered.chars();
    let head: String = chars.by_ref().take(MAX_CHARS).collect();
    if chars.next().is_some() {
        format!("{head}...")
    } else {
        head
    }
}

impl Error {
    /// Returns the identifier of a task that was created and billed, when this
    /// failure left one behind.
    ///
    /// Creating a task is what costs money, so a failure after that point
    /// leaves a result worth recovering: wait on this identifier again with
    /// `wait_for_result` instead of creating a second task. The service holds a
    /// result for five minutes after creation.
    ///
    /// `None` means nothing was billed — the failure happened before or during
    /// task creation, so there is nothing to recover.
    ///
    /// Which variant the identifier is stored on is an implementation detail;
    /// this is the one place to ask.
    ///
    /// ```
    /// # fn example(error: ezcapsolver::Error) {
    /// if let Some(task_id) = error.task_id() {
    ///     // The task is still on the service. Waiting again is free.
    ///     eprintln!("recover with wait_for_result({task_id})");
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn task_id(&self) -> Option<&str> {
        match self {
            Self::WaitInterrupted { task_id, .. } | Self::PollingExhausted { task_id, .. } => {
                Some(task_id)
            }
            Self::EzError(error) => error.task_id.as_deref(),
            _ => None,
        }
    }
}

impl From<EzError> for Error {
    fn from(error: EzError) -> Self {
        Self::EzError(Box::new(error))
    }
}

#[cfg(any(feature = "async", feature = "blocking"))]
impl Error {
    /// Reports an invalid client configuration.
    ///
    /// The prefix is added here rather than at each call site so that every
    /// configuration failure reads the same way. These are raised while a
    /// client is being built, never once one is in use, so a caller holding
    /// this error has not been billed for anything.
    pub(crate) fn config(detail: impl fmt::Display) -> Self {
        Self::Message(format!("invalid client configuration: {detail}"))
    }

    /// Attaches the identifiers of a created task to a failure raised while
    /// waiting for its result.
    ///
    /// Every failure leaving the wait has to carry the task identifier, because
    /// the caller of `solve` has no other way to reach a task that was already
    /// billed. Variants that hold the identifier themselves are filled in
    /// place; the rest are wrapped, which is the only place to put it.
    pub(crate) fn with_task_context(self, task_id: String, request_id: Option<String>) -> Self {
        match self {
            Self::EzError(mut error) => {
                error.task_id = Some(task_id);
                error.request_id = request_id;
                Self::EzError(error)
            }
            // These already carry a task_id, so wrapping them would only make a
            // caller unwrap one more layer to read the same value.
            already_identified @ (Self::PollingExhausted { .. } | Self::WaitInterrupted { .. }) => {
                already_identified
            }
            source => Self::WaitInterrupted {
                task_id,
                request_id,
                source: Box::new(source),
            },
        }
    }
}
