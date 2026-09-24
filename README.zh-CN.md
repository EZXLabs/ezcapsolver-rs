<div align="center">
  <img src="./assets/ez-captcha-logo.svg" alt="EZCaptchaSolver by EZXLabs" height="88">
  &nbsp;&nbsp;
  <img src="./assets/rust.svg" alt="Rust" height="88">
  <h1>EZCaptchaSolver Rust SDK</h1>
  <p>
    <a href="https://github.com/EZXLabs/ezcapsolver-rs/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/EZXLabs/ezcapsolver-rs/actions/workflows/ci.yml/badge.svg"></a>
    <a href="https://crates.io/crates/ezcapsolver-rs"><img alt="crates.io" src="https://img.shields.io/crates/v/ezcapsolver-rs.svg?logo=rust"></a>
    <a href="https://docs.rs/ezcapsolver-rs"><img alt="docs.rs" src="https://img.shields.io/docsrs/ezcapsolver-rs?logo=docsdotrs"></a>
    <a href="./LICENSE"><img alt="License: Apache-2.0" src="https://img.shields.io/badge/license-Apache--2.0-blue.svg"></a>
    <a href="https://www.rust-lang.org"><img alt="Rust 1.96.0+" src="https://img.shields.io/badge/rust-1.96.0%2B-orange.svg?logo=rust&logoColor=white"></a>
    <a href="https://ezxlabs.com"><img alt="EZXLabs 官网" src="https://img.shields.io/badge/website-ezxlabs.com-FFDB29?logoColor=black"></a>
  </p>
  <p>
    <a href="https://ezxlabs.com">🌐 官网</a> &nbsp;·&nbsp;
    <a href="https://docs.ezxlabs.com/zh/docs/captcha/api">📚 EZCaptchaSolver API 文档</a> &nbsp;·&nbsp;
    <a href="./examples">🧪 示例</a> &nbsp;·&nbsp;
    <a href="#-支持的验证码类型">🧩 验证码类型</a>
  </p>
  <p><a href="./README.md">English</a> &nbsp;·&nbsp; <b>简体中文</b></p>
</div>

---

