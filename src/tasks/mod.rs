use std::collections::BTreeMap;

use serde_json::Value;

/// Gives a task model a `Serialize` that merges `extra` without letting it
/// shadow a declared field.
///
/// Every model carries `#[serde(remote = "Self")]`, which keeps the derived
/// implementation reachable as an inherent function. Calling it from here is
/// therefore not recursive — the same shape as the local `type wire T` alias
/// the Go SDK uses for this.
macro_rules! impl_task_serialize {
    ($($ty:ident),+ $(,)?) => {$(
        impl serde::Serialize for $ty {
            fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                $crate::wire::merge_extra(|s| <$ty>::serialize(self, s), &self.extra, serializer)
            }
        }
    )+};
}

mod akamai;
mod cloudflare;
mod datadome;
mod funcaptcha;
mod hcaptcha;
mod incapsula;
mod perimeterx;
mod recaptcha;
mod tls;

pub use akamai::{AkamaiSbsdTask, AkamaiSbsdTaskBuilder, AkamaiWebTask, AkamaiWebTaskBuilder};
pub use cloudflare::{
    Cloudflare5sTask, Cloudflare5sTaskBuilder, CloudflareTurnstileTask,
    CloudflareTurnstileTaskBuilder,
};
pub use datadome::{
    DataDomeJsType, DataDomeStep, DataDomeTagsTask, DataDomeTagsTaskBuilder, DataDomeTask,
    DataDomeTaskBuilder,
};
pub use funcaptcha::{
    FunCaptchaClassificationTask, FunCaptchaClassificationTaskBuilder, FunCaptchaTask,
    FunCaptchaTaskBuilder,
};
pub use hcaptcha::{
    HcaptchaClassificationTask, HcaptchaClassificationTaskBuilder, HcaptchaTask,
    HcaptchaTaskBuilder,
};
pub use incapsula::{IncapsulaTask, IncapsulaTaskBuilder};
pub use perimeterx::{PerimeterXTask, PerimeterXTaskBuilder};
pub use recaptcha::{
    RecaptchaV2ClassificationTask, RecaptchaV2ClassificationTaskBuilder, RecaptchaV2Task,
    RecaptchaV2TaskBuilder, RecaptchaV3Task, RecaptchaV3TaskBuilder,
};
pub use tls::{TlsForwardTask, TlsForwardTaskBuilder, TlsHttpMethod};

impl_task_serialize!(
    AkamaiSbsdTask,
    AkamaiWebTask,
    Cloudflare5sTask,
    CloudflareTurnstileTask,
    DataDomeTagsTask,
    DataDomeTask,
    FunCaptchaClassificationTask,
    FunCaptchaTask,
    HcaptchaClassificationTask,
    HcaptchaTask,
    IncapsulaTask,
    PerimeterXTask,
    RecaptchaV2ClassificationTask,
    RecaptchaV2Task,
    RecaptchaV3Task,
    TlsForwardTask,
);

/// Additional task properties forwarded to the worker without SDK validation.
///
/// Keys here are sent as-is next to the declared fields. A key that a declared
/// field already produced is dropped rather than applied, so a pass-through
/// value can add a parameter but never rewrite one the model states. A field
/// left unset produces no key, and stays available for `extra` to fill.
pub type ExtraFields = BTreeMap<String, Value>;
