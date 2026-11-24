//! Handing the step-3 answer to DataDome, and reading the cookie it answers with.

use std::time::Duration;

use serde_json::Value;
use url::Url;

use super::DataDomeCookie;
use crate::{DataDomeSolution, Error, Result};

/// How long a GET submission waits when the answer names no delay.
const DEFAULT_GET_DELAY: Duration = Duration::from_secs(20);
/// Challenge kind whose answer always goes out as a GET.
const SLIDER_KIND: &str = "slider";
/// `view` of a submission response that shows the captcha again.
const CAPTCHA_VIEW: &str = "captcha";
/// Query value of `t` that marks a banned visitor.
const BANNED_MARKER: &str = "bv";
/// How much of an unreadable response an error message quotes.
const PREVIEW_CHARS: usize = 200;

/// The request that hands the step-3 answer to DataDome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Submission {
    /// A slider answer, or one without a body.
    Get {
        /// Submission address.
        url: String,
    },
    /// Any other answer, posted as a form.
    Post {
        /// Submission address.
        url: String,
        /// Form body.
        body: String,
    },
}

impl Submission {
    /// Reads the submission out of a step-3 answer.
    pub(crate) fn from_answer(answer: &DataDomeSolution) -> Result<Self> {
        let url = answer
            .url
            .clone()
            .filter(|url| !url.is_empty())
            .ok_or_else(|| Error::DataDome("step 3 returned no submission url".to_owned()))?;
        let slider = answer
            .kind
            .as_deref()
            .is_some_and(|kind| kind.contains(SLIDER_KIND));
        match answer.body.as_deref() {
            Some(body) if !slider && !body.is_empty() => Ok(Self::Post {
                url,
                body: body.to_owned(),
            }),
            _ => Ok(Self::Get { url }),
        }
    }
}

/// How long to wait, counted from when the challenge page loaded, before
/// submitting.
///
/// The answer's `submit_after_ms` decides; when it is absent or zero, a GET
/// waits [`DEFAULT_GET_DELAY`] and a POST goes out at once.
pub(crate) fn submit_delay(answer: &DataDomeSolution, submission: &Submission) -> Duration {
    match answer.submit_after_ms.filter(|millis| *millis > 0) {
        Some(millis) => Duration::from_millis(u64::from(millis)),
        None if matches!(submission, Submission::Get { .. }) => DEFAULT_GET_DELAY,
        None => Duration::ZERO,
    }
}

/// Reads DataDome's response to a submission and returns the cookie it issued.
///
/// A response without a cookie is a failed run: a ban when DataDome shows the
/// captcha again with `t=bv`, a rejected answer otherwise.
pub(crate) fn read_verdict(body: &str, target: &Url) -> Result<DataDomeCookie> {
    let verdict: Value = serde_json::from_str(body).map_err(|_| {
        Error::DataDome(format!(
            "the submission response is not JSON: {}",
            preview(body)
        ))
    })?;
    match text_field(&verdict, "cookie") {
        "" => Err(rejection(&verdict)),
        cookie => Ok(parse_cookie(cookie, target)),
    }
}

/// Explains why a submission brought back no cookie.
fn rejection(verdict: &Value) -> Error {
    let view = text_field(verdict, "view");
    let marker = query_value(text_field(verdict, "url"), "t");
    if view == CAPTCHA_VIEW && marker.as_deref() == Some(BANNED_MARKER) {
        Error::DataDomeBlocked("the submission was answered with a ban (t=bv)".to_owned())
    } else {
        Error::DataDome(format!("the challenge was not passed (view `{view}`)"))
    }
}

/// Splits DataDome's cookie string into its parts.
///
/// The domain falls back to the last two labels of the target host when the
/// cookie carries no `Domain` attribute.
fn parse_cookie(raw: &str, target: &Url) -> DataDomeCookie {
    let mut parts = raw.split(';');
    let pair = parts.next().unwrap_or_default();
    let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
    let domain = parts
        .map(str::trim)
        .find_map(|attribute| {
            attribute
                .strip_prefix("Domain=")
                .or_else(|| attribute.strip_prefix("domain="))
        })
        .map_or_else(
            || fallback_domain(target),
            |domain| domain.trim().to_owned(),
        );
    DataDomeCookie {
        name: name.trim().to_owned(),
        value: value.trim().to_owned(),
        domain,
        raw: raw.to_owned(),
    }
}

/// `.` plus the last two labels of the target host, such as `.example.com`.
fn fallback_domain(target: &Url) -> String {
    let host = target.host_str().unwrap_or_default();
    let labels: Vec<&str> = host.split('.').collect();
    format!(".{}", labels[labels.len().saturating_sub(2)..].join("."))
}

