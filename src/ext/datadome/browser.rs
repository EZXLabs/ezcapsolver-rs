//! The browser a challenge run presents itself as.
//!
//! DataDome compares what each request claims against how it is sent, so
//! every header set here mirrors what Chrome sends for that kind of request,
//! and the TLS fingerprint follows the Chrome version the user agent names.

use lkrequest::{ClientPreset, preset};
use url::Url;

use crate::{Error, Result};

/// `Accept` of a document or frame navigation.
const DOCUMENT_ACCEPT: &str = "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8";
/// `Accept-Encoding` Chrome sends.
const ACCEPT_ENCODING: &str = "gzip, deflate, br, zstd";
/// Origin of the page a POST submission comes from.
const CHALLENGE_ORIGIN: &str = "https://geo.captcha-delivery.com";
/// Content type of a POST submission.
const FORM_CONTENT_TYPE: &str = "application/x-www-form-urlencoded; charset=UTF-8";
/// Where the Chrome version starts in a user agent.
const CHROME_TOKEN: &str = "Chrome/";

/// Request headers, in the order they are added.
pub(crate) type Headers = Vec<(&'static str, String)>;

/// Browser identity every request of one challenge run shares.
#[derive(Debug, Clone)]
pub(crate) struct BrowserProfile {
    user_agent: String,
    accept_language: String,
    chrome_major: u16,
}

impl BrowserProfile {
    /// Builds the profile, rejecting a user agent that names no Chrome version.
    pub(crate) fn new(user_agent: &str, accept_language: &str) -> Result<Self> {
        let chrome_major = chrome_major(user_agent).ok_or_else(|| {
            Error::DataDome(format!(
                "user_agent must name a Chrome version, got `{user_agent}`"
            ))
        })?;
        Ok(Self {
            user_agent: user_agent.to_owned(),
            accept_language: accept_language.to_owned(),
            chrome_major,
        })
    }

    /// TLS and HTTP/2 fingerprint of the Chrome version the user agent names.
    pub(crate) fn tls_preset(&self) -> Result<ClientPreset> {
        chrome_preset(self.chrome_major).ok_or_else(|| {
            Error::DataDome(format!(
                "no TLS fingerprint for Chrome {}; use 131, or 144 to 158",
                self.chrome_major
            ))
        })
    }

    /// Headers of the first visit to the protected page.
    pub(crate) fn navigation(&self) -> Headers {
        let mut headers = self.device();
        headers.extend([
            ("accept", DOCUMENT_ACCEPT.to_owned()),
            ("upgrade-insecure-requests", "1".to_owned()),
            ("sec-fetch-site", "none".to_owned()),
            ("sec-fetch-mode", "navigate".to_owned()),
            ("sec-fetch-user", "?1".to_owned()),
            ("sec-fetch-dest", "document".to_owned()),
        ]);
        headers
    }

    /// Headers of loading the challenge into its frame.
    ///
    /// The slider frame gets the full page address as its referer; the
    /// interstitial only its origin, as a real visitor's browser sends them.
    pub(crate) fn challenge_frame(&self, target: &Url, slider: bool) -> Headers {
        let referer = if slider {
            target.as_str().to_owned()
        } else {
            origin(target)
        };
        let mut headers = self.device();
        headers.extend([
            ("accept", DOCUMENT_ACCEPT.to_owned()),
            ("upgrade-insecure-requests", "1".to_owned()),
            ("sec-fetch-site", "cross-site".to_owned()),
            ("sec-fetch-mode", "navigate".to_owned()),
            ("sec-fetch-dest", "iframe".to_owned()),
            ("referer", referer),
        ]);
        headers
    }

    /// Headers of fetching the external challenge script.
    ///
    /// It has to look like a script load: the CDN rejects navigation headers.
    pub(crate) fn script(&self, challenge_url: &Url) -> Headers {
        let mut headers = self.device();
        headers.extend([
            ("accept", "*/*".to_owned()),
            ("sec-fetch-site", "same-site".to_owned()),
            ("sec-fetch-mode", "no-cors".to_owned()),
            ("sec-fetch-dest", "script".to_owned()),
            ("referer", format!("{}/", origin(challenge_url))),
        ]);
        headers
    }

    /// Headers of submitting the answer from the challenge frame.
    pub(crate) fn submission(&self, challenge_url: &Url, post: bool) -> Headers {
        let mut headers = self.device();
        if post {
            headers.extend([
                ("content-type", FORM_CONTENT_TYPE.to_owned()),
                ("origin", CHALLENGE_ORIGIN.to_owned()),
            ]);
        }
        headers.extend([
            ("accept", "*/*".to_owned()),
            ("sec-fetch-site", "same-origin".to_owned()),
            ("sec-fetch-mode", "cors".to_owned()),
            ("sec-fetch-dest", "empty".to_owned()),
            ("referer", challenge_url.as_str().to_owned()),
        ]);
        headers
    }

    /// Headers every request carries: the client hints and the identity.
    fn device(&self) -> Headers {
        vec![
            ("sec-ch-ua", self.sec_ch_ua()),
            ("sec-ch-ua-mobile", "?0".to_owned()),
            ("sec-ch-ua-platform", format!("\"{}\"", self.platform())),
            ("user-agent", self.user_agent.clone()),
            ("accept-encoding", ACCEPT_ENCODING.to_owned()),
            ("accept-language", self.accept_language.clone()),
        ]
    }

    /// `Sec-CH-UA` brand list matching the Chrome version.
    fn sec_ch_ua(&self) -> String {
        let version = self.chrome_major;
        format!(
            "\"Chromium\";v=\"{version}\", \"Google Chrome\";v=\"{version}\", \"Not/A)Brand\";v=\"99\""
        )
    }

    /// `Sec-CH-UA-Platform` value matching the operating system in the user agent.
    fn platform(&self) -> &'static str {
        if self.user_agent.contains("Macintosh") {
            "macOS"
        } else if self.user_agent.contains("Linux") {
            "Linux"
        } else {
            "Windows"
        }
    }
}

