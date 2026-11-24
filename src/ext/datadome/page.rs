//! Reading the pages of a DataDome challenge.
//!
//! The script hash is the key the service caches challenge scripts under, so
//! the extraction and hashing rules here have to match the service's exactly.
//! The tests at the bottom pin them to fixed vectors.

use std::{fmt::Write as _, sync::LazyLock};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use md5::{Digest as _, Md5};
use regex::Regex;
use url::Url;

/// Text DataDome puts on the page of a visitor it has banned outright.
const HARD_BLOCK_MARKER: &str = "hardblock";
/// Text every page DataDome intercepted carries.
const CHALLENGE_MARKER: &str = "var dd";
/// Path segment of a slider captcha address; the interstitial uses another.
const SLIDER_PATH: &str = "/captcha/";
/// What closes the invisible interstitial bundle.
const BUNDLE_CLOSE: &str = "})()";
/// CDN scripts that bootstrap a challenge but are not its bundle.
const LOADER_SCRIPTS: [&str; 2] = ["i.js", "c.js"];

/// Every `<script>` element: its opening tag, then its body.
static SCRIPT_ELEMENT: LazyLock<Regex> =
    LazyLock::new(|| pattern(r"(?is)(<script\b[^>]*>)(.*?)</script>"));
/// A `src` attribute, which marks an opening tag as an external script.
static SRC_ATTRIBUTE: LazyLock<Regex> = LazyLock::new(|| pattern(r"\bsrc\s*="));
/// The version banner, `/* DataDome ... */` or `/** DataDome ... */`.
static BANNER: LazyLock<Regex> = LazyLock::new(|| pattern(r"/\*+\s*DataDome[^*]*\*/"));
/// How the interstitial bundle opens, right after its banner.
static INTERSTITIAL_OPENING: LazyLock<Regex> =
    LazyLock::new(|| pattern(r"^\s*;\s*\(function\s*\(\)\s*\{"));
/// Where a bundle starts when it carries no banner: the slider or the interstitial form.
static BUNDLE_START: LazyLock<Regex> =
    LazyLock::new(|| pattern(r"!\s*function\s+[A-Za-z_$][\w$]*\s*\(|;\s*\(function\s*\(\)\s*\{"));
/// An external challenge script with a descriptive name.
static NAMED_EXTERNAL_SCRIPT: LazyLock<Regex> = LazyLock::new(|| {
    pattern(
        r#"(?i)<script\b[^>]*?\bsrc\s*=\s*["']((?:https:)?//ct\.captcha-delivery\.com/(?:interstitial|captcha)\.[^"']+?\.js)["']"#,
    )
});
/// An external challenge script named by its content hash, which DataDome
/// switched to in November 2026.
static HASHED_EXTERNAL_SCRIPT: LazyLock<Regex> = LazyLock::new(|| {
    pattern(
        r#"(?i)<script\b[^>]*?\bsrc\s*=\s*["']((?:https:)?//ct\.captcha-delivery\.com/[A-Za-z0-9][A-Za-z0-9._-]*?\.js)["']"#,
    )
});

/// Compiles one of the patterns above.
#[expect(
    clippy::expect_used,
    reason = "the patterns are literals in this file, and its tests compile every one"
)]
fn pattern(source: &str) -> Regex {
    Regex::new(source).expect("DataDome page patterns are valid")
}

/// Returns true when DataDome banned the visitor outright.
///
/// No challenge clears that; only a different IP helps.
pub(crate) fn is_hard_blocked(html: &str) -> bool {
    html.contains(HARD_BLOCK_MARKER)
}

/// Returns true when DataDome intercepted the page with a challenge.
pub(crate) fn is_challenged(html: &str) -> bool {
    html.contains(CHALLENGE_MARKER)
}

/// Returns true when a challenge address leads to the slider captcha rather
/// than to the invisible interstitial.
pub(crate) fn is_slider(challenge_url: &Url) -> bool {
    challenge_url.path().contains(SLIDER_PATH)
}

/// Base64-encodes page or script text the way the service expects it.
pub(crate) fn encode_text(text: &str) -> String {
    STANDARD.encode(text)
}