/// A string field of the verdict, empty when absent or not a string.
fn text_field<'a>(verdict: &'a Value, name: &str) -> &'a str {
    verdict
        .get(name)
        .and_then(Value::as_str)
        .unwrap_or_default()
}

/// First value of a query parameter, for an absolute or a relative address.
fn query_value(url: &str, name: &str) -> Option<String> {
    let query = url.split_once('?')?.1;
    let query = query.split('#').next().unwrap_or(query);
    url::form_urlencoded::parse(query.as_bytes())
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.into_owned())
}

/// The start of a response body, for an error message.
fn preview(body: &str) -> String {
    body.chars().take(PREVIEW_CHARS).collect()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    /// Result of a test that mixes the crate error with parse errors.
    type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

    /// A step-3 answer with the given fields.
    fn answer(
        url: Option<&str>,
        kind: Option<&str>,
        body: Option<&str>,
        submit_after_ms: Option<u32>,
    ) -> DataDomeSolution {
        DataDomeSolution {
            kind: kind.map(str::to_owned),
            url: url.map(str::to_owned),
            body: body.map(str::to_owned),
            status: None,
            submit_after_ms,
            extra: BTreeMap::new(),
        }
    }

    #[test]
    fn a_slider_or_bodyless_answer_is_submitted_as_a_get() -> Result<()> {
        let slider = answer(Some("https://dd/check"), Some("slider"), Some("x=1"), None);
        assert_eq!(
            Submission::from_answer(&slider)?,
            Submission::Get {
                url: "https://dd/check".to_owned()
            }
        );
        let bodyless = answer(Some("https://dd/check"), None, Some(""), None);
        assert!(matches!(
            Submission::from_answer(&bodyless)?,
            Submission::Get { .. }
        ));
        let interstitial = answer(
            Some("https://dd/i"),
            Some("interstitial"),
            Some("x=1"),
            None,
        );
        assert_eq!(
            Submission::from_answer(&interstitial)?,
            Submission::Post {
                url: "https://dd/i".to_owned(),
                body: "x=1".to_owned()
            }
        );
        assert!(matches!(
            Submission::from_answer(&answer(None, None, None, None)),
            Err(Error::DataDome(_))
        ));
        Ok(())
    }

    #[test]
    fn the_answer_decides_the_delay_with_a_default_for_gets() -> Result<()> {
        let timed = answer(Some("u"), Some("slider"), None, Some(1500));
        let submission = Submission::from_answer(&timed)?;
        assert_eq!(
            submit_delay(&timed, &submission),
            Duration::from_millis(1500)
        );

        let untimed_get = answer(Some("u"), Some("slider"), None, Some(0));
        let submission = Submission::from_answer(&untimed_get)?;
        assert_eq!(submit_delay(&untimed_get, &submission), DEFAULT_GET_DELAY);

        let untimed_post = answer(Some("u"), None, Some("x=1"), None);
        let submission = Submission::from_answer(&untimed_post)?;
        assert_eq!(submit_delay(&untimed_post, &submission), Duration::ZERO);
        Ok(())
    }

    #[test]
    fn the_cookie_is_split_into_its_parts() -> TestResult {
        let target = Url::parse("https://shop.example.com/signup")?;
        let raw = "datadome=abc123; Max-Age=31536000; Domain=.example.com; Path=/; Secure";
        let cookie = read_verdict(
            &format!(r#"{{"view":"redirect","cookie":"{raw}"}}"#),
            &target,
        )?;
        assert_eq!(
            cookie,
            DataDomeCookie {
                name: "datadome".to_owned(),
                value: "abc123".to_owned(),
                domain: ".example.com".to_owned(),
                raw: raw.to_owned(),
            }
        );

        let without_domain = read_verdict(r#"{"cookie":"datadome=v; Path=/"}"#, &target)?;
        assert_eq!(without_domain.domain, ".example.com");
        Ok(())
    }

    #[test]
    fn a_response_without_a_cookie_says_why() -> TestResult {
        let target = Url::parse("https://example.com")?;
        let banned =
            r#"{"view":"captcha","url":"https://geo.captcha-delivery.com/captcha/?t=bv&cid=x"}"#;
        assert!(matches!(
            read_verdict(banned, &target),
            Err(Error::DataDomeBlocked(_))
        ));
        let rejected =
            r#"{"view":"captcha","url":"https://geo.captcha-delivery.com/captcha/?t=fe"}"#;
        assert!(matches!(
            read_verdict(rejected, &target),
            Err(Error::DataDome(_))
        ));
        assert!(matches!(
            read_verdict("<html>oops</html>", &target),
            Err(Error::DataDome(_))
        ));
        Ok(())
    }
}