EZCaptchaSolver Rust SDK 是 [EZXLabs](https://ezxlabs.com) 维护的开源 Rust 客户端，用于接入其 CAPTCHA 识别任务 API。它为下列受支持的任务类型提供类型化请求、异步和阻塞客户端；crate 还包含不用于解验证码的 TLS 转发任务。了解 SDK 产品系列请查看 [EZCaptchaSolver SDK 产品页](https://ezxlabs.com/zh/products/sdk)，查看 HTTP 请求与响应字段请访问 [EZCaptchaSolver API 文档](https://docs.ezxlabs.com/zh/docs/captcha/api)，Rust 调用代码请以[本仓库示例](./examples/README.zh-CN.md)为准。

## 🧩 支持的验证码类型

验证码任务类型分为同步和异步两种：

- 同步：创建任务后阻塞请求，直到任务完成取得任务结果。
- 异步：创建任务成功后返回任务ID，后续可通过任务ID轮询尝试获取任务结果。适合验证码处理时长较久的任务类型。

同一种验证码任务类型能够同时支持同步和异步两种，几乎所有类型都会支持同步方式。然后有部分类型只会支持异步方式。

### reCAPTCHA v2

| 任务类型 | 支持方式 | 示例 | 描述 |
| :-: | :---: | :-: | --- |
| `ReCaptchaV2TaskProxyless` | all | [示例](#ReCaptchaV2TaskProxyless) | reCAPTCHA v2 解决方案 |
| `ReCaptchaV2TaskProxylessS9` | all | [示例](#ReCaptchaV2TaskProxylessS9) | reCAPTCHA v2，返回分值 ≥ 0.9 的 token |
| `ReCaptchaV2STaskProxyless` | all | [示例](#ReCaptchaV2STaskProxyless) | reCAPTCHA v2 携带挑战绑定的 `s` 参数 |
| `ReCaptchaV2EnterpriseTaskProxyless` | all | [示例](#ReCaptchaV2EnterpriseTaskProxyless) | reCAPTCHA v2 企业版 |
| `ReCaptchaV2SEnterpriseTaskProxyless` | all | [示例](#ReCaptchaV2SEnterpriseTaskProxyless) | reCAPTCHA v2 企业版，并携带 `s` 参数 |
| `ReCaptchaV2Classification` | sync | [示例](#ReCaptchaV2Classification) | reCAPTCHA v2 图片识别 |

### reCAPTCHA v3

| 任务类型 | 支持方式 | 示例 | 描述 |
| :-: | :---: | :-: | --- |
| `ReCaptchaV3TaskProxyless` | all | [示例](#ReCaptchaV3TaskProxyless) | reCAPTCHA v3 解决方案 |
| `ReCaptchaV3TaskProxylessS9` | all | [示例](#ReCaptchaV3TaskProxylessS9) | reCAPTCHA v3，返回分值 ≥ 0.9 的 token |
| `ReCaptchaV3EnterpriseTaskProxyless` | all | [示例](#ReCaptchaV3EnterpriseTaskProxyless) | reCAPTCHA v3 企业版 |
| `ReCaptchaV3EnterpriseTaskProxylessS9` | all | [示例](#ReCaptchaV3EnterpriseTaskProxylessS9) | reCAPTCHA v3 企业版 返回分值 ≥ 0.9 的 token |

### FunCaptcha / Arkose Labs

| 任务类型 | 支持方式 | 示例 | 描述 |
| :-: | :---: | :-: | --- |
| `FuncaptchaTaskProxyless` | async | [示例](#FuncaptchaTaskProxyless) | FunCaptcha / Arkose Labs 解决方案 |
| `FunCaptchaClassification` | sync | [示例](#FunCaptchaClassification) | FunCaptcha 图片识别 |

### hCaptcha

| 任务类型 | 支持方式 | 示例 | 描述 |
| :-: | :---: | :-: | --- |
| `HCaptcha` | async | [示例](#HCaptcha) | hCaptcha 解决方案 |
| `HCaptchaClassification` | sync | [示例](#HCaptchaClassification) | hCaptcha 图片识别，支持单图与多图 |

### Cloudflare

| 任务类型 | 支持方式 | 示例 | 描述 |
| :-: | :---: | :-: | --- |
| `CloudFlare5STask` | async | [示例](#CloudFlare5STask) | CF 5 秒盾，**必须**传 `proxy` |
| `CloudFlareTurnstileTask` | async | [示例](#CloudFlareTurnstileTask) | Turnstile，返回 token |

### Akamai

| 任务类型 | 支持方式 | 示例 | 描述 |
| :-: | :---: | :-: | --- |
| `AkamaiWEBTaskProxyless` | sync | [示例](#AkamaiWEBTaskProxyless) | Akamai Web 解决方案 |
| `AkamaiSBSDTaskProxyless` | sync | [示例](#AkamaiSBSDTaskProxyless) | Akamai SBSD 解决方案 |

> Akamai Web 是多轮流程：把本轮返回的 `encodedata` 作为下一轮的 `encode_data` 传入。这两个拼写在线格式上就是不同的，SDK 原样保留服务端的定义，不做「修正」。

### DataDome

| 任务类型 | 支持方式 | 示例 | 描述 |
| :-: | :---: | :-: | --- |
| `DataDomeTaskProxyless` | sync | [示例](#DataDomeTaskProxyless) | 拦截后的挑战，分两步，靠 `step` 区分 |
| `DataDomeTagsTaskProxyless` | sync | [示例](#DataDomeTagsTaskProxyless) | 正常浏览路径上报指纹 |

### 其他

| 任务类型 | 支持方式 | 示例 | 描述 |
| :-: | :---: | :-: | --- |
| `PerimeterX` | async | [示例](#PerimeterX) | PerimeterX 放行 cookie |
| `IncapsulaTaskProxyless` | sync | [示例](#IncapsulaTaskProxyless) | Incapsula Reese84 载荷 |
| `TlsTask` | sync | [示例](#TlsTask) | HTTP TLS 转发请求，返回上游响应 |

## 📦 安装

默认特性同时包含同步与异步客户端：

```toml
[dependencies]
ezcapsolver-rs = "0.1"
```

只启用其中一个：

```toml
[dependencies]
ezcapsolver-rs = { version = "0.1", default-features = false, features = ["blocking"] }
# 或
ezcapsolver-rs = { version = "0.1", default-features = false, features = ["async"] }
```

关闭默认特性后，任务模型与响应模型依然可用——只负责拼请求的 crate 不会被迫引入整套 HTTP 栈。

## 🚀 快速开始

配置中没有显式提供密钥时，客户端会自动从环境变量 `EZCAPTCHA_API_KEY` 读取。

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

## 📖 使用说明

### 同步/异步

**每个任务类型都有两个方法**，参数与返回类型完全相同，区别只是走哪个端点：

```rust,no_run
use ezcapsolver::{AsyncEzCapSolverClient, RecaptchaV2Task};

async fn run(client: &AsyncEzCapSolverClient, task: &RecaptchaV2Task) -> ezcapsolver::Result<()> {
  // 创建 + 轮询
  let solved = client.solve_recaptcha_v2_task_proxyless(task).await?;
  assert!(solved.task_id.is_some());

  // 同步端点，一次请求拿结果
  let solved = client.sync_solve_recaptcha_v2_task_proxyless(task).await?;
  assert!(solved.task_id.is_none());   // 同步端点不分配任务 ID
  Ok(())
}
```

### 代理格式

接受 `proxy` 的任务类型有两种格式，取决于任务类型：

| 格式 | 形态 | 适用 |
| --- | --- | --- |
| `NORMAL` | `protocol://username:password@host:port` | 除 FunCaptcha 外全部 |
| `FUN` | `protocol://host:port:username:password` | 仅 `FuncaptchaTaskProxyless` |

`protocol` 取 `http`、`https` 或 `socks5`。**用户名和密码都必填**——无认证代理会被服务端判为非法
——且 host 不能是内网地址（`127.0.*`、`192.168.*`、`172.16.*`、`10.0.*`）。字段本身可选时留空没问题，
只有「非空但格式不对」才会被拒。


### reCAPTCHA v2

[reCAPTCHA v2 API 文档](https://docs.ezxlabs.com/zh/docs/captcha/api/recaptcha-v2)

前五个类型共用 `RecaptchaV2Task` 与 `RecaptchaSolution`，只有方法名不同。

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

参数与普通 v2 完全一致，走高分队列，返回分值 ≥ 0.9 的 token。

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

携带挑战绑定的 `s` 参数。该参数并非强制，不传时行为与普通 v2 一致。

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

企业版。若站点用了 `data-s` 之外的企业参数，放进 `extra` 透传即可。

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

企业版，并携带 `s` 参数。

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

[reCAPTCHA v2 图片识别 API 文档](https://docs.ezxlabs.com/zh/docs/captcha/api/recaptcha-v2-classification)

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
      println!("选中格子 {:?}", solution.objects);
  } else if solution.is_single() {
      println!("是否含目标: {}", solution.has_object);
  } else {
      println!("类型: {}, 透传字段: {:?}", solution.r#type, solution.extra);
  }
  Ok(())
}
```

### reCAPTCHA v3

[reCAPTCHA v3 API 文档](https://docs.ezxlabs.com/zh/docs/captcha/api/recaptcha-v3)

四个类型共用 `RecaptchaV3Task` 与 `RecaptchaSolution`。`page_action` 要与页面上
`grecaptcha.execute` 传的 `action` 一致，否则站点侧校验会失败。

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

[FunCaptcha API 文档](https://docs.ezxlabs.com/zh/docs/captcha/api/funcaptcha)

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

> FunCaptcha 是唯一使用 `FUN` 代理格式的类型：`protocol://host:port:username:password`，
> 账号密码在 host 之后而不是之前。

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

  // 该类型的结果形态尚未确认，worker 返回的字段全部落在 extra 里。
  println!("{:?}", solution.extra);
  Ok(())
}
```

### hCaptcha

[hCaptcha API 文档](https://docs.ezxlabs.com/zh/docs/captcha/api/hcaptcha)

<a id="HCaptcha"></a>

#### HCaptcha

hCaptcha 的 token 字段是 `generated_pass_uuid`，不是 `token`——这是服务端的命名，SDK 原样保留。

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

单图用 `image`，多图用 `images`，两者都是可选字段。

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

  // 形态同样未确认，全部字段在 extra 里。
  println!("{:?}", solution.extra);
  Ok(())
}
```

### Cloudflare

<a id="CloudFlare5STask"></a>

#### CloudFlare5STask

[Cloudflare 5S API 文档](https://docs.ezxlabs.com/zh/docs/captcha/api/cloudflare-5s)

5 秒盾**必须**传 `proxy`，而且返回的不是单个 token，而是要回放到目标站点的 header 与放行 cookie：

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
  println!("TLS 指纹 = {}", solution.tls_version);
  Ok(())
}
```

<a id="CloudFlareTurnstileTask"></a>

#### CloudFlareTurnstileTask

[Cloudflare Turnstile API 文档](https://docs.ezxlabs.com/zh/docs/captcha/api/turnstile)

Turnstile 的 `proxy` 是可选的，返回单个 token。

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

[Akamai Web API 文档](https://docs.ezxlabs.com/zh/docs/captcha/api/akamai-web)

Akamai Web 是多轮流程：把本轮返回的 `encodedata` 作为下一轮的 `encode_data` 传回去。
两个拼写在线格式上就是不同的，SDK 原样保留服务端的定义。

```rust,no_run
use ezcapsolver::{AkamaiWebSolution, AkamaiWebTask, AsyncEzCapSolverClient};

async fn run(client: &AsyncEzCapSolverClient) -> Result<(), Box<dyn std::error::Error>> {
  let mut encode_data = String::new();

  for index in 1..=3 {
      let task = AkamaiWebTask::builder()
          .page_url("https://example.com")
          .ua("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
          .lang("zh-CN")
          .index(index)
          .encode_data(encode_data.clone())
          .build();
      let solution: AkamaiWebSolution = client
          .sync_solve_akamai_web_task_proxyless(&task)
          .await?
      .solution;

      println!("第 {index} 轮 payload = {}", solution.payload);
      encode_data = solution.encodedata;
  }
  Ok(())
}
```

<a id="AkamaiSBSDTaskProxyless"></a>

#### AkamaiSBSDTaskProxyless

[Akamai SBSD API 文档](https://docs.ezxlabs.com/zh/docs/captcha/api/akamai-sbsd)

单轮任务，六个字段全部必填。

```rust,no_run
use ezcapsolver::{AkamaiSbsdSolution, AkamaiSbsdTask, AsyncEzCapSolverClient};

async fn run(client: &AsyncEzCapSolverClient, script_base64: String) -> Result<(), Box<dyn std::error::Error>> {
  let task = AkamaiSbsdTask::builder()
      .page_url("https://example.com")
      .sbsd_url("https://example.com/.well-known/sbsd")
      .bm_so("value-from-the-page")
      .ua("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
      .lang("zh-CN")
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

DataDome 拦截后的挑战分两步，两步共用 `DataDomeTask`，靠 `step` 区分；`DataDomeSolution`
里哪些字段有值取决于当前是哪一步：

```rust,no_run
use ezcapsolver::{AsyncEzCapSolverClient, DataDomeSolution, DataDomeStep, DataDomeTask};

async fn run(client: &AsyncEzCapSolverClient, html_b64: String) -> Result<(), Box<dyn std::error::Error>> {
  // 第一步：从被拦截页面拿到挑战地址。
  let task = DataDomeTask::builder()
      .html_b64(html_b64)
      .step(DataDomeStep::One)
      .build();
  let solution: DataDomeSolution = client.sync_solve_data_dome_task_proxyless(&task).await?.solution;

  println!("挑战地址 = {:?}", solution.url);

  // 第二步换成 DataDomeStep::Two，结果里带回校验用的 body。
  Ok(())
}
```

<a id="DataDomeTagsTaskProxyless"></a>

#### DataDomeTagsTaskProxyless

正常浏览路径上报指纹，字段名是 camelCase，与上面的 `DataDomeTask` 不同。

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

### 其他

<a id="PerimeterX"></a>

#### PerimeterX

[PerimeterX API 文档](https://docs.ezxlabs.com/zh/docs/captcha/api/perimeterx)

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

[Incapsula API 文档](https://docs.ezxlabs.com/zh/docs/captcha/api/incapsula)

```rust,no_run
use ezcapsolver::{AsyncEzCapSolverClient, IncapsulaSolution, IncapsulaTask};

async fn run(client: &AsyncEzCapSolverClient, script: String) -> Result<(), Box<dyn std::error::Error>> {
  let task = IncapsulaTask::builder()
      .page_url("https://example.com")
      .script(script)
      .ua("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
      .accept_language("zh-CN,zh;q=0.9")
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

[TLS 转发 API 文档](https://docs.ezxlabs.com/zh/docs/captcha/api/tls-forward)

这个类型不解验证码，而是借 worker 的 TLS 指纹发一次 HTTP 请求，把上游响应原样带回来：

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

  println!("HTTP {}，响应体 {} 字节", solution.status, solution.body.len());
  Ok(())
}
```

### 额外参数

每个任务模型都有一个 `extra` 映射，其中的键值会被平铺进任务 JSON。服务端之后新增的参数不需要升级 SDK 就能用。保留字段 `type` 始终由 SDK 控制：

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
// 本版本未建模的参数，或服务端之后才上线的参数
task.extra.insert("futureFlag".to_owned(), json!(true));
```

同名时已声明的字段优先于 `extra`，所以往里加东西不会悄悄改掉显式设置过的参数。

每个任务结果也都有一个 `extra` 字段映射, 会把映射体中不存在，但服务端响应中存在的新字段传递回来

```rust,no_run
use ezcapsolver::{AsyncEzCapSolverClient, Cloudflare5sTask};

async fn run(client: &AsyncEzCapSolverClient, task: &Cloudflare5sTask) -> Result<(), Box<dyn std::error::Error>> {
  let solved = client.solve_cloudflare_5s_task(task).await?;
  let solution = &solved.solution;

  for (name, value) in &solution.cookies {
      println!("{name}={value}");
  }
  println!("指纹：{}", solution.tls_version);

  if let Some(value) = solution.extra.get("aFieldAddedLater") {
      println!("{value}");
  }
  Ok(())
}
```

### 定制化任务类型

任务类型是**开放集合**。直接把类型名当字符串传，参数用任何能序列化成 JSON 对象的值：

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

如果官方提供了新的验证码类型，但 SDK 还未同步更新时，可使用此方案临时代替。

`solve` 创建异步任务并轮询结果。对于支持同步端点的任务类型，使用 `sync_solve`，一次请求即可拿到结果：

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

两个方法均返回 `Solved<Solution>`，包含 `request_id`，并在 `raw` 中保留原始结果。`EzCapSolverClient` 提供相同方法，调用时去掉 `.await` 即可。`sync_solve` 使用 `sync_timeout`，不会轮询。

## ⚙️ 配置

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

所有配置项均可省略，`ClientConfig::builder().build()` 与 `ClientConfig::default()` 一致。使用 `.client_key("your-client-key")` 显式设置密钥；未设置时，客户端在创建时读取 `EZCAPTCHA_API_KEY`。字符串 setter 接受 `&str` 或 `String`，`maybe_*` 方法可接收可选值。`build()` 直接返回配置，校验仍在创建客户端时执行。原有 `with_*` 方法继续可用。

| 配置项 | 默认值 | 作用范围 |
| --- | :---: | --- |
| `timeout` | 30 秒 | 异步端点与余额查询 |
| `sync_timeout` | 240 秒 | 同步任务端点，比服务端 180 秒的 Worker deadline 留了余量 |
| `polling` | 3 秒 × 50 次 | 结果查询，单任务上限 150 秒 |
| `proxy` | 无 | **SDK 自身出网**使用，与任务参数里的 `proxy` 无关 |
| `async_base_url` | `https://api.ez-captcha.com` | 异步任务与余额查询 |
| `sync_base_url` | `https://sync.ez-captcha.com` | 同步任务；服务端把两套部署拆开了 |

用 `.with_base_urls(async_url, sync_url)` 改 base URL，可以指向私有网关或测试服务器。要自带 HTTP 客户端（共享连接池、装中间件、换 TLS 配置），用 `with_http_client(config, http)` 构造客户端，该客户端自己的 transport 与 header 会原样生效。

服务端只保留任务结果 **5 分钟**。超出这个窗口的轮询预算不可能成功，对更早的任务调 `wait_for_result` 会拿到 `ERROR_TASK_NOT_EXIST`。

## ⚠️ 错误处理

```rust
use ezcapsolver::Error;

fn inspect(error: Error) {
  match error {
      // 服务端明确拒绝了请求。错误码、描述、HTTP 状态码都在；
      // 参数校验失败还会带上逐字段的错误信息。
      Error::EzError(api) => eprintln!(
          "{:?}: {:?} (HTTP {})",
          api.error_code, api.error_description, api.http_status
      ),

      // 轮询次数用尽。任务可能仍在执行，task_id 可用于后续查询。
      Error::PollingExhausted { task_id, attempts, .. } => {
          eprintln!("任务 {task_id} 轮询 {attempts} 次后仍未完成")
      }

      // 任务已创建并扣费，但等待中断了。
      Error::WaitInterrupted { source, .. } => eprintln!("等待中断：{source}"),

      // Worker 返回了本版本未建模的形态，原始值随错误一起带出。
      Error::SolutionDecode { raw, .. } => eprintln!("形态非预期：{raw}"),

      other => eprintln!("{other}"),
  }
}
```

不论是哪种失败，判断「有没有一个**已扣费**的任务还能救回来」只需要问一句：

```rust
if let Some(task_id) = error.task_id() {
    // 任务在服务端，结果自创建起保留五分钟。再等一次是免费的，
    // 重新创建任务要再扣一次费。
    let result = client.wait_for_result(task_id).await?;
}
```

返回 `None` 表示没有扣过费，也就没有东西需要救。id 存在哪个变体上属于实现细节。


本 SDK **不替你重发请求**：创建任务会扣费且非幂等，重试策略归调用方。`EzError` 上有三个判定方法提供决策依据：

| 方法 | 何时为真 | 该怎么做 |
| --- | --- | --- |
| `is_authentication_error()` | `ERROR_KEY_DOES_NOT_EXIST`、`ERROR_KEY_NOT_AVAILABLE`、`ERROR_ZERO_BALANCE` | **停手**。服务端按密钥计数，一分钟 30 次即封禁——退避重试只会更快撞上 |
| `is_terminal()` | 同样的请求重发会得到同样的失败 | 改请求，重试不会有任何变化 |
| `is_rate_limited()` | `ERROR_REQUEST_LIMIT`、`ERROR_REQUEST_BANNED` | 等一下再问，两者都会自行解除 |

`is_terminal()` 对本版本未见过的码返回 `false`：不能让一个新的临时性错误码劝退一次本可以成功的重试。`is_rate_limited()` 比它窄得多——它只认这两个码，而不是「所有不确定终局的码」。

`wait_for_result` 唯一会重试的就是被限流的那次轮询。`ERROR_REQUEST_LIMIT` 与 `ERROR_REQUEST_BANNED` 拒的是**这次查询**，不是任务：服务端在查任务之前就把请求挡回来了，任务仍在排队、也仍然扣着费。所以轮询循环消耗掉这一次机会后接着问，而不是把一个马上就要拿到的结果丢掉。其余任何 API 错误都是这次轮询的答案，会直接结束等待。

## 📝 日志

SDK 通过 `tracing` 输出诊断信息，由应用自行选择 subscriber 与过滤器。

| 级别 | 事件 |
| :---: | --- |
| `INFO` | 任务已创建、任务已完成 |
| `DEBUG` | 每个操作一行，每次轮询一行 |
| `TRACE` | 请求体，以及响应的状态码、耗时、大小与响应体 |

`TRACE` 用于排查接入故障。请求体在渲染时会把**任意嵌套深度**的 `clientKey` 与 `proxy` 替换为 `[REDACTED]`；而且只要没有 subscriber 打开 `TRACE`，请求体根本不会被渲染——默认日志级别下零开销。

过滤器要限定到本 crate，全局 `TRACE` 会把 SDK 的日志淹没在 HTTP 栈自己的输出里：

```rust,no_run
use tracing::Level;
use tracing_subscriber::{filter::Targets, prelude::*};

tracing_subscriber::registry()
    .with(tracing_subscriber::fmt::layer())
    .with(Targets::new().with_target("ezcapsolver", Level::DEBUG))
    .init();
```

## 🧪 可运行示例

可运行的示例在 [`examples/`](./examples) 目录。先设置 `EZCAPTCHA_API_KEY`，然后：

```bash
export EZCAPTCHA_API_KEY=your-client-key

cargo run --example recaptcha_v2_task_proxyless
cargo run --example cloud_flare_turnstile_task
cargo run --example tls_task

# 讲 SDK 本身而非某个任务类型
cargo run --example async_client_task     # 逐项列出全部客户端配置
cargo run --example blocking_client_task  # 同上，不需要运行时
cargo run --example concurrency           # 一个客户端跑多个任务
cargo run --example raw_usage             # 底层的创建/轮询工作流
cargo run --example logging_and_errors    # tracing 配置与各类失败的区分
```

每种任务类型一个文件，文件名取自线格式的任务类型名，与 Go、Python 两版逐个对应——完整清单见
[示例索引](./examples/README.zh-CN.md)。每个任务示例都标注了它覆盖的任务类型，并链接到官方接口文档的对应页面。需要代理的示例从
`EZCAPTCHA_PROXY` 读取；示例一律不硬编码凭据。

> 每次运行都会创建真实任务并**计费**，worker 失败也照扣。

## 🎛️ 特性开关

| 特性 | 默认 | 启用内容 |
| --- | :---: | --- |
| `async` | ✅ | `AsyncEzCapSolverClient`、Tokio、Reqwest |
| `blocking` | ✅ | `EzCapSolverClient`、Reqwest blocking |

两个特性共用同一套 transport 实现，因此两个客户端的行为不会产生漂移。

## 🛠️ 开发

最低支持的 Rust 版本：**1.96.0**，edition 2024。

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo doc --no-deps --all-features
```

## 📄 许可证

基于 [Apache License 2.0](./LICENSE) 授权。