/// Strips the version banner DataDome puts on the first line of its script.
///
/// The banner changes with every release while the code under it may not, so
/// it stays out of the hash. Only a first line that opens and closes a comment
/// counts as a banner; anything else is returned unchanged.
pub(crate) fn strip_banner(script: &str) -> &str {
    if !script.starts_with("/*") {
        return script;
    }
    let first_line_end = script.find('\n').map_or(script.len(), |index| index + 1);
    let (first_line, rest) = script.split_at(first_line_end);
    if first_line.contains("*/") {
        rest
    } else {
        script
    }
}

/// Content-addressed identifier of a challenge script: the lower-case hex MD5
/// of the script without its banner.
pub(crate) fn script_hash(script: &str) -> String {
    let digest = Md5::digest(strip_banner(script).as_bytes());
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest.iter() {
        // Writing to a String cannot fail.
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}

/// Address of the challenge script when the page loads it from DataDome's CDN.
///
/// A descriptively named script wins; otherwise the first hash-named one that
/// is not a loader. A protocol-relative address is completed with `https:`.
pub(crate) fn external_script_url(html: &str) -> Option<String> {
    if let Some(src) = NAMED_EXTERNAL_SCRIPT
        .captures(html)
        .and_then(|captures| captures.get(1))
    {
        return Some(absolute_src(src.as_str()));
    }
    HASHED_EXTERNAL_SCRIPT
        .captures_iter(html)
        .filter_map(|captures| captures.get(1))
        .map(|src| src.as_str())
        .find(|src| !is_loader(src))
        .map(absolute_src)
}

/// Returns true for a CDN script that only bootstraps a challenge.
fn is_loader(src: &str) -> bool {
    let file_name = src.rsplit('/').next().unwrap_or(src).to_ascii_lowercase();
    LOADER_SCRIPTS.contains(&file_name.as_str())
}

/// Completes a protocol-relative script address with `https:`.
fn absolute_src(src: &str) -> String {
    if src.starts_with("//") {
        format!("https:{src}")
    } else {
        src.to_owned()
    }
}

/// The challenge script embedded in the page, cut down to the part that is the
/// same for every session.
///
/// The bundle is the longest inline script by far, and it is cut from its
/// version banner on. An interstitial bundle also ends at its closing `})()`:
/// what follows is page glue that carries session-unique identifiers, and
/// hashing those would never hit the cache. A slider bundle keeps everything to
/// the end of its script. Without a banner, the bundle runs from where its code
/// starts.
pub(crate) fn inline_script(html: &str) -> Option<&str> {
    let script = longest_inline_script(html)?;
    let Some(banner) = BANNER.find(script) else {
        let start = BUNDLE_START.find(script).map_or(0, |found| found.start());
        return Some(&script[start..]);
    };
    let bundle = &script[banner.start()..];
    let after_banner = &bundle[banner.len()..];
    if INTERSTITIAL_OPENING.is_match(after_banner)
        && let Some(close) = bundle.rfind(BUNDLE_CLOSE)
    {
        return Some(&bundle[..close + BUNDLE_CLOSE.len()]);
    }
    Some(bundle)
}

/// The longest non-empty body among the scripts without a `src` attribute;
/// the first one wins a tie.
fn longest_inline_script(html: &str) -> Option<&str> {
    let mut longest = "";
    for element in SCRIPT_ELEMENT.captures_iter(html) {
        let opening_tag = element.get(1).map_or("", |tag| tag.as_str());
        if SRC_ATTRIBUTE.is_match(opening_tag) {
            continue;
        }
        let body = element.get(2).map_or("", |body| body.as_str());
        if body.len() > longest.len() {
            longest = body;
        }
    }
    (!longest.is_empty()).then_some(longest)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BANNER_LINE: &str = "/* DataDome is a cyberfraud solution to detect bot activity @license https://datadome.co v1.33.1 d5110df */";
    const INTERSTITIAL_BODY: &str = ";(function(){var A={};})();";

    #[test]
    fn every_pattern_compiles() {
        for compiled in [
            &SCRIPT_ELEMENT,
            &SRC_ATTRIBUTE,
            &BANNER,
            &INTERSTITIAL_OPENING,
            &BUNDLE_START,
            &NAMED_EXTERNAL_SCRIPT,
            &HASHED_EXTERNAL_SCRIPT,
        ] {
            LazyLock::force(compiled);
        }
    }

    #[test]
    fn only_a_closed_first_line_comment_is_a_banner() {
        let with_banner = format!("{BANNER_LINE}\n{INTERSTITIAL_BODY}");
        assert_eq!(strip_banner(&with_banner), INTERSTITIAL_BODY);
        assert_eq!(strip_banner(INTERSTITIAL_BODY), INTERSTITIAL_BODY);

        let code_first = format!("var x=1;\n{INTERSTITIAL_BODY}");
        assert_eq!(strip_banner(&code_first), code_first);
        assert_eq!(strip_banner("/* 未闭合\nvar x=1;"), "/* 未闭合\nvar x=1;");
        assert_eq!(strip_banner(BANNER_LINE), "");
    }

    #[test]
    fn the_hash_is_the_md5_of_the_script_without_its_banner() {
        // RFC 1321 test vector.
        assert_eq!(script_hash(""), "d41d8cd98f00b204e9800998ecf8427e");
        assert_eq!(
            script_hash(&format!("{BANNER_LINE}\n{INTERSTITIAL_BODY}")),
            script_hash(INTERSTITIAL_BODY)
        );
    }

    #[test]
    fn an_interstitial_bundle_ends_before_the_session_glue() {
        let html = format!(
            "<html><head>\
             <script src=\"https://ct.captcha-delivery.com/interstitial.1.33.0.x.js\"></script>\
             <script>var small=1;</script>\
             <script>var ddm={{cid:'session-unique'}};\n{BANNER_LINE}\n{INTERSTITIAL_BODY}\n\
             var glue='initialCid=session-unique';</script>\
             </head></html>"
        );
        let expected = format!("{BANNER_LINE}\n{INTERSTITIAL_BODY}");
        assert_eq!(inline_script(&html), Some(expected.trim_end_matches(';')));
        assert_eq!(
            external_script_url(&html).as_deref(),
            Some("https://ct.captcha-delivery.com/interstitial.1.33.0.x.js")
        );
    }

    #[test]
    fn a_slider_bundle_keeps_its_static_glue() {
        let html = "<script>var ddm={};\n/** DataDome v1.34.0 96eb15b */\n!function A(){}();\nvar g=1;</script>";
        assert_eq!(
            inline_script(html),
            Some("/** DataDome v1.34.0 96eb15b */\n!function A(){}();\nvar g=1;")
        );
    }

    #[test]
    fn a_bundle_without_a_banner_starts_where_its_code_does() {
        assert_eq!(inline_script("<script>var a=1;</script>"), Some("var a=1;"));
        assert_eq!(
            inline_script("<script>var a=1;\n!function B(){}();</script>"),
            Some("!function B(){}();")
        );
        assert_eq!(inline_script("<script src=\"x.js\"></script>"), None);
        assert_eq!(inline_script("<p>no scripts</p>"), None);
    }

    #[test]
    fn a_hash_named_external_script_is_found_past_the_loaders() {
        let html = "<script src=\"https://ct.captcha-delivery.com/i.js\"></script>\
                    <script src=\"//ct.captcha-delivery.com/5d512141a346e3bf.js\"></script>";
        assert_eq!(
            external_script_url(html).as_deref(),
            Some("https://ct.captcha-delivery.com/5d512141a346e3bf.js")
        );
        assert_eq!(
            external_script_url("<script src=\"https://ct.captcha-delivery.com/c.js\"></script>"),
            None
        );
    }

    #[test]
    fn pages_are_classified_by_their_markers() -> Result<(), url::ParseError> {
        assert!(is_hard_blocked("<p>hardblock</p>"));
        assert!(is_challenged("<script>var dd={'cid':'x'}</script>"));
        assert!(!is_challenged("<html>welcome</html>"));
        assert!(is_slider(&Url::parse(
            "https://geo.captcha-delivery.com/captcha/?initialCid=x"
        )?));
        assert!(!is_slider(&Url::parse(
            "https://geo.captcha-delivery.com/interstitial/?initialCid=x"
        )?));
        Ok(())
    }
}
