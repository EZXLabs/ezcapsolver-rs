//! The DataDome challenge workflow on the async client.

use std::time::{Duration, Instant};

use lkrequest::{Session, session::RequestBuilder};
use tracing::{debug, info};
use url::Url;

use super::{
    DataDomeChallenge, DataDomeOutcome,
    browser::{BrowserProfile, Headers},
    page::{
        encode_text, external_script_url, inline_script, is_challenged, is_hard_blocked, is_slider,
        script_hash,
    },
    verdict::{Submission, read_verdict, submit_delay},
};
use crate::{AsyncEzCapSolverClient, DataDomeSolution, DataDomeStep, DataDomeTask, Error, Result};

/// Time limit of each browser-side request.
const BROWSER_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

impl AsyncEzCapSolverClient {
    /// Clears the DataDome challenge in front of a page and returns the cookie
    /// DataDome issues for it.
    ///
    /// Only the page address and a browser identity are needed; the whole flow
    /// runs inside this call:
    ///
    /// 1. Fetch the target page. A page DataDome did not intercept ends the run
    ///    with [`DataDomeOutcome::NotTriggered`].
    /// 2. Step 1 finds the challenge address, and the challenge page is fetched.
    /// 3. The challenge script is read from that page, or fetched from
    ///    DataDome's CDN when the page links it.
    /// 4. Step 2 asks whether the service already caches the script.
    /// 5. Step 3 computes the answer, sending the whole script only when it is
    ///    not cached.
    /// 6. The answer is submitted after the delay it asks for, and the cookie in
    ///    DataDome's response is returned.
    ///
    /// Steps 1 to 3 are `DataDomeTaskProxyless` tasks on the synchronous
    /// endpoint, so a challenged page costs three tasks. Browser-side requests
    /// go through [`DataDomeChallenge::proxy`] with the Chrome TLS fingerprint
    /// of [`DataDomeChallenge::user_agent`].
    ///
    /// # Errors
    ///
    /// - [`Error::DataDomeBlocked`] when DataDome banned the IP; retry through
    ///   a different proxy.
    /// - [`Error::DataDome`] when the flow cannot go on: an unusable
    ///   `target_url` or `user_agent`, a challenge without a script, or a
    ///   submission DataDome did not accept.
    /// - [`Error::BrowserRequest`] when a request to the protected site or to
    ///   DataDome fails.
    /// - Any error of the API calls, as from
    ///   [`Self::sync_solve_data_dome_task_proxyless`].
    pub async fn solve_data_dome_workflow(
        &self,
        challenge: &DataDomeChallenge,
    ) -> Result<DataDomeOutcome> {
        let target = parse_url(&challenge.target_url, "target_url")?;
        let browser = Browser::open(challenge)?;

        let page = browser
            .get(target.as_str(), browser.profile.navigation())
            .await?;
        if is_hard_blocked(&page) {
            return Err(blocked("the target page"));
        }
        if !is_challenged(&page) {
            info!("datadome did not intercept the target page");
            return Ok(DataDomeOutcome::NotTriggered);
        }

        let challenge_url = self
            .data_dome_challenge_url(&page, &target, &challenge.user_agent)
            .await?;
        let slider = is_slider(&challenge_url);
        debug!(slider, "datadome challenge located");
        let frame_headers = browser.profile.challenge_frame(&target, slider);
        let challenge_page = browser.get(challenge_url.as_str(), frame_headers).await?;
        if is_hard_blocked(&challenge_page) {
            return Err(blocked("the challenge page"));
        }
        let loaded_at = Instant::now();

        let script = browser
            .challenge_script(&challenge_page, &challenge_url)
            .await?;
        let answer = self
            .data_dome_answer(
                &challenge_page,
                &challenge_url,
                &challenge.user_agent,
                &script,
            )
            .await?;
        let submission = Submission::from_answer(&answer)?;
        let wait = submit_delay(&answer, &submission).saturating_sub(loaded_at.elapsed());
        debug!(?wait, "waiting before submitting the datadome answer");
        tokio::time::sleep(wait).await;

        let verdict = browser.submit(&submission, &challenge_url).await?;
        let cookie = read_verdict(&verdict, &target)?;
        info!(name = cookie.name, "datadome challenge passed");
        Ok(DataDomeOutcome::Passed(cookie))
    }

    /// Step 1: asks the service where the challenge of an intercepted page lives.
    async fn data_dome_challenge_url(
        &self,
        page: &str,
        target: &Url,
        user_agent: &str,
    ) -> Result<Url> {
        let task = DataDomeTask::builder()
            .step(DataDomeStep::One)
            .html_b64(encode_text(page))
            .referer(target.as_str())
            .user_agent(user_agent)
            .build();
        let answer = self
            .sync_solve_data_dome_task_proxyless(&task)
            .await?
            .solution;
        let url = answer
            .url
            .ok_or_else(|| Error::DataDome("step 1 returned no challenge url".to_owned()))?;
        parse_url(&url, "the step 1 challenge url")
    }

