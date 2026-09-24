<div align="center">
  <img src="./assets/ez-captcha-logo.svg" alt="EzCaptchaSolver" height="88">
  &nbsp;&nbsp;
  <img src="./assets/rust.svg" alt="Rust" height="88">
  <h1>EzCaptchaSolver Rust SDK</h1>
  <p>
    <a href="https://github.com/EZXLabs/ezcapsolver-rs/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/EZXLabs/ezcapsolver-rs/actions/workflows/ci.yml/badge.svg"></a>
    <a href="https://crates.io/crates/ezcapsolver-rs"><img alt="crates.io" src="https://img.shields.io/crates/v/ezcapsolver-rs.svg?logo=rust"></a>
    <a href="https://docs.rs/ezcapsolver-rs"><img alt="docs.rs" src="https://img.shields.io/docsrs/ezcapsolver-rs?logo=docsdotrs"></a>
    <a href="./LICENSE"><img alt="License: Apache-2.0" src="https://img.shields.io/badge/license-Apache--2.0-blue.svg"></a>
    <a href="https://www.rust-lang.org"><img alt="Rust 1.96.0+" src="https://img.shields.io/badge/rust-1.96.0%2B-orange.svg?logo=rust&logoColor=white"></a>
    <a href="https://ezxlabs.com"><img alt="Website" src="https://img.shields.io/badge/website-ezxlabs.com-FFDB29?logoColor=black"></a>
  </p>
  <p>
    <a href="https://ezxlabs.com">🌐 Website</a> &nbsp;·&nbsp;
    <a href="https://ezxlabs.com/en/docs/captcha/api">📚 API Docs</a> &nbsp;·&nbsp;
    <a href="./examples">🧪 Examples</a> &nbsp;·&nbsp;
    <a href="#-supported-captcha-types">🧩 Captcha Types</a>
  </p>
  <p><b>English</b> &nbsp;·&nbsp; <a href="./README.zh-CN.md">简体中文</a></p>
</div>

---

