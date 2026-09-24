# Examples

[English](./README.md) · [简体中文](./README.zh-CN.md)

One runnable file per task type, named after the wire task type so it lines up
with the Go and Python SDKs file for file.

Run them from the repository root — the classification examples read their
images from `examples/fixtures/`:

```bash
export EZCAPTCHA_API_KEY=your-key
cargo run --example recaptcha_v2_task_proxyless
```

No example hardcodes a key; they all read `EZCAPTCHA_API_KEY`. Ones that need a
worker proxy carry a placeholder for you to fill in.

> Every run creates a real task and is billed, whether or not the worker
> succeeds. The reCAPTCHA v2 and hCaptcha examples point at the vendors' own
> demo pages and work as written; the rest carry placeholder site keys.

## The SDK itself

| Example | What it covers |
| --- | --- |
| [`async_client_task.rs`](./async_client_task.rs) | The Tokio client, and why the two timeout budgets are separate |
| [`blocking_client_task.rs`](./blocking_client_task.rs) | The same task without a runtime |
| [`concurrency.rs`](./concurrency.rs) | Sharing one client across tasks; cloning shares the pool |
| [`raw_usage.rs`](./raw_usage.rs) | Polling by hand, and reaching a task type the SDK does not model |
| [`logging_and_errors.rs`](./logging_and_errors.rs) | Installing a subscriber, and telling the failure kinds apart |

## reCAPTCHA v2

[Docs](https://docs.ezxlabs.com/docs/captcha/api/recaptcha-v2)

| Example | Task type |
| --- | --- |
| [`recaptcha_v2_task_proxyless.rs`](./recaptcha_v2_task_proxyless.rs) | `ReCaptchaV2TaskProxyless` |
| [`recaptcha_v2_task_proxyless_s9.rs`](./recaptcha_v2_task_proxyless_s9.rs) | `ReCaptchaV2TaskProxylessS9` |
| [`recaptcha_v2_s_task_proxyless.rs`](./recaptcha_v2_s_task_proxyless.rs) | `ReCaptchaV2STaskProxyless` |
| [`recaptcha_v2_enterprise_task_proxyless.rs`](./recaptcha_v2_enterprise_task_proxyless.rs) | `ReCaptchaV2EnterpriseTaskProxyless` |
| [`recaptcha_v2_s_enterprise_task_proxyless.rs`](./recaptcha_v2_s_enterprise_task_proxyless.rs) | `ReCaptchaV2SEnterpriseTaskProxyless` |
| [`recaptcha_v2_classification.rs`](./recaptcha_v2_classification.rs) | `ReCaptchaV2Classification` |

## reCAPTCHA v3

[Docs](https://docs.ezxlabs.com/docs/captcha/api/recaptcha-v3)

| Example | Task type |
| --- | --- |
| [`recaptcha_v3_task_proxyless.rs`](./recaptcha_v3_task_proxyless.rs) | `ReCaptchaV3TaskProxyless` |
| [`recaptcha_v3_task_proxyless_s9.rs`](./recaptcha_v3_task_proxyless_s9.rs) | `ReCaptchaV3TaskProxylessS9` |
| [`recaptcha_v3_enterprise_task_proxyless.rs`](./recaptcha_v3_enterprise_task_proxyless.rs) | `ReCaptchaV3EnterpriseTaskProxyless` |
| [`recaptcha_v3_enterprise_task_proxyless_s9.rs`](./recaptcha_v3_enterprise_task_proxyless_s9.rs) | `ReCaptchaV3EnterpriseTaskProxylessS9` |

## FunCaptcha / Arkose Labs

| Example | Task type |
| --- | --- |
| [`funcaptcha_task_proxyless.rs`](./funcaptcha_task_proxyless.rs) | `FuncaptchaTaskProxyless` |
| [`funcaptcha_classification.rs`](./funcaptcha_classification.rs) | `FunCaptchaClassification` |

## hCaptcha

| Example | Task type |
| --- | --- |
| [`hcaptcha.rs`](./hcaptcha.rs) | `HCaptcha` |
| [`hcaptcha_classification.rs`](./hcaptcha_classification.rs) | `HCaptchaClassification` |

## Cloudflare

| Example | Task type |
| --- | --- |
| [`cloud_flare_5s_task.rs`](./cloud_flare_5s_task.rs) | `CloudFlare5STask` |
| [`cloud_flare_turnstile_task.rs`](./cloud_flare_turnstile_task.rs) | `CloudFlareTurnstileTask` |

## Akamai

| Example | Task type |
| --- | --- |
| [`akamai_web_task_proxyless.rs`](./akamai_web_task_proxyless.rs) | `AkamaiWEBTaskProxyless` |
| [`akamai_sbsd_task_proxyless.rs`](./akamai_sbsd_task_proxyless.rs) | `AkamaiSBSDTaskProxyless` |

## DataDome

| Example | Task type |
| --- | --- |
| [`datadome_task_proxyless.rs`](./datadome_task_proxyless.rs) | `DataDomeTaskProxyless` |
| [`datadome_tags_task_proxyless.rs`](./datadome_tags_task_proxyless.rs) | `DataDomeTagsTaskProxyless` |

## Other

| Example | Task type |
| --- | --- |
| [`perimeter_x.rs`](./perimeter_x.rs) | `PerimeterX` |
| [`incapsula_task_proxyless.rs`](./incapsula_task_proxyless.rs) | `IncapsulaTaskProxyless` |
| [`tls_task.rs`](./tls_task.rs) | `TlsTask` |
