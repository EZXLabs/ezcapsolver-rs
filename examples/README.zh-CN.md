# 示例

[English](./README.md) · [简体中文](./README.zh-CN.md)

每种任务类型一个可运行文件，文件名取自线格式的任务类型名，与 Go、Python 两版逐个对应。

从仓库根目录运行——图片识别类示例会从 `examples/fixtures/` 读图：

```bash
export EZCAPTCHA_API_KEY=your-key
cargo run --example recaptcha_v2_task_proxyless
```

所有示例都不硬编码密钥，一律读 `EZCAPTCHA_API_KEY`。需要 worker 代理的留了占位符，自行填入。

> 每次运行都会创建一个真实任务并**扣费**，无论 worker 是否成功。reCAPTCHA v2 与
> hCaptcha 的示例指向厂商自己的 demo 页面，可以直接跑；其余的站点密钥是占位符。

## SDK 本身

| 示例 | 讲什么 |
| --- | --- |
| [`async_client_task.rs`](./async_client_task.rs) | Tokio 客户端，以及两条超时预算为什么要分开 |
| [`blocking_client_task.rs`](./blocking_client_task.rs) | 同一个任务，不需要运行时 |
| [`concurrency.rs`](./concurrency.rs) | 一个客户端跑多个任务；clone 是共享连接池而不是复制 |
| [`raw_usage.rs`](./raw_usage.rs) | 手动轮询，以及调用 SDK 尚未建模的任务类型 |
| [`logging_and_errors.rs`](./logging_and_errors.rs) | 装 subscriber，以及区分各类失败 |

## reCAPTCHA v2

[接口文档](https://ezxlabs.com/zh/docs/captcha/api/recaptcha-v2)

| 示例 | 任务类型 |
| --- | --- |
| [`recaptcha_v2_task_proxyless.rs`](./recaptcha_v2_task_proxyless.rs) | `ReCaptchaV2TaskProxyless` |
| [`recaptcha_v2_task_proxyless_s9.rs`](./recaptcha_v2_task_proxyless_s9.rs) | `ReCaptchaV2TaskProxylessS9` |
| [`recaptcha_v2_s_task_proxyless.rs`](./recaptcha_v2_s_task_proxyless.rs) | `ReCaptchaV2STaskProxyless` |
| [`recaptcha_v2_enterprise_task_proxyless.rs`](./recaptcha_v2_enterprise_task_proxyless.rs) | `ReCaptchaV2EnterpriseTaskProxyless` |
| [`recaptcha_v2_s_enterprise_task_proxyless.rs`](./recaptcha_v2_s_enterprise_task_proxyless.rs) | `ReCaptchaV2SEnterpriseTaskProxyless` |
| [`recaptcha_v2_classification.rs`](./recaptcha_v2_classification.rs) | `ReCaptchaV2Classification` |

## reCAPTCHA v3

[接口文档](https://ezxlabs.com/zh/docs/captcha/api/recaptcha-v3)

| 示例 | 任务类型 |
| --- | --- |
| [`recaptcha_v3_task_proxyless.rs`](./recaptcha_v3_task_proxyless.rs) | `ReCaptchaV3TaskProxyless` |
| [`recaptcha_v3_task_proxyless_s9.rs`](./recaptcha_v3_task_proxyless_s9.rs) | `ReCaptchaV3TaskProxylessS9` |
| [`recaptcha_v3_enterprise_task_proxyless.rs`](./recaptcha_v3_enterprise_task_proxyless.rs) | `ReCaptchaV3EnterpriseTaskProxyless` |
| [`recaptcha_v3_enterprise_task_proxyless_s9.rs`](./recaptcha_v3_enterprise_task_proxyless_s9.rs) | `ReCaptchaV3EnterpriseTaskProxylessS9` |

## FunCaptcha / Arkose Labs

| 示例 | 任务类型 |
| --- | --- |
| [`funcaptcha_task_proxyless.rs`](./funcaptcha_task_proxyless.rs) | `FuncaptchaTaskProxyless` |
| [`funcaptcha_classification.rs`](./funcaptcha_classification.rs) | `FunCaptchaClassification` |

## hCaptcha

| 示例 | 任务类型 |
| --- | --- |
| [`hcaptcha.rs`](./hcaptcha.rs) | `HCaptcha` |
| [`hcaptcha_classification.rs`](./hcaptcha_classification.rs) | `HCaptchaClassification` |

## Cloudflare

| 示例 | 任务类型 |
| --- | --- |
| [`cloud_flare_5s_task.rs`](./cloud_flare_5s_task.rs) | `CloudFlare5STask` |
| [`cloud_flare_turnstile_task.rs`](./cloud_flare_turnstile_task.rs) | `CloudFlareTurnstileTask` |

## Akamai

| 示例 | 任务类型 |
| --- | --- |
| [`akamai_web_task_proxyless.rs`](./akamai_web_task_proxyless.rs) | `AkamaiWEBTaskProxyless` |
| [`akamai_sbsd_task_proxyless.rs`](./akamai_sbsd_task_proxyless.rs) | `AkamaiSBSDTaskProxyless` |

## DataDome

| 示例 | 任务类型 |
| --- | --- |
| [`datadome_task_proxyless.rs`](./datadome_task_proxyless.rs) | `DataDomeTaskProxyless` |
| [`datadome_tags_task_proxyless.rs`](./datadome_tags_task_proxyless.rs) | `DataDomeTagsTaskProxyless` |

## 其他

| 示例 | 任务类型 |
| --- | --- |
| [`perimeter_x.rs`](./perimeter_x.rs) | `PerimeterX` |
| [`incapsula_task_proxyless.rs`](./incapsula_task_proxyless.rs) | `IncapsulaTaskProxyless` |
| [`tls_task.rs`](./tls_task.rs) | `TlsTask` |
