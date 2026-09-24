use std::{collections::BTreeMap, time::Duration};

use reqwest::header::HeaderValue;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use tracing::{Level, enabled, trace};

use crate::{Error, EzError, Result};

/// JSON keys whose values are credentials and must never reach the logs.
const REDACTED_KEYS: &[&str] = &["clientKey", "proxy"];

/// Characters of a body kept in a trace log. An error envelope fits well under
/// this; a solved token and an oversized task parameter get truncated, which is
/// the point — a DataDome `html_b64` or an Akamai `script_base64` runs to
/// megabytes and would otherwise land in the log verbatim.
const TRACE_PREVIEW_CHARS: usize = 256;

/// Header carrying the client key on every request, alongside the `clientKey`
/// field of the body.
pub(crate) const CLIENT_KEY_HEADER: &str = "X-API-Key";

/// Builds the client-key header value once, when a client is created.
///
/// The value is marked sensitive, which keeps it out of `Debug` output and out
/// of the HTTP/2 header compression table. A key that is not a valid header
/// value is rejected here as a configuration error, rather than failing every
/// request later with an opaque transport error.
pub(crate) fn client_key_header(client_key: &str) -> Result<HeaderValue> {
    let mut value = HeaderValue::from_str(client_key)
        .map_err(|_| Error::config("client key contains invalid characters"))?;
    value.set_sensitive(true);
    Ok(value)
}

#[derive(Debug, Deserialize)]
struct ApiEnvelope {
    #[serde(rename = "errorId", default)]
    error_id: i32,
    #[serde(rename = "errorCode", default)]
    error_code: Option<String>,
    #[serde(rename = "errorDescription", default)]
    error_description: Option<String>,
    #[serde(rename = "requestId", default)]
    request_id: Option<String>,
    #[serde(default)]
    errors: std::collections::BTreeMap<String, String>,
}

pub(crate) fn endpoint(base_url: &str, path: &str) -> String {
    format!(
        "{}/{}",
        base_url.trim_end_matches('/'),
        path.trim_start_matches('/')
    )
}

pub(crate) fn parse_api_response<T>(status: u16, body: &[u8]) -> Result<T>
where
    T: DeserializeOwned,
{
    let value = match serde_json::from_slice::<serde_json::Value>(body) {
        Ok(value) => value,
        // A non-success status whose body is not JSON is still an error from
        // the service, so it is reported with the same shape as one that does
        // carry an envelope.
        Err(_) if !(200..300).contains(&status) => {
            return Err(status_only_error(status, body));
        }
        Err(source) => {
            return Err(Error::UnexpectedResponse(format!(
                "{source}; body: {}",
                body_preview(body)
            )));
        }
    };

    // `errorId` is the sole discriminator: the service sets it on every
    // response and `0` always means success. Treating a non-empty `errorCode`
    // as a second signal would let a future success-side code turn a solved
    // task into an error, and every code the service defines is already
    // paired with a non-zero `errorId`.
    //
    // Reading the envelope borrows the parsed body; deserializing from an owned
    // copy would deep-clone every response just to look at four fields.
    if let Ok(envelope) = ApiEnvelope::deserialize(&value)
        && envelope.error_id != 0
    {
        return Err(EzError {
            task_id: None,
            error_code: envelope.error_code,
            error_description: envelope.error_description,
            request_id: envelope.request_id,
            errors: envelope.errors,
            http_status: status,
        }
        .into());
    }

    if !(200..300).contains(&status) {
        return Err(status_only_error(status, body));
    }

    serde_json::from_value(value).map_err(|source| {
        Error::UnexpectedResponse(format!("{source}; body: {}", body_preview(body)))
    })
}

/// Reports a non-success status whose body carried no error envelope.
///
/// This keeps [`EzError::http_status`] the single place to read the status
/// code, whether or not the service described the failure.
fn status_only_error(status: u16, body: &[u8]) -> Error {
    EzError {
        request_id: None,
        task_id: None,
        error_code: None,
        error_description: Some(body_preview(body)),
        errors: BTreeMap::new(),
        http_status: status,
    }
    .into()
}

/// Logs an outgoing request body with credentials replaced.
///
/// The body is not rendered at all unless trace logging is active, so this
/// costs nothing at the default log level.
pub(crate) fn log_request<B>(url: &str, body: &B)
where
    B: Serialize + ?Sized,
{
    if !enabled!(Level::TRACE) {
        return;
    }
    match serde_json::to_value(body) {
        Ok(mut value) => {
            redact(&mut value);
            trace!(
                url,
                body = %preview(value.to_string().as_bytes(), TRACE_PREVIEW_CHARS),
                "sending API request"
            );
        }
        Err(error) => {
            trace!(url, %error, "sending API request with an unrenderable body");
        }
    }
}