/// Chrome major version named in a user agent.
fn chrome_major(user_agent: &str) -> Option<u16> {
    let start = user_agent.find(CHROME_TOKEN)? + CHROME_TOKEN.len();
    let rest = &user_agent[start..];
    let end = rest
        .find(|character: char| !character.is_ascii_digit())
        .unwrap_or(rest.len());
    rest[..end].parse().ok()
}

/// lkrequest fingerprint preset for a Chrome major version.
fn chrome_preset(major: u16) -> Option<ClientPreset> {
    let preset = match major {
        131 => preset::chrome_131(),
        144 => preset::chrome_144(),
        145 => preset::chrome_145(),
        146 => preset::chrome_146(),
        147 => preset::chrome_147(),
        148 => preset::chrome_148(),
        149 => preset::chrome_149(),
        150 => preset::chrome_150(),
        151 => preset::chrome_151(),
        152 => preset::chrome_152(),
        153 => preset::chrome_153(),
        154 => preset::chrome_154(),
        155 => preset::chrome_155(),
        156 => preset::chrome_156(),
        157 => preset::chrome_157(),
        158 => preset::chrome_158(),
        _ => return None,
    };
    Some(preset)
}

/// `scheme://host[:port]` of an address, the form an origin-only referer takes.
fn origin(url: &Url) -> String {
    url.origin().ascii_serialization()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Result of a test that mixes the crate error with parse errors.
    type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

    const WINDOWS_CHROME_153: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/153.0.0.0 Safari/537.36";

    /// Looks a header up by name.
    fn header<'a>(headers: &'a Headers, name: &str) -> Option<&'a str> {
        headers
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| value.as_str())
    }

    #[test]
    fn the_chrome_version_comes_from_the_user_agent() {
        assert_eq!(chrome_major(WINDOWS_CHROME_153), Some(153));
        assert_eq!(chrome_major("Chrome/99"), Some(99));
        assert_eq!(chrome_major("Mozilla/5.0 Firefox/133.0"), None);
        assert_eq!(chrome_major("Chrome/x"), None);
    }

    #[test]
    fn only_versions_with_a_preset_are_accepted() -> Result<()> {
        let supported = BrowserProfile::new(WINDOWS_CHROME_153, "en")?;
        assert!(supported.tls_preset().is_ok());

        let unsupported = BrowserProfile::new("Chrome/120.0.0.0", "en")?;
        assert!(matches!(unsupported.tls_preset(), Err(Error::DataDome(_))));
        assert!(matches!(
            BrowserProfile::new("Firefox/133.0", "en"),
            Err(Error::DataDome(_))
        ));
        Ok(())
    }

    #[test]
    fn client_hints_follow_the_user_agent() -> Result<()> {
        let windows = BrowserProfile::new(WINDOWS_CHROME_153, "en-US,en;q=0.9")?;
        let headers = windows.navigation();
        assert_eq!(
            header(&headers, "sec-ch-ua"),
            Some("\"Chromium\";v=\"153\", \"Google Chrome\";v=\"153\", \"Not/A)Brand\";v=\"99\"")
        );
        assert_eq!(header(&headers, "sec-ch-ua-platform"), Some("\"Windows\""));
        assert_eq!(header(&headers, "user-agent"), Some(WINDOWS_CHROME_153));
        assert_eq!(header(&headers, "accept-language"), Some("en-US,en;q=0.9"));

        let mac = BrowserProfile::new(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) Chrome/153.0.0.0",
            "en",
        )?;
        assert_eq!(
            header(&mac.navigation(), "sec-ch-ua-platform"),
            Some("\"macOS\"")
        );
        Ok(())
    }

    #[test]
    fn each_request_carries_the_referer_a_browser_would() -> TestResult {
        let profile = BrowserProfile::new(WINDOWS_CHROME_153, "en")?;
        let target = Url::parse("https://shop.example.com/signup?next=1")?;
        let challenge = Url::parse("https://geo.captcha-delivery.com/captcha/?initialCid=x")?;

        let slider = profile.challenge_frame(&target, true);
        assert_eq!(
            header(&slider, "referer"),
            Some("https://shop.example.com/signup?next=1")
        );
        let interstitial = profile.challenge_frame(&target, false);
        assert_eq!(
            header(&interstitial, "referer"),
            Some("https://shop.example.com")
        );
        assert_eq!(
            header(&profile.script(&challenge), "referer"),
            Some("https://geo.captcha-delivery.com/")
        );
        Ok(())
    }

    #[test]
    fn only_a_post_submission_carries_a_form_content_type() -> TestResult {
        let profile = BrowserProfile::new(WINDOWS_CHROME_153, "en")?;
        let challenge = Url::parse("https://geo.captcha-delivery.com/interstitial/?x=1")?;

        let post = profile.submission(&challenge, true);
        assert_eq!(header(&post, "content-type"), Some(FORM_CONTENT_TYPE));
        assert_eq!(header(&post, "origin"), Some(CHALLENGE_ORIGIN));
        assert_eq!(header(&post, "referer"), Some(challenge.as_str()));

        let get = profile.submission(&challenge, false);
        assert_eq!(header(&get, "content-type"), None);
        assert_eq!(header(&get, "origin"), None);
        Ok(())
    }
}