    /// Steps 2 and 3: asks whether the service caches the script, then for the
    /// answer, sending the script itself only when it is not cached.
    async fn data_dome_answer(
        &self,
        challenge_page: &str,
        challenge_url: &Url,
        user_agent: &str,
        script: &str,
    ) -> Result<DataDomeSolution> {
        let hash = script_hash(script);
        let cached = self.data_dome_script_cached(&hash).await?;
        debug!(hash, cached, "probed the datadome script cache");
        let (script_hash, script_base64) = if cached {
            (Some(hash), None)
        } else {
            (None, Some(encode_text(script)))
        };
        let task = DataDomeTask::builder()
            .step(DataDomeStep::Three)
            .html_b64(encode_text(challenge_page))
            .referer(challenge_url.as_str())
            .user_agent(user_agent)
            .maybe_external_js_hash(script_hash)
            .maybe_external_js_base64(script_base64)
            .build();
        Ok(self
            .sync_solve_data_dome_task_proxyless(&task)
            .await?
            .solution)
    }

    /// Step 2: whether the service already caches the script with this hash.
    ///
    /// An answer without `status` counts as not cached, which only means the
    /// script is sent in full.
    async fn data_dome_script_cached(&self, hash: &str) -> Result<bool> {
        let task = DataDomeTask::builder()
            .step(DataDomeStep::Two)
            .hash(hash)
            .build();
        let answer = self
            .sync_solve_data_dome_task_proxyless(&task)
            .await?
            .solution;
        Ok(answer.status.unwrap_or(false))
    }
}

/// Browser side of a challenge run: one cookie-keeping session with a Chrome
/// TLS fingerprint, and the identity its requests present.
struct Browser {
    session: Session,
    profile: BrowserProfile,
}

impl Browser {
    /// Opens the session a challenge run goes through.
    fn open(challenge: &DataDomeChallenge) -> Result<Self> {
        let profile = BrowserProfile::new(&challenge.user_agent, &challenge.accept_language)?;
        let client = lkrequest::Client::builder()
            .preset(profile.tls_preset()?)
            .total_timeout(BROWSER_REQUEST_TIMEOUT)
            .build();
        let mut session = client.session();
        if let Some(proxy) = &challenge.proxy {
            session = session.proxy(proxy);
        }
        Ok(Self {
            session: session.build(),
            profile,
        })
    }

    /// Sends a GET and returns the response body.
    ///
    /// The status is not checked: DataDome answers an intercepted page with an
    /// error status, and the body is what tells the cases apart.
    async fn get(&self, url: &str, headers: Headers) -> Result<String> {
        send(with_headers(self.session.get(url), headers)).await
    }

    /// Sends a form POST and returns the response body.
    async fn post(&self, url: &str, headers: Headers, body: String) -> Result<String> {
        send(with_headers(self.session.post(url), headers).body(body)).await
    }

    /// The challenge script: fetched from the CDN when the page links it, read
    /// from the page otherwise.
    async fn challenge_script(&self, page: &str, challenge_url: &Url) -> Result<String> {
        let Some(src) = external_script_url(page) else {
            debug!("reading the inline datadome script");
            return inline_script(page).map(str::to_owned).ok_or_else(|| {
                Error::DataDome("the challenge page carries no challenge script".to_owned())
            });
        };
        debug!(src, "fetching the external datadome script");
        let script = self.get(&src, self.profile.script(challenge_url)).await?;
        if script.trim().is_empty() {
            return Err(Error::DataDome(
                "the external challenge script is empty".to_owned(),
            ));
        }
        Ok(script)
    }

    /// Hands the answer to DataDome and returns its response body.
    async fn submit(&self, submission: &Submission, challenge_url: &Url) -> Result<String> {
        match submission {
            Submission::Get { url } => {
                self.get(url, self.profile.submission(challenge_url, false))
                    .await
            }
            Submission::Post { url, body } => {
                self.post(
                    url,
                    self.profile.submission(challenge_url, true),
                    body.clone(),
                )
                .await
            }
        }
    }
}

/// Adds headers to a request in order.
fn with_headers(request: RequestBuilder, headers: Headers) -> RequestBuilder {
    headers.iter().fold(request, |request, (name, value)| {
        request.header(name, value)
    })
}

/// Sends a browser-side request and decodes its body leniently.
async fn send(request: RequestBuilder) -> Result<String> {
    let response = request
        .send()
        .await
        .map_err(|error| Error::BrowserRequest(Box::new(error)))?;
    Ok(String::from_utf8_lossy(response.bytes()).into_owned())
}

/// Parses an address the flow has to request, naming it in the error.
fn parse_url(url: &str, what: &str) -> Result<Url> {
    Url::parse(url)
        .map_err(|error| Error::DataDome(format!("{what} `{url}` is not a valid URL: {error}")))
}

/// The error for a page DataDome serves to a banned visitor.
fn blocked(what: &str) -> Error {
    Error::DataDomeBlocked(format!("{what} is a block page"))
}
