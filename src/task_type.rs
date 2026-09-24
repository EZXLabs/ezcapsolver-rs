use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};

/// Static task execution mode declared by the service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskMode {
    /// The task is created and queried through the asynchronous API.
    Async,
    /// The task returns its result from the synchronous API.
    Sync,
}

/// Known EzCaptchaSolver task type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum TaskType {
    /// ReCaptcha V2 proxyless task.
    #[default]
    RecaptchaV2TaskProxyless,
    /// ReCaptcha V2 high-score proxyless task.
    RecaptchaV2TaskProxylessS9,
    /// ReCaptcha V2 task carrying the `s` parameter.
    RecaptchaV2STaskProxyless,
    /// ReCaptcha V2 Enterprise proxyless task.
    RecaptchaV2EnterpriseTaskProxyless,
    /// ReCaptcha V2 Enterprise task carrying the `s` parameter.
    RecaptchaV2SEnterpriseTaskProxyless,
    /// ReCaptcha V2 image classification task.
    RecaptchaV2Classification,
    /// ReCaptcha V3 proxyless task.
    RecaptchaV3TaskProxyless,
    /// ReCaptcha V3 high-score proxyless task.
    RecaptchaV3TaskProxylessS9,
    /// ReCaptcha V3 Enterprise proxyless task.
    RecaptchaV3EnterpriseTaskProxyless,
    /// ReCaptcha V3 Enterprise high-score proxyless task.
    RecaptchaV3EnterpriseTaskProxylessS9,
    /// FunCaptcha proxyless task.
    FuncaptchaTaskProxyless,
    /// FunCaptcha image classification task.
    FuncaptchaClassification,
    /// PerimeterX task.
    PerimeterX,
    /// HCaptcha token task.
    Hcaptcha,
    /// HCaptcha image classification task.
    HcaptchaClassification,
    /// Akamai Web task.
    AkamaiWebTaskProxyless,
    /// Akamai SBSD task.
    AkamaiSbsdTaskProxyless,
    /// TLS forwarding task.
    TlsTask,
    /// Cloudflare five-second challenge task.
    Cloudflare5sTask,
    /// Cloudflare Turnstile task.
    CloudflareTurnstileTask,
    /// DataDome challenge task.
    DataDomeTaskProxyless,
    /// DataDome tags task.
    DataDomeTagsTaskProxyless,
    /// Incapsula Reese84 task.
    IncapsulaTaskProxyless,
}

impl TaskType {
    /// Task types declared by the current EzCaptchaSolver service contract.
    pub const KNOWN: &'static [Self] = &[
        Self::RecaptchaV2TaskProxyless,
        Self::RecaptchaV2TaskProxylessS9,
        Self::RecaptchaV2STaskProxyless,
        Self::RecaptchaV2EnterpriseTaskProxyless,
        Self::RecaptchaV2SEnterpriseTaskProxyless,
        Self::RecaptchaV2Classification,
        Self::RecaptchaV3TaskProxyless,
        Self::RecaptchaV3TaskProxylessS9,
        Self::RecaptchaV3EnterpriseTaskProxyless,
        Self::RecaptchaV3EnterpriseTaskProxylessS9,
        Self::FuncaptchaTaskProxyless,
        Self::FuncaptchaClassification,
        Self::PerimeterX,
        Self::Hcaptcha,
        Self::HcaptchaClassification,
        Self::AkamaiWebTaskProxyless,
        Self::AkamaiSbsdTaskProxyless,
        Self::TlsTask,
        Self::Cloudflare5sTask,
        Self::CloudflareTurnstileTask,
        Self::DataDomeTaskProxyless,
        Self::DataDomeTagsTaskProxyless,
        Self::IncapsulaTaskProxyless,
    ];

    /// Returns the canonical task type string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::RecaptchaV2TaskProxyless => "ReCaptchaV2TaskProxyless",
            Self::RecaptchaV2TaskProxylessS9 => "ReCaptchaV2TaskProxylessS9",
            Self::RecaptchaV2STaskProxyless => "ReCaptchaV2STaskProxyless",
            Self::RecaptchaV2EnterpriseTaskProxyless => "ReCaptchaV2EnterpriseTaskProxyless",
            Self::RecaptchaV2SEnterpriseTaskProxyless => "ReCaptchaV2SEnterpriseTaskProxyless",
            Self::RecaptchaV2Classification => "ReCaptchaV2Classification",
            Self::RecaptchaV3TaskProxyless => "ReCaptchaV3TaskProxyless",
            Self::RecaptchaV3TaskProxylessS9 => "ReCaptchaV3TaskProxylessS9",
            Self::RecaptchaV3EnterpriseTaskProxyless => "ReCaptchaV3EnterpriseTaskProxyless",
            // The catalog writes this one with a lowercase c. Every language SDK
            // normalises it so one capitalisation runs through the whole ReCaptcha
            // family; the service matches task types case-insensitively, so it
            // reaches the same worker. Do not "fix" it back.
            Self::RecaptchaV3EnterpriseTaskProxylessS9 => "ReCaptchaV3EnterpriseTaskProxylessS9",
            Self::FuncaptchaTaskProxyless => "FuncaptchaTaskProxyless",
            Self::FuncaptchaClassification => "FunCaptchaClassification",
            Self::PerimeterX => "PerimeterX",
            Self::Hcaptcha => "HCaptcha",
            Self::HcaptchaClassification => "HCaptchaClassification",
            Self::AkamaiWebTaskProxyless => "AkamaiWEBTaskProxyless",
            Self::AkamaiSbsdTaskProxyless => "AkamaiSBSDTaskProxyless",
            Self::TlsTask => "TlsTask",
            Self::Cloudflare5sTask => "CloudFlare5STask",
            Self::CloudflareTurnstileTask => "CloudFlareTurnstileTask",
            Self::DataDomeTaskProxyless => "DataDomeTaskProxyless",
            Self::DataDomeTagsTaskProxyless => "DataDomeTagsTaskProxyless",
            Self::IncapsulaTaskProxyless => "IncapsulaTaskProxyless",
        }
    }

    /// Returns the execution mode the service documents for this type.
    ///
    /// This is **informational**. Every type has both a `solve_*` method that
    /// polls and a `sync_solve_*` method that uses the synchronous endpoint, and
    /// nothing in the SDK consults this value to pick between them.
    ///
    /// It matters because the service can reject a type on the endpoint it does
    /// not serve — so this is the mode to follow when you have no reason to
    /// prefer the other. Such a rejection is refused before billing, so it
    /// costs a round trip rather than a task.
    #[must_use]
    pub const fn mode(&self) -> Option<TaskMode> {
        match self {
            Self::RecaptchaV2TaskProxyless
            | Self::RecaptchaV2TaskProxylessS9
            | Self::RecaptchaV2STaskProxyless
            | Self::RecaptchaV2EnterpriseTaskProxyless
            | Self::RecaptchaV2SEnterpriseTaskProxyless
            | Self::RecaptchaV3TaskProxyless
            | Self::RecaptchaV3TaskProxylessS9
            | Self::RecaptchaV3EnterpriseTaskProxyless
            | Self::RecaptchaV3EnterpriseTaskProxylessS9
            | Self::FuncaptchaTaskProxyless
            | Self::PerimeterX
            | Self::Hcaptcha
            | Self::Cloudflare5sTask
            | Self::CloudflareTurnstileTask => Some(TaskMode::Async),
            Self::RecaptchaV2Classification
            | Self::FuncaptchaClassification
            | Self::HcaptchaClassification
            | Self::AkamaiWebTaskProxyless
            | Self::AkamaiSbsdTaskProxyless
            | Self::TlsTask
            | Self::DataDomeTaskProxyless
            | Self::DataDomeTagsTaskProxyless
            | Self::IncapsulaTaskProxyless => Some(TaskMode::Sync),
        }
    }

    fn from_wire(value: &str) -> Option<Self> {
        Self::KNOWN
            .iter()
            .find(|task_type| task_type.as_str().eq_ignore_ascii_case(value))
            .copied()
    }
}

