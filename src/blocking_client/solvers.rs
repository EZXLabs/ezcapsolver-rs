use super::EzCapSolverClient;
use crate::{
    AkamaiSbsdTask, AkamaiWebTask, Cloudflare5sTask, CloudflareTurnstileTask, DataDomeTagsTask,
    DataDomeTask, FunCaptchaClassificationTask, FunCaptchaTask, HcaptchaClassificationTask,
    HcaptchaTask, IncapsulaTask, PerimeterXTask, RecaptchaV2ClassificationTask, RecaptchaV2Task,
    RecaptchaV3Task, Result, Solved, TaskType, TlsForwardTask,
    task_solvers::{decode_sync_result, for_each_task_solver},
};

/// Two methods per task type, one per execution mode.
///
/// `solve_*` creates the task and polls for its result; `sync_solve_*` runs the
/// same task through the synchronous endpoint, which answers on the creating
/// request. Both take the same request model and return the same [`Solved<T>`].
///
/// The synchronous name is concatenated from the polling one, so the task table
/// stays the only place a method name is written down.
macro_rules! define_solver {
    ($mode:ident, $name:ident, $task:ty, $task_type:ident, $solution:ty) => {
        /// Creates the task, polls until it finishes, and returns its decoded solution.
        ///
        /// The `sync_`-prefixed method runs the same type through the
        /// synchronous endpoint instead.
        pub fn $name(&self, task: &$task) -> Result<Solved<$solution>> {
            let solved = self.solve(TaskType::$task_type, task)?;
            let solution = solved.solution.deserialize()?;
            Ok(Solved {
                task_id: solved.task_id,
                request_id: solved.request_id,
                solution,
                raw: solved.raw,
            })
        }

        pastey::paste! {
            /// Runs the task through the synchronous endpoint and returns its
            /// decoded solution.
            ///
            /// [`Solved::task_id`] carries the identifier this endpoint assigns
            /// and returns alongside the result.
            ///
            /// Whether a given type is accepted on this endpoint is the
            /// service's decision; a type it does not serve is rejected with
            /// `ERROR_TASK_TYPE_NOT_ALLOWED` before anything is billed.
            pub fn [<sync_ $name>](&self, task: &$task) -> Result<Solved<$solution>> {
                let result = self.create_sync_task(TaskType::$task_type, task)?;
                decode_sync_result(result)
            }
        }
    };
}

impl EzCapSolverClient {
    for_each_task_solver!(define_solver);
}
