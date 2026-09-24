use serde::{Serialize, Serializer, ser::Error as _};
use serde_json::Value;

/// Create task parameters.
#[derive(Debug)]
pub(crate) struct TaskPayload<'a, T: ?Sized> {
    /// Task type, already resolved to its wire name
    pub(crate) task_type: &'a str,
    /// Specific parameters of the task
    pub(crate) params: &'a T,
}

impl<T> Serialize for TaskPayload<'_, T>
where
    T: Serialize + ?Sized,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = serde_json::to_value(self.params).map_err(S::Error::custom)?;
        let Value::Object(mut params) = value else {
            return Err(S::Error::custom(
                "task parameters must serialize as an object",
            ));
        };
        params.insert("type".to_owned(), Value::String(self.task_type.to_owned()));
        params.serialize(serializer)
    }
}

/// Create task request parameters
#[derive(Debug, Serialize)]
pub(crate) struct CreateTaskRequest<'a, T: ?Sized> {
    /// Account secret key or subscription secret key
    #[serde(rename = "clientKey")]
    pub(crate) client_key: &'a str,
    /// Developer Application ID (optional)
    #[serde(rename = "appId", skip_serializing_if = "Option::is_none")]
    pub(crate) app_id: Option<i32>,
    /// Task payload
    pub(crate) task: TaskPayload<'a, T>,
}

/// Request parameters for querying asynchronous task results
#[derive(Debug, Serialize)]
pub(crate) struct QueryTaskResultRequest<'a> {
    /// Account secret key or subscription secret key
    /// It must match the user or subscription secret key used when creating the task.
    #[serde(rename = "clientKey")]
    pub(crate) client_key: &'a str,
    /// Task ID
    #[serde(rename = "taskId")]
    pub(crate) task_id: &'a str,
}

/// Request parameters for account balance query
#[derive(Debug, Serialize)]
pub(crate) struct QueryBalanceRequest<'a> {
    #[serde(rename = "clientKey")]
    pub(crate) client_key: &'a str,
}