Integrate the [EzCaptchaSolver](https://ezxlabs.com) captcha solving service into your Rust program to automate solving captchas of any kind. Request samples for every captcha type are available in the [EzCaptchaSolver Docs](https://ezxlabs.com/en/docs/captcha/api).

## 🧩 Supported Captcha Types

Captcha task types come in a synchronous and an asynchronous form:

- Synchronous: the request blocks after the task is created and returns once the task is done.
- Asynchronous: creating the task returns a task ID, and the result is fetched later by polling that ID. This suits captcha types that take a while to solve.

A captcha type can support both forms at once, and almost every type supports the synchronous one. A few types are asynchronous only.

### reCAPTCHA v2

| Task type | Modes | Example | Description |
| :-: | :---: | :-: | --- |
| `ReCaptchaV2TaskProxyless` | all | [Example](#ReCaptchaV2TaskProxyless) | reCAPTCHA v2 |
| `ReCaptchaV2TaskProxylessS9` | all | [Example](#ReCaptchaV2TaskProxylessS9) | reCAPTCHA v2, returns a token scored ≥ 0.9 |
| `ReCaptchaV2STaskProxyless` | all | [Example](#ReCaptchaV2STaskProxyless) | reCAPTCHA v2 carrying the challenge-bound `s` parameter |
| `ReCaptchaV2EnterpriseTaskProxyless` | all | [Example](#ReCaptchaV2EnterpriseTaskProxyless) | reCAPTCHA v2 Enterprise |
| `ReCaptchaV2SEnterpriseTaskProxyless` | all | [Example](#ReCaptchaV2SEnterpriseTaskProxyless) | reCAPTCHA v2 Enterprise, carrying the `s` parameter |
| `ReCaptchaV2Classification` | sync | [Example](#ReCaptchaV2Classification) | reCAPTCHA v2 image recognition |

### reCAPTCHA v3

| Task type | Modes | Example | Description |
| :-: | :---: | :-: | --- |
| `ReCaptchaV3TaskProxyless` | all | [Example](#ReCaptchaV3TaskProxyless) | reCAPTCHA v3 |
| `ReCaptchaV3TaskProxylessS9` | all | [Example](#ReCaptchaV3TaskProxylessS9) | reCAPTCHA v3, returns a token scored ≥ 0.9 |
| `ReCaptchaV3EnterpriseTaskProxyless` | all | [Example](#ReCaptchaV3EnterpriseTaskProxyless) | reCAPTCHA v3 Enterprise |
| `ReCaptchaV3EnterpriseTaskProxylessS9` | all | [Example](#ReCaptchaV3EnterpriseTaskProxylessS9) | reCAPTCHA v3 Enterprise, returns a token scored ≥ 0.9 |

### FunCaptcha / Arkose Labs

| Task type | Modes | Example | Description |
| :-: | :---: | :-: | --- |
| `FuncaptchaTaskProxyless` | async | [Example](#FuncaptchaTaskProxyless) | FunCaptcha / Arkose Labs |
| `FunCaptchaClassification` | sync | [Example](#FunCaptchaClassification) | FunCaptcha image recognition |

### hCaptcha

| Task type | Modes | Example | Description |
| :-: | :---: | :-: | --- |
| `HCaptcha` | async | [Example](#HCaptcha) | hCaptcha |
| `HCaptchaClassification` | sync | [Example](#HCaptchaClassification) | hCaptcha image recognition, single or multiple images |

### Cloudflare

| Task type | Modes | Example | Description |
| :-: | :---: | :-: | --- |
| `CloudFlare5STask` | async | [Example](#CloudFlare5STask) | CF five-second interstitial, **requires** a `proxy` |
| `CloudFlareTurnstileTask` | async | [Example](#CloudFlareTurnstileTask) | Turnstile, returns a token |

### Akamai

| Task type | Modes | Example | Description |
| :-: | :---: | :-: | --- |
| `AkamaiWEBTaskProxyless` | sync | [Example](#AkamaiWEBTaskProxyless) | Akamai Web |
| `AkamaiSBSDTaskProxyless` | sync | [Example](#AkamaiSBSDTaskProxyless) | Akamai SBSD |

> Akamai Web is a multi-round flow: feed the `encodedata` of one round back as the `encode_data` of the next. The two spellings genuinely differ on the wire; the SDK keeps the service's definitions as they are rather than "fixing" them.

### DataDome

| Task type | Modes | Example | Description |
| :-: | :---: | :-: | --- |
| `DataDomeTaskProxyless` | sync | [Example](#DataDomeTaskProxyless) | The challenge after an interception, in two steps selected by `step` |
| `DataDomeTagsTaskProxyless` | sync | [Example](#DataDomeTagsTaskProxyless) | Reports a fingerprint on the normal browsing path |

### Other

| Task type | Modes | Example | Description |
| :-: | :---: | :-: | --- |
| `PerimeterX` | async | [Example](#PerimeterX) | PerimeterX clearance cookies |
| `IncapsulaTaskProxyless` | sync | [Example](#IncapsulaTaskProxyless) | Incapsula Reese84 payload |
| `TlsTask` | sync | [Example](#TlsTask) | HTTP request forwarded over TLS, returns the upstream response |

## 📦 Installation

The default feature set includes both the blocking and async clients:

```toml
[dependencies]
ezcapsolver-rs = "0.1"
```

Enable only one of them:

```toml
[dependencies]
ezcapsolver-rs = { version = "0.1", default-features = false, features = ["blocking"] }
# or
ezcapsolver-rs = { version = "0.1", default-features = false, features = ["async"] }
```

Task and response models remain available with the default features off — a crate that only builds requests is not forced to pull in an HTTP stack.

## 🚀 Quick Start

The client reads `EZCAPTCHA_API_KEY` from the environment when the configuration carries no explicit key.

```rust,no_run
use ezcapsolver::{AsyncEzCapSolverClient, RecaptchaV2Task};

async fn run() -> Result<(), Box<dyn std::error::Error>> {
  let client = AsyncEzCapSolverClient::new()?;
  let task = RecaptchaV2Task::builder()
      .website_url("https://example.com")
      .website_key("6Lc_your_site_key")
      .build();
  let solved = client
      .solve_recaptcha_v2_task_proxyless(&task)
      .await?;

  println!("task_id = {:?}", solved.task_id);
  println!("token   = {}", solved.solution.token);
  Ok(())
}
```

## 📖 Usage

### Sync / async

**Every task type has two methods**, taking the same arguments and returning the
same type; only the endpoint differs:

```rust,no_run
use ezcapsolver::{AsyncEzCapSolverClient, RecaptchaV2Task};

async fn run(client: &AsyncEzCapSolverClient, task: &RecaptchaV2Task) -> ezcapsolver::Result<()> {
  // Create, then poll
  let solved = client.solve_recaptcha_v2_task_proxyless(task).await?;
  assert!(solved.task_id.is_some());

  // The synchronous endpoint, answered in one request
  let solved = client.sync_solve_recaptcha_v2_task_proxyless(task).await?;
  assert!(solved.task_id.is_none());   // the endpoint assigns none
  Ok(())
}
```

### Proxy format

Task types that accept a `proxy` take it in one of two shapes, decided by the task type:

| Format | Shape | Used by |
| --- | --- | --- |
| `NORMAL` | `protocol://username:password@host:port` | every type except FunCaptcha |
| `FUN` | `protocol://host:port:username:password` | `FuncaptchaTaskProxyless` only |

`protocol` is one of `http`, `https` or `socks5`. **Both credentials are required** — the service
rejects an unauthenticated proxy — and the host may not be a private address (`127.0.*`,
`192.168.*`, `172.16.*`, `10.0.*`). Where the field is optional, leaving it empty is fine; it is
only a non-empty malformed value that is rejected.


### reCAPTCHA v2

[API docs](https://ezxlabs.com/en/docs/captcha/api/recaptcha-v2)

The first five types share `RecaptchaV2Task` and `RecaptchaSolution`; only the method name differs.

<a id="ReCaptchaV2TaskProxyless"></a>

#### ReCaptchaV2TaskProxyless

```rust,no_run
use ezcapsolver::{AsyncEzCapSolverClient, RecaptchaV2Task};

async fn run() -> Result<(), Box<dyn std::error::Error>> {
  let client = AsyncEzCapSolverClient::new()?;
  let task = RecaptchaV2Task::builder()
      .website_url("https://example.com")
      .website_key("6Lc_your_site_key")
      .build();
  let solved = client
      .solve_recaptcha_v2_task_proxyless(&task)
      .await?;

  println!("task_id = {:?}", solved.task_id);
  println!("token   = {}", solved.solution.token);
  Ok(())
}
```

<a id="ReCaptchaV2TaskProxylessS9"></a>

#### ReCaptchaV2TaskProxylessS9

Same parameters as plain v2, on the high-score queue, returning a token scored 0.9 or above.

```rust,no_run
use ezcapsolver::{AsyncEzCapSolverClient, RecaptchaV2Task};

async fn run() -> Result<(), Box<dyn std::error::Error>> {
  let client = AsyncEzCapSolverClient::new()?;
  let task = RecaptchaV2Task::builder()
      .website_url("https://example.com")
      .website_key("6Lc_your_site_key")
      .build();
  let solved = client
      .solve_recaptcha_v2_task_proxyless_s9(&task)
      .await?;

  println!("token = {}", solved.solution.token);
  Ok(())
}
```

<a id="ReCaptchaV2STaskProxyless"></a>

#### ReCaptchaV2STaskProxyless

Carries the challenge-bound `s` parameter. It is not actually mandatory; leaving it out behaves like plain v2.

```rust,no_run
use ezcapsolver::{AsyncEzCapSolverClient, RecaptchaV2Task};

async fn run() -> Result<(), Box<dyn std::error::Error>> {
  let client = AsyncEzCapSolverClient::new()?;
  let task = RecaptchaV2Task::builder()
      .website_url("https://example.com")
      .website_key("6Lc_your_site_key")
      .s("value-from-the-page")
      .build();
  let solved = client
      .solve_recaptcha_v2_s_task_proxyless(&task)
      .await?;

  println!("token = {}", solved.solution.token);
  Ok(())
}
```

<a id="ReCaptchaV2EnterpriseTaskProxyless"></a>

#### ReCaptchaV2EnterpriseTaskProxyless

Enterprise. If the site uses enterprise parameters beyond `data-s`, pass them through `extra`.

```rust,no_run
use ezcapsolver::{AsyncEzCapSolverClient, RecaptchaV2Task};

async fn run() -> Result<(), Box<dyn std::error::Error>> {
  let client = AsyncEzCapSolverClient::new()?;
  let task = RecaptchaV2Task::builder()
      .website_url("https://example.com")
      .website_key("6Lc_your_site_key")
      .build();
  let solved = client
      .solve_recaptcha_v2_enterprise_task_proxyless(&task)
      .await?;

  println!("token = {}", solved.solution.token);
  Ok(())
}
```

<a id="ReCaptchaV2SEnterpriseTaskProxyless"></a>

#### ReCaptchaV2SEnterpriseTaskProxyless

Enterprise, carrying the `s` parameter.

```rust,no_run
use ezcapsolver::{AsyncEzCapSolverClient, RecaptchaV2Task};

async fn run() -> Result<(), Box<dyn std::error::Error>> {
  let client = AsyncEzCapSolverClient::new()?;
  let task = RecaptchaV2Task::builder()
      .website_url("https://example.com")
      .website_key("6Lc_your_site_key")
      .s("value-from-the-page")
      .build();
  let solved = client
      .solve_recaptcha_v2_s_enterprise_task_proxyless(&task)
      .await?;

  println!("token = {}", solved.solution.token);
  Ok(())
}
```

<a id="ReCaptchaV2Classification"></a>

#### ReCaptchaV2Classification

[API docs](https://ezxlabs.com/en/docs/captcha/api/recaptcha-v2-classification)

An image task that returns the cell indexes to click rather than a token. `size` takes 1, 3 or 4 for a 1x1, 3x3 or 4x4 grid, and defaults to 4.

```rust,no_run
use ezcapsolver::{AsyncEzCapSolverClient, ReClassificationSolution, RecaptchaV2ClassificationTask};

async fn run(client: &AsyncEzCapSolverClient, image_base64: String) -> Result<(), Box<dyn std::error::Error>> {
  let task = RecaptchaV2ClassificationTask::builder()
      .image(image_base64)
      .question("/m/0k4j")
      .build();
  let solution: ReClassificationSolution = client
      .sync_solve_recaptcha_v2_classification(&task)
      .await?
      .solution;

  if solution.is_multi() {
      println!("select cells {:?}", solution.objects);
  } else if solution.is_single() {
      println!("contains target: {}", solution.has_object);
  } else {
      println!("type: {}, extra: {:?}", solution.r#type, solution.extra);
  }
  Ok(())
}
```

### reCAPTCHA v3

[API docs](https://ezxlabs.com/en/docs/captcha/api/recaptcha-v3)

The four types share `RecaptchaV3Task` and `RecaptchaSolution`. `page_action` has to match the `action` passed to `grecaptcha.execute` on the page, or the site-side check fails.

<a id="ReCaptchaV3TaskProxyless"></a>

#### ReCaptchaV3TaskProxyless

```rust,no_run
use ezcapsolver::{AsyncEzCapSolverClient, RecaptchaV3Task};

async fn run() -> Result<(), Box<dyn std::error::Error>> {
  let client = AsyncEzCapSolverClient::new()?;
  let task = RecaptchaV3Task::builder()
      .website_url("https://example.com")
      .website_key("6Lc_your_site_key")
      .page_action("login")
      .build();
  let solved = client
      .solve_recaptcha_v3_task_proxyless(&task)
      .await?;

  println!("task_id = {:?}", solved.task_id);
  println!("token   = {}", solved.solution.token);
  Ok(())
}
```

<a id="ReCaptchaV3TaskProxylessS9"></a>

#### ReCaptchaV3TaskProxylessS9

```rust,no_run
use ezcapsolver::{AsyncEzCapSolverClient, RecaptchaV3Task};

async fn run() -> Result<(), Box<dyn std::error::Error>> {
  let client = AsyncEzCapSolverClient::new()?;
  let task = RecaptchaV3Task::builder()
      .website_url("https://example.com")
      .website_key("6Lc_your_site_key")
      .page_action("login")
      .build();
  let solved = client
      .solve_recaptcha_v3_task_proxyless_s9(&task)
      .await?;

  println!("token = {}", solved.solution.token);
  Ok(())
}
```

<a id="ReCaptchaV3EnterpriseTaskProxyless"></a>

#### ReCaptchaV3EnterpriseTaskProxyless

```rust,no_run
use ezcapsolver::{AsyncEzCapSolverClient, RecaptchaV3Task};

async fn run() -> Result<(), Box<dyn std::error::Error>> {
  let client = AsyncEzCapSolverClient::new()?;
  let task = RecaptchaV3Task::builder()
      .website_url("https://example.com")
      .website_key("6Lc_your_site_key")
      .page_action("login")
      .build();
  let solved = client
      .solve_recaptcha_v3_enterprise_task_proxyless(&task)
      .await?;

  println!("token = {}", solved.solution.token);
  Ok(())
}
```

<a id="ReCaptchaV3EnterpriseTaskProxylessS9"></a>

#### ReCaptchaV3EnterpriseTaskProxylessS9

```rust,no_run
use ezcapsolver::{AsyncEzCapSolverClient, RecaptchaV3Task};

async fn run() -> Result<(), Box<dyn std::error::Error>> {
  let client = AsyncEzCapSolverClient::new()?;
  let task = RecaptchaV3Task::builder()
      .website_url("https://example.com")
      .website_key("6Lc_your_site_key")
      .page_action("login")
      .build();
  let solved = client
      .solve_recaptcha_v3_enterprise_task_proxyless_s9(&task)
      .await?;

  println!("token = {}", solved.solution.token);
  Ok(())
}
```

### FunCaptcha / Arkose Labs

[API docs](https://ezxlabs.com/en/docs/captcha/api/funcaptcha)

<a id="FuncaptchaTaskProxyless"></a>

#### FuncaptchaTaskProxyless

```rust,no_run
use ezcapsolver::{AsyncEzCapSolverClient, FunCaptchaTask};

async fn run() -> Result<(), Box<dyn std::error::Error>> {
  let client = AsyncEzCapSolverClient::new()?;
  let task = FunCaptchaTask::builder()
      .website_url("https://example.com")
      .website_key("your-public-key")
      .build();
  let solved = client.solve_funcaptcha_task_proxyless(&task).await?;

  println!("token = {}", solved.solution.token);
  Ok(())
}
```

> FunCaptcha is the one task type whose proxy uses the `FUN` format:
> `protocol://host:port:username:password`, with the credentials after the host
> rather than before it.

<a id="FunCaptchaClassification"></a>

#### FunCaptchaClassification

```rust,no_run
use ezcapsolver::{AsyncEzCapSolverClient, FunCaptchaClassificationSolution, FunCaptchaClassificationTask};

async fn run(client: &AsyncEzCapSolverClient, image_base64: String) -> Result<(), Box<dyn std::error::Error>> {
  let task = FunCaptchaClassificationTask::builder()
      .image(image_base64)
      .question("Pick the animal facing left")
      .build();
  let solution: FunCaptchaClassificationSolution = client
      .sync_solve_funcaptcha_classification(&task)
      .await?
      .solution;

  // The shape of this solution is not confirmed yet, so every field the worker
  // returns lands in extra.
  println!("{:?}", solution.extra);
  Ok(())
}
```

### hCaptcha

[API docs](https://ezxlabs.com/en/docs/captcha/api/hcaptcha)

<a id="HCaptcha"></a>

#### HCaptcha

The hCaptcha token field is `generated_pass_uuid`, not `token` — that is the service's naming, and the SDK keeps it.

```rust,no_run
use ezcapsolver::{AsyncEzCapSolverClient, HcaptchaTask};

async fn run() -> Result<(), Box<dyn std::error::Error>> {
  let client = AsyncEzCapSolverClient::new()?;
  let task = HcaptchaTask::builder()
      .website_url("https://example.com")
      .website_key("your-site-key")
      .build();
  let solved = client.solve_hcaptcha(&task).await?;

  println!("token = {}", solved.solution.generated_pass_uuid);
  Ok(())
}
```

<a id="HCaptchaClassification"></a>

#### HCaptchaClassification

Use `image` for a single image and `images` for several; both fields are optional.

```rust,no_run
use ezcapsolver::{AsyncEzCapSolverClient, HcaptchaClassificationSolution, HcaptchaClassificationTask};

async fn run(client: &AsyncEzCapSolverClient, images: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
  let task = HcaptchaClassificationTask::builder()
      .images(images)
      .question("Please click each image containing a bicycle")
      .build();
  let solution: HcaptchaClassificationSolution = client
      .sync_solve_hcaptcha_classification(&task)
      .await?
      .solution;

  // The shape is unconfirmed here too; everything is in extra.
  println!("{:?}", solution.extra);
  Ok(())
}
```

### Cloudflare

<a id="CloudFlare5STask"></a>

#### CloudFlare5STask

[API docs](https://ezxlabs.com/en/docs/captcha/api/cloudflare-5s)

The five-second interstitial **requires** a `proxy`, and what comes back is not a single token but the headers and clearance cookies to replay against the protected site:

```rust,no_run
use ezcapsolver::{AsyncEzCapSolverClient, Cloudflare5sTask};

async fn run() -> Result<(), Box<dyn std::error::Error>> {
  let client = AsyncEzCapSolverClient::new()?;
  let task = Cloudflare5sTask::builder()
      .website_url("https://example.com")
      .proxy("http://user:pass@127.0.0.1:8080")
      .build();
  let solved = client.solve_cloudflare_5s_task(&task).await?;
  let solution = &solved.solution;

  for (name, value) in &solution.cookies {
      println!("{name}={value}");
  }
  println!("TLS fingerprint = {}", solution.tls_version);
  Ok(())
}
```

<a id="CloudFlareTurnstileTask"></a>

#### CloudFlareTurnstileTask

[API docs](https://ezxlabs.com/en/docs/captcha/api/turnstile)

Turnstile takes an optional `proxy` and returns a single token.

```rust,no_run
use ezcapsolver::{AsyncEzCapSolverClient, CloudflareTurnstileTask};

async fn run() -> Result<(), Box<dyn std::error::Error>> {
  let client = AsyncEzCapSolverClient::new()?;
  let task = CloudflareTurnstileTask::builder()
      .website_url("https://example.com")
      .website_key("0x4AAA_your_site_key")
      .build();
  let solved = client.solve_cloudflare_turnstile_task(&task).await?;

  println!("token = {}", solved.solution.token);
  Ok(())
}
```

### Akamai

<a id="AkamaiWEBTaskProxyless"></a>

#### AkamaiWEBTaskProxyless

[API docs](https://ezxlabs.com/en/docs/captcha/api/akamai)

Akamai Web is a multi-round flow: feed the `encodedata` of one round back as the
`encode_data` of the next. The two spellings genuinely differ on the wire, and the
SDK keeps the service's definitions as they are.

```rust,no_run
use ezcapsolver::{AkamaiWebSolution, AkamaiWebTask, AsyncEzCapSolverClient};

async fn run(client: &AsyncEzCapSolverClient) -> Result<(), Box<dyn std::error::Error>> {
  let mut encode_data = String::new();

  for index in 1..=3 {
      let task = AkamaiWebTask::builder()
          .page_url("https://example.com")
          .ua("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
          .lang("en-US")
          .index(index)
          .encode_data(encode_data.clone())
          .build();
      let solution: AkamaiWebSolution = client
          .sync_solve_akamai_web_task_proxyless(&task)
          .await?
      .solution;

      println!("round {index} payload = {}", solution.payload);
      encode_data = solution.encodedata;
  }
  Ok(())
}
```

<a id="AkamaiSBSDTaskProxyless"></a>

#### AkamaiSBSDTaskProxyless

[API docs](https://ezxlabs.com/en/docs/captcha/api/akamai-sbsd)

A single-round task; all six fields are required.

```rust,no_run
use ezcapsolver::{AkamaiSbsdSolution, AkamaiSbsdTask, AsyncEzCapSolverClient};

async fn run(client: &AsyncEzCapSolverClient, script_base64: String) -> Result<(), Box<dyn std::error::Error>> {
  let task = AkamaiSbsdTask::builder()
      .page_url("https://example.com")
      .sbsd_url("https://example.com/.well-known/sbsd")
      .bm_so("value-from-the-page")
      .ua("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
      .lang("en-US")
      .script_base64(script_base64)
      .build();
  let solution: AkamaiSbsdSolution = client
      .sync_solve_akamai_sbsd_task_proxyless(&task)
      .await?
      .solution;

  println!("payload = {}", solution.payload);
  Ok(())
}
```

### DataDome

<a id="DataDomeTaskProxyless"></a>

#### DataDomeTaskProxyless

The challenge after a DataDome interception runs in two steps. Both share
`DataDomeTask` and are selected by `step`; which fields of `DataDomeSolution`
carry a value depends on the step:

```rust,no_run
use ezcapsolver::{AsyncEzCapSolverClient, DataDomeSolution, DataDomeStep, DataDomeTask};

async fn run(client: &AsyncEzCapSolverClient, html_b64: String) -> Result<(), Box<dyn std::error::Error>> {
  // Step one: get the challenge address out of the intercepted page.
  let task = DataDomeTask::builder()
      .html_b64(html_b64)
      .step(DataDomeStep::One)
      .build();
  let solution: DataDomeSolution = client.sync_solve_data_dome_task_proxyless(&task).await?.solution;

  println!("challenge url = {:?}", solution.url);

  // Step two uses DataDomeStep::Two and brings back the validation body.
  Ok(())
}
```

<a id="DataDomeTagsTaskProxyless"></a>

#### DataDomeTagsTaskProxyless

Reports a fingerprint on the normal browsing path. Its field names are camelCase, unlike `DataDomeTask` above.

```rust,no_run
use ezcapsolver::{AsyncEzCapSolverClient, DataDomeSolution, DataDomeTagsTask};

async fn run(client: &AsyncEzCapSolverClient) -> Result<(), Box<dyn std::error::Error>> {
  let task = DataDomeTagsTask::builder()
      .ddk("your-datadome-key")
      .referer("https://example.com")
      .ua("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
      .bpc(1)
      .build();
  let solution: DataDomeSolution = client
      .sync_solve_data_dome_tags_task_proxyless(&task)
      .await?
      .solution;

  println!("{:?}", solution.extra);
  Ok(())
}
```

### Other

<a id="PerimeterX"></a>

#### PerimeterX

[API docs](https://ezxlabs.com/en/docs/captcha/api/perimeterx)

```rust,no_run
use ezcapsolver::{AsyncEzCapSolverClient, PerimeterXTask};

async fn run() -> Result<(), Box<dyn std::error::Error>> {
  let client = AsyncEzCapSolverClient::new()?;
  let task = PerimeterXTask::builder()
      .website_key("PX_your_app_id")
      .build();
  let solved = client.solve_perimeter_x(&task).await?;
  let solution = &solved.solution;

  println!("_px3   = {}", solution.px3);
  println!("_pxvid = {}", solution.pxvid);
  Ok(())
}
```

<a id="IncapsulaTaskProxyless"></a>

#### IncapsulaTaskProxyless

[API docs](https://ezxlabs.com/en/docs/captcha/api/incapsula)

```rust,no_run
use ezcapsolver::{AsyncEzCapSolverClient, IncapsulaSolution, IncapsulaTask};

async fn run(client: &AsyncEzCapSolverClient, script: String) -> Result<(), Box<dyn std::error::Error>> {
  let task = IncapsulaTask::builder()
      .page_url("https://example.com")
      .script(script)
      .ua("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
      .accept_language("en-US,en;q=0.9")
      .build();
  let solution: IncapsulaSolution = client
      .sync_solve_incapsula_task_proxyless(&task)
      .await?
      .solution;

  println!("reese84 = {}", solution.data);
  Ok(())
}
```

<a id="TlsTask"></a>

#### TlsTask

[API docs](https://ezxlabs.com/en/docs/captcha/api/tls-forward)

This type does not solve a captcha. It sends one HTTP request through a worker's TLS fingerprint and hands the upstream response back untouched:

```rust,no_run
use ezcapsolver::{AsyncEzCapSolverClient, TlsForwardSolution, TlsForwardTask, TlsHttpMethod};

async fn run(client: &AsyncEzCapSolverClient) -> Result<(), Box<dyn std::error::Error>> {
  let task = TlsForwardTask::builder()
      .tls_type("chrome")
      .url("https://example.com/api")
      .proxy("http://user:pass@127.0.0.1:8080")
      .method(TlsHttpMethod::Get)
      .build();
  let solution: TlsForwardSolution = client.sync_solve_tls_task(&task).await?.solution;

  println!("HTTP {}, {} bytes of body", solution.status, solution.body.len());
  Ok(())
}
```

### Pass-through fields

Every task model exposes an `extra` map whose entries are flattened into the task JSON. A parameter the service adds later works without an SDK update. The reserved `type` field always stays under SDK control:

```rust
use ezcapsolver::HcaptchaTask;
use serde_json::json;

let mut task = HcaptchaTask {
    website_url: "https://example.com".to_owned(),
    website_key: "site-key".to_owned(),
    lang: "en-US".to_owned(),
    invisible: false,
    ..Default::default()
};
// A parameter this release does not model, or one the service ships later.
task.extra.insert("futureFlag".to_owned(), json!(true));
```

A declared field always wins over an `extra` entry of the same name, so adding
one here can never silently change a parameter that was set explicitly.

Every solution carries an `extra` map as well, which hands back the fields the service returns but the model does not declare:

```rust,no_run
use ezcapsolver::{AsyncEzCapSolverClient, Cloudflare5sTask};

async fn run(client: &AsyncEzCapSolverClient, task: &Cloudflare5sTask) -> Result<(), Box<dyn std::error::Error>> {
  let solved = client.solve_cloudflare_5s_task(task).await?;
  let solution = &solved.solution;

  for (name, value) in &solution.cookies {
      println!("{name}={value}");
  }
  println!("fingerprint: {}", solution.tls_version);

  if let Some(value) = solution.extra.get("aFieldAddedLater") {
      println!("{value}");
  }
  Ok(())
}
```

### Custom task types

Task types are an **open set**. Pass the type name as a plain string, with parameters given as any value that serializes to a JSON object:

```rust,no_run
use ezcapsolver::AsyncEzCapSolverClient;
use serde_json::json;

async fn run(client: &AsyncEzCapSolverClient) -> Result<(), Box<dyn std::error::Error>> {
  let solved = client
      .solve("BrandNewTaskType", &json!({
          "websiteURL": "https://example.com",
          "anyFutureParam": 42
      }))
      .await?;

  println!("{:?}", solved.solution.as_value());
  Ok(())
}
```

Use this when the service ships a new captcha type before the SDK catches up.

`solve` creates an asynchronous task and polls for its result. For a type supported by the synchronous endpoint, use `sync_solve` to get the result in one request:

```rust,no_run
use ezcapsolver::AsyncEzCapSolverClient;
use serde_json::json;

async fn run(client: &AsyncEzCapSolverClient) -> Result<(), Box<dyn std::error::Error>> {
  let solved = client
      .sync_solve("BrandNewSyncTaskType", &json!({
          "websiteURL": "https://example.com",
          "anyFutureParam": 42
      }))
      .await?;

  assert!(solved.task_id.is_none());
  println!("{:?}", solved.solution.as_value());
  Ok(())
}
```

Both methods return `Solved<Solution>`, including `request_id` and the untouched result in `raw`. `EzCapSolverClient` provides the same methods without `.await`. `sync_solve` uses `sync_timeout` and does not poll.

## ⚙️ Configuration

```rust
use std::time::Duration;

use ezcapsolver::{ClientConfig, PollingConfig};

let config = ClientConfig::builder()
    .timeout(Duration::from_secs(30))
    .sync_timeout(Duration::from_secs(240))
    .polling(PollingConfig::new(Duration::from_secs(3), 50))
    .app_id(42)
    .proxy("http://127.0.0.1:8080")
    .user_agent("my-app/1.0")
    .build();
```

All settings are optional: `ClientConfig::builder().build()` equals `ClientConfig::default()`. Set `.client_key("your-client-key")` to use an explicit key; otherwise the client reads `EZCAPTCHA_API_KEY` when it is created. String setters accept `&str` or `String`, and `maybe_*` setters accept optional values. `build()` returns the configuration directly; validation still runs during client creation. The existing `with_*` methods remain available.

| Setting | Default | Covers |
| --- | :---: | --- |
| `timeout` | 30 s | Asynchronous endpoints and balance queries |
| `sync_timeout` | 240 s | The synchronous task endpoint, clearing the service's 180 s worker deadline |
| `polling` | 3 s × 50 | Result queries, up to 150 s per task |
| `proxy` | none | **The SDK's own egress**, unrelated to a task's `proxy` parameter |
| `async_base_url` | `https://api.ez-captcha.com` | Asynchronous tasks and balance queries |
| `sync_base_url` | `https://sync.ez-captcha.com` | Synchronous tasks; the service splits the two deployments |

Set `async_base_url` / `sync_base_url` with `.with_base_urls(async_url, sync_url)` to reach a private gateway or a test server. To bring your own HTTP client — a shared pool, a middleware stack, a custom TLS setup — build the client with `with_http_client(config, http)`; that client's own transport and headers are used as they are.

The service keeps a task result for **five minutes** after creation. A polling budget past that cannot succeed, and `wait_for_result` on an older task returns `ERROR_TASK_NOT_EXIST`.

## ⚠️ Errors

```rust
use ezcapsolver::Error;

fn inspect(error: Error) {
  match error {
      // The service refused the request. Code, description, and HTTP status
      // are all present; validation failures also carry per-field messages.
      Error::EzError(api) => eprintln!(
          "{:?}: {:?} (HTTP {})",
          api.error_code, api.error_description, api.http_status
      ),

      // Polling ran out. The task may still be running, and the task ID stays
      // usable for a later query.
      Error::PollingExhausted { task_id, attempts, .. } => {
          eprintln!("task {task_id} unfinished after {attempts} attempts")
      }

      // The task was created and billed, but the wait broke off.
      Error::WaitInterrupted { source, .. } => eprintln!("wait broke off: {source}"),

      // The worker returned a shape this release does not model; the raw value
      // travels with the error.
      Error::SolutionDecode { raw, .. } => eprintln!("unexpected shape: {raw}"),

      other => eprintln!("{other}"),
  }
}
```

Whichever failure it is, one question answers whether a **billed** task is still recoverable:

```rust
if let Some(task_id) = error.task_id() {
    // The task is on the service; its result is held for five minutes after
    // creation. Waiting again is free — creating a second task is billed again.
    let result = client.wait_for_result(task_id).await?;
}
```

`None` means nothing was billed, so there is nothing to recover. Which variant stores the id is an implementation detail.


This SDK does not retry a request for you: creating a task is billed and is not idempotent, so the retry policy is yours. Three helpers on `EzError` supply the facts to decide with:

| Method | True when | What to do |
| --- | --- | --- |
| `is_authentication_error()` | `ERROR_KEY_DOES_NOT_EXIST`, `ERROR_KEY_NOT_AVAILABLE`, `ERROR_ZERO_BALANCE` | **Stop.** The service counts these per key and bans after thirty in a minute — backing off makes it worse |
| `is_terminal()` | The identical request would fail identically | Fix the request; retrying changes nothing |
| `is_rate_limited()` | `ERROR_REQUEST_LIMIT`, `ERROR_REQUEST_BANNED` | Wait and ask again. Both clear on their own |

`is_terminal()` returns `false` for a code this release has not seen, so a new transient code never talks you out of a retry that would have worked. `is_rate_limited()` is narrower than that: it names two codes, rather than every code not known to be terminal.

The one thing `wait_for_result` does retry is a throttled poll. `ERROR_REQUEST_LIMIT` and `ERROR_REQUEST_BANNED` refuse the *query*, not the task — the service turns the request away before it ever looks the task up, so the task is still queued and still billed. The loop spends the attempt and polls again instead of throwing away a result that was about to arrive. Every other API error is the poll's answer and ends the wait.

## 📝 Logging

The SDK emits diagnostics through `tracing`; the application picks the subscriber and filter.

| Level | Events |
| :---: | --- |
| `INFO` | Task created, task completed |
| `DEBUG` | One line per operation, and one per polling attempt |
| `TRACE` | Request bodies, plus response status, duration, size, and body |

`TRACE` is for debugging a failing integration. Request bodies are rendered with every `clientKey` and `proxy` replaced by `[REDACTED]` at **any nesting depth**, and bodies are not rendered at all unless a subscriber enabled `TRACE` — so the default log level costs nothing.

Scope the filter to this crate; a global `TRACE` would bury the SDK's output under the HTTP stack's own:

```rust,no_run
use tracing::Level;
use tracing_subscriber::{filter::Targets, prelude::*};

tracing_subscriber::registry()
    .with(tracing_subscriber::fmt::layer())
    .with(Targets::new().with_target("ezcapsolver", Level::DEBUG))
    .init();
```

## 🧪 Runnable Examples

Runnable examples live in [`examples/`](./examples). Set `EZCAPTCHA_API_KEY` first, then:

```bash
export EZCAPTCHA_API_KEY=your-client-key

cargo run --example recaptcha_v2_task_proxyless
cargo run --example cloud_flare_turnstile_task
cargo run --example tls_task

# The SDK itself rather than one task type
cargo run --example async_client_task     # every client setting, spelled out
cargo run --example blocking_client_task  # the same, without a runtime
cargo run --example concurrency           # one client across many tasks
cargo run --example raw_usage             # low-level create/poll workflow
cargo run --example logging_and_errors    # tracing setup and every failure kind
```

There is one file per task type, named after the wire task type so it lines up with the Go and Python SDKs file for file — the full list is in the [example index](./examples/README.md). Each task example names the task type it covers and links to the matching page of the official API documentation. Examples that need a proxy read `EZCAPTCHA_PROXY`; none of them hardcode a credential.

> Every run creates a real task and is billed, whether or not the worker succeeds.

## 🎛️ Feature Flags

| Feature | Default | Enables |
| --- | :---: | --- |
| `async` | ✅ | `AsyncEzCapSolverClient`, Tokio, Reqwest |
| `blocking` | ✅ | `EzCapSolverClient`, Reqwest blocking |

Both features share one transport implementation, so behaviour cannot drift between the two clients.

## 🛠️ Development

Minimum supported Rust version: **1.96.0**, edition 2024.

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo doc --no-deps --all-features
```

## 📄 License

Licensed under the [Apache License 2.0](./LICENSE).
