use serde::de::DeserializeOwned;

use crate::{Error, Result, Solved, TaskResult, TaskStatus};

/// Turns a synchronous endpoint result into the shape both modes return.
///
/// The identifier comes from the response: `/createSyncTask` assigns one and
/// returns it on both the success and the failure path.
pub(crate) fn decode_sync_result<T>(result: TaskResult) -> Result<Solved<T>>
where
    T: DeserializeOwned,
{
    // Decode through a borrow, then move `meta` and `solution` out, rather
    // than cloning the whole result just to keep `raw`.
    let solution: T = match &result.status {
        TaskStatus::Ready => result.deserialize_solution()?,
        TaskStatus::Processing => {
            return Err(Error::UnexpectedResponse(
                "task result is still processing".to_owned(),
            ));
        }
        TaskStatus::Error => {
            return Err(Error::UnexpectedResponse(
                "task result returned status `error` without an API error".to_owned(),
            ));
        }
    };
    Ok(Solved {
        task_id: result.task_id,
        request_id: result.meta.request_id,
        solution,
        raw: result.solution,
    })
}

/// Every task type, with both of its convenience method names.
///
/// Columns: the mode the service documents, the polling method, the synchronous
/// method, the request model, the wire type, and the solution model.
///
/// The documented mode does **not** restrict which methods exist — every type
/// gets both. It only decides which one the generated documentation points at,
/// because sending a type to the endpoint the service does not serve can be
/// rejected. Such a rejection is refused before billing, so it costs a round
/// trip rather than a task.
macro_rules! for_each_task_solver {
    ($callback:ident) => {
        $callback!(
            polling,
            solve_recaptcha_v2_task_proxyless,
            RecaptchaV2Task,
            RecaptchaV2TaskProxyless,
            crate::RecaptchaSolution
        );
        $callback!(
            polling,
            solve_recaptcha_v2_task_proxyless_s9,
            RecaptchaV2Task,
            RecaptchaV2TaskProxylessS9,
            crate::RecaptchaSolution
        );
        $callback!(
            polling,
            solve_recaptcha_v2_s_task_proxyless,
            RecaptchaV2Task,
            RecaptchaV2STaskProxyless,
            crate::RecaptchaSolution
        );
        $callback!(
            polling,
            solve_recaptcha_v2_enterprise_task_proxyless,
            RecaptchaV2Task,
            RecaptchaV2EnterpriseTaskProxyless,
            crate::RecaptchaSolution
        );
        $callback!(
            polling,
            solve_recaptcha_v2_s_enterprise_task_proxyless,
            RecaptchaV2Task,
            RecaptchaV2SEnterpriseTaskProxyless,
            crate::RecaptchaSolution
        );
        $callback!(
            sync,
            solve_recaptcha_v2_classification,
            RecaptchaV2ClassificationTask,
            RecaptchaV2Classification,
            crate::ReClassificationSolution
        );
        $callback!(
            polling,
            solve_recaptcha_v3_task_proxyless,
            RecaptchaV3Task,
            RecaptchaV3TaskProxyless,
            crate::RecaptchaSolution
        );
        $callback!(
            polling,
            solve_recaptcha_v3_task_proxyless_s9,
            RecaptchaV3Task,
            RecaptchaV3TaskProxylessS9,
            crate::RecaptchaSolution
        );
        $callback!(
            polling,
            solve_recaptcha_v3_enterprise_task_proxyless,
            RecaptchaV3Task,
            RecaptchaV3EnterpriseTaskProxyless,
            crate::RecaptchaSolution
        );
        $callback!(
            polling,
            solve_recaptcha_v3_enterprise_task_proxyless_s9,
            RecaptchaV3Task,
            RecaptchaV3EnterpriseTaskProxylessS9,
            crate::RecaptchaSolution
        );
        $callback!(
            polling,
            solve_funcaptcha_task_proxyless,
            FunCaptchaTask,
            FuncaptchaTaskProxyless,
            crate::FunCaptchaSolution
        );
        $callback!(
            sync,
            solve_funcaptcha_classification,
            FunCaptchaClassificationTask,
            FuncaptchaClassification,
            crate::FunCaptchaClassificationSolution
        );
        $callback!(
            polling,
            solve_perimeter_x,
            PerimeterXTask,
            PerimeterX,
            crate::PerimeterXSolution
        );
        $callback!(
            polling,
            solve_hcaptcha,
            HcaptchaTask,
            Hcaptcha,
            crate::HcaptchaSolution
        );
        $callback!(
            sync,
            solve_hcaptcha_classification,
            HcaptchaClassificationTask,
            HcaptchaClassification,
            crate::HcaptchaClassificationSolution
        );
        $callback!(
            sync,
            solve_akamai_web_task_proxyless,
            AkamaiWebTask,
            AkamaiWebTaskProxyless,
            crate::AkamaiWebSolution
        );
        $callback!(
            sync,
            solve_akamai_sbsd_task_proxyless,
            AkamaiSbsdTask,
            AkamaiSbsdTaskProxyless,
            crate::AkamaiSbsdSolution
        );
        $callback!(
            sync,
            solve_tls_task,
            TlsForwardTask,
            TlsTask,
            crate::TlsForwardSolution
        );
        $callback!(
            polling,
            solve_cloudflare_5s_task,
            Cloudflare5sTask,
            Cloudflare5sTask,
            crate::Cloudflare5sSolution
        );
        $callback!(
            polling,
            solve_cloudflare_turnstile_task,
            CloudflareTurnstileTask,
            CloudflareTurnstileTask,
            crate::CloudflareTurnstileSolution
        );
        $callback!(
            sync,
            solve_data_dome_task_proxyless,
            DataDomeTask,
            DataDomeTaskProxyless,
            crate::DataDomeSolution
        );
        $callback!(
            sync,
            solve_data_dome_tags_task_proxyless,
            DataDomeTagsTask,
            DataDomeTagsTaskProxyless,
            crate::DataDomeSolution
        );
        $callback!(
            sync,
            solve_incapsula_task_proxyless,
            IncapsulaTask,
            IncapsulaTaskProxyless,
            crate::IncapsulaSolution
        );
    };
}

pub(crate) use for_each_task_solver;