/// Source of the `type` value sent with a task.
///
/// Implemented for [`TaskType`] and for plain strings, so a task type the
/// service adds after this release can still be used without waiting for an
/// SDK update: pass the type name directly.
///
/// ```no_run
/// # use ezcapsolver::{AsyncEzCapSolverClient, TaskType};
/// # async fn run(client: &AsyncEzCapSolverClient) -> ezcapsolver::Result<()> {
/// # let task = serde_json::json!({});
/// // A modelled type
/// client.create_task(TaskType::RecaptchaV2TaskProxyless, &task).await?;
/// // The escape hatch: a type the service added that the SDK has yet to model
/// client.create_task("BrandNewTaskType", &task).await?;
/// # Ok(())
/// # }
/// ```
pub trait TaskTypeName {
    /// Returns the wire value written to the `type` field.
    fn task_type_name(&self) -> &str;
}

impl TaskTypeName for TaskType {
    fn task_type_name(&self) -> &str {
        self.as_str()
    }
}

impl TaskTypeName for str {
    fn task_type_name(&self) -> &str {
        self
    }
}

impl TaskTypeName for String {
    fn task_type_name(&self) -> &str {
        self
    }
}

impl<T> TaskTypeName for &T
where
    T: TaskTypeName + ?Sized,
{
    fn task_type_name(&self) -> &str {
        (*self).task_type_name()
    }
}

impl fmt::Display for TaskType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl Serialize for TaskType {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for TaskType {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::from_wire(&value)
            .ok_or_else(|| D::Error::custom(format!("unknown task type `{value}`")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_type_deserialization_is_case_insensitive() -> Result<(), serde_json::Error> {
        let task_type: TaskType = serde_json::from_str(r#""recaptchav2taskproxyless""#)?;
        assert_eq!(task_type, TaskType::RecaptchaV2TaskProxyless);
        assert_eq!(
            serde_json::to_string(&task_type)?,
            r#""ReCaptchaV2TaskProxyless""#
        );
        Ok(())
    }

    #[test]
    fn a_plain_string_can_stand_in_for_a_task_type() {
        // The escape hatch: a new service type needs no SDK release.
        assert_eq!("BrandNewTaskType".task_type_name(), "BrandNewTaskType");
        assert_eq!(
            TaskType::RecaptchaV2TaskProxyless.task_type_name(),
            "ReCaptchaV2TaskProxyless"
        );
    }

    #[test]
    fn unknown_type_is_rejected() {
        let error = serde_json::from_str::<TaskType>(r#""FutureTask""#).unwrap_err();
        assert!(error.to_string().contains("unknown task type `FutureTask`"));
    }
}