/// Logs the status, duration, size, and truncated body of an API response.
pub(crate) fn log_response(url: &str, status: u16, elapsed: Duration, body: &[u8]) {
    if !enabled!(Level::TRACE) {
        return;
    }
    trace!(
        url,
        status,
        elapsed_ms = elapsed.as_millis() as u64,
        bytes = body.len(),
        body = %preview(body, TRACE_PREVIEW_CHARS),
        "received API response"
    );
}

/// Replaces credential values in place, at any depth of the payload.
fn redact(value: &mut Value) {
    match value {
        Value::Object(entries) => {
            for (key, entry) in entries.iter_mut() {
                if REDACTED_KEYS.contains(&key.as_str()) {
                    *entry = Value::String("[REDACTED]".to_owned());
                } else {
                    redact(entry);
                }
            }
        }
        Value::Array(items) => items.iter_mut().for_each(redact),
        _ => {}
    }
}

/// Embeds a body in an error message, where context matters more than brevity.
fn body_preview(body: &[u8]) -> String {
    const ERROR_PREVIEW_CHARS: usize = 512;
    preview(body, ERROR_PREVIEW_CHARS)
}

/// Truncates a body to `max_chars`, marking the cut with an ellipsis.
fn preview(body: &[u8], max_chars: usize) -> String {
    let text = String::from_utf8_lossy(body);
    let mut chars = text.chars();
    let head: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{head}...")
    } else {
        head
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct TestResponse {
        value: String,
    }

    #[test]
    fn redaction_removes_credentials_at_every_depth() {
        let mut value = serde_json::json!({
            "clientKey": "super-secret-key",
            "task": {
                "type": "TlsTask",
                "proxy": "http://user:hunter2@127.0.0.1:8080",
                "rounds": [{"proxy": "socks5://user:hunter2@host:1080"}]
            }
        });

        redact(&mut value);

        let rendered = value.to_string();
        assert!(!rendered.contains("super-secret-key"));
        assert!(!rendered.contains("hunter2"));
        assert_eq!(value["clientKey"], "[REDACTED]");
        assert_eq!(value["task"]["proxy"], "[REDACTED]");
        assert_eq!(value["task"]["rounds"][0]["proxy"], "[REDACTED]");
        assert_eq!(value["task"]["type"], "TlsTask");
    }

    #[test]
    fn a_non_json_error_body_still_reports_the_status_code()
    -> std::result::Result<(), Box<dyn std::error::Error>> {
        // A gateway or CDN can answer with HTML. The status code must stay
        // readable from the same place as a structured API error.
        let error = parse_api_response::<TestResponse>(503, b"<html>Service Unavailable</html>")
            .err()
            .ok_or_else(|| std::io::Error::other("expected an API error"))?;

        let Error::EzError(api_error) = error else {
            return Err(std::io::Error::other("expected a structured API error").into());
        };
        assert_eq!(api_error.http_status, 503);
        assert_eq!(api_error.error_code, None);
        assert!(
            api_error
                .error_description
                .as_deref()
                .is_some_and(|body| body.contains("Service Unavailable")),
            "the raw body must survive for diagnosis"
        );
        Ok(())
    }

    #[test]
    fn an_oversized_request_body_is_truncated_in_the_log() {
        // Akamai's script_base64 and DataDome's html_b64 run to megabytes, and
        // logging one whole makes the line unreadable.
        let rendered = preview(&vec![b'x'; 10_000], TRACE_PREVIEW_CHARS);

        assert_eq!(rendered.chars().count(), TRACE_PREVIEW_CHARS + 3);
        assert!(rendered.ends_with("..."));
    }

    #[test]
    fn parses_successful_response() -> crate::Result<()> {
        let response = parse_api_response::<TestResponse>(200, br#"{"errorId":0,"value":"ok"}"#)?;
        assert_eq!(
            response,
            TestResponse {
                value: "ok".to_owned()
            }
        );
        Ok(())
    }

    #[test]
    fn preserves_structured_api_error() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let error = parse_api_response::<TestResponse>(
            400,
            br#"{
                "errorId":1,
                "errorCode":"ERROR_REQUEST_PARAMETERS",
                "errorDescription":"Invalid parameters",
                "requestId":"request-1",
                "errors":{"task.websiteKey":"Must not be blank"}
            }"#,
        )
        .err()
        .ok_or_else(|| std::io::Error::other("expected an API error"))?;

        let Error::EzError(api_error) = error else {
            return Err(std::io::Error::other("expected a structured API error").into());
        };
        assert_eq!(api_error.task_id, None);
        assert_eq!(api_error.request_id.as_deref(), Some("request-1"));
        assert_eq!(
            api_error.error_code.as_deref(),
            Some("ERROR_REQUEST_PARAMETERS")
        );
        assert_eq!(
            api_error.errors.get("task.websiteKey"),
            Some(&"Must not be blank".to_owned())
        );
        Ok(())
    }
}
