use std::{
    fmt,
    time::{Duration, Instant},
};

use reqwest::header::HeaderValue;
use serde::{Serialize, de::DeserializeOwned};
use tracing::{debug, info, warn};

use crate::{
    ClientConfig, CreateTaskResponse, Error, PollingConfig, Result, Solution, Solved, TaskResult,
    TaskStatus, TaskTypeName,
    request::{CreateTaskRequest, QueryBalanceRequest, QueryTaskResultRequest, TaskPayload},
    response::BalanceResponse,
    transport::{
        CLIENT_KEY_HEADER, client_key_header, endpoint, log_request, log_response,
        parse_api_response,
    },
};

/// Non-blocking client for asynchronous and synchronous EzCaptchaSolver API operations.
#[derive(Clone)]
pub struct AsyncEzCapSolverClient {
    client_key: String,
    client_key_header: HeaderValue,
    config: ClientConfig,
    http: reqwest::Client,
}

impl fmt::Debug for AsyncEzCapSolverClient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AsyncEzCapSolverClient")
            .field("client_key", &"[REDACTED]")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl AsyncEzCapSolverClient {
    /// Creates a client with default configuration.
    ///
    /// The client key is loaded from `EZCAPTCHA_API_KEY`.
    pub fn new() -> Result<Self> {
        Self::with_config(ClientConfig::default())
    }

    /// Creates a client with the supplied configuration.
    ///
    /// If the configuration does not contain an explicit client key,
    /// `EZCAPTCHA_API_KEY` is loaded from the environment.
    pub fn with_config(config: ClientConfig) -> Result<Self> {
        let client_key = config.resolve_client_key()?;
        let client_key_header = client_key_header(&client_key)?;
        config.validate()?;
        let http = build_http_client(&config)?;
        Ok(Self {
            client_key,
            client_key_header,
            config,
            http,
        })
    }

    /// Creates a client with an explicit client key and default configuration.
    pub fn with_client_key(client_key: impl Into<String>) -> Result<Self> {
        Self::with_config(ClientConfig::default().with_client_key(client_key))
    }

    /// Creates a client that sends through a caller-supplied HTTP client.
    ///
    /// Use it to share a connection pool, or to install a middleware stack or a
    /// custom TLS setup. [`ClientConfig::proxy`] and [`ClientConfig::user_agent`]
    /// are **not** applied to it — the supplied client brings its own transport
    /// and headers. Per-request timeouts and the `X-API-Key` header still are.
    pub fn with_http_client(config: ClientConfig, http: reqwest::Client) -> Result<Self> {
        let client_key = config.resolve_client_key()?;
        let client_key_header = client_key_header(&client_key)?;
        config.validate()?;
        Ok(Self {
            client_key,
            client_key_header,
            config,
            http,
        })
    }

    /// Returns the active client configuration.
    #[must_use]
    pub const fn config(&self) -> &ClientConfig {
        &self.config
    }

    /// Creates an asynchronous API task without waiting for completion.
    ///
    /// The supplied task type is injected into the serialized parameter object.
    ///
    /// Most callers should use [`Self::solve`] to create the task and wait for
    /// its result in one call.
    pub async fn create_task<N, T>(&self, task_type: N, task: &T) -> Result<CreateTaskResponse>
    where
        N: TaskTypeName,
        T: Serialize + ?Sized,
    {
        let task_type = task_type.task_type_name();
        debug!(task_type, "creating task");
        let request = CreateTaskRequest {
            client_key: &self.client_key,
            app_id: self.config.app_id,
            task: TaskPayload {
                task_type,
                params: task,
            },
        };
        let response: CreateTaskResponse = self
            .post(
                &endpoint(&self.config.async_base_url, "/createTask"),
                &request,
                self.config.timeout,
            )
            .await?;
        if response.task_id.trim().is_empty() {
            return Err(Error::UnexpectedResponse(
                "createTask returned an empty taskId".to_owned(),
            ));
        }
        info!(task_id = %response.task_id, task_type, "task created");
        Ok(response)
    }

    /// Queries the current result of an asynchronous API task.
    ///
    /// This is called once per polling attempt, so it does not log on its own;
    /// the caller records the attempt and `TRACE` records the exchange.
    pub async fn get_task_result(&self, task_id: &str) -> Result<TaskResult> {
        let request = QueryTaskResultRequest {
            client_key: &self.client_key,
            task_id,
        };
        self.post(
            &endpoint(&self.config.async_base_url, "/getTaskResult"),
            &request,
            self.config.timeout,
        )
        .await
    }

    /// Waits for an existing asynchronous task using client polling settings.
    pub async fn wait_for_result(&self, task_id: &str) -> Result<TaskResult> {
        self.wait_for_result_with(task_id, self.config.polling)
            .await
    }

    /// Waits for an existing asynchronous task using per-call polling settings.
    pub async fn wait_for_result_with(
        &self,
        task_id: &str,
        polling: PollingConfig,
    ) -> Result<TaskResult> {
        polling.validate()?;
        for attempt in 1..=polling.max_attempts {
            // Wait before every query, including the first: a task that was
            // just created is still queued and would report `processing`.
            tokio::time::sleep(polling.interval).await;
            debug!(task_id = %task_id, attempt, "polling task result");
            let result = match self.get_task_result(task_id).await {
                Ok(result) => result,
                // Throttling says nothing about the task, which is still
                // queued. Spend the attempt and poll again rather than failing
                // a task that has already been billed. The attempt is spent on
                // purpose: interval x attempts is what keeps the whole wait
                // inside the five-minute window the result is held for.
                Err(Error::EzError(error)) if error.is_rate_limited() => {
                    warn!(
                        task_id = %task_id,
                        attempt,
                        error_code = error.error_code.as_deref().unwrap_or_default(),
                        "polling throttled, retrying"
                    );
                    continue;
                }
                Err(error) => return Err(error),
            };
            match &result.status {
                TaskStatus::Ready => {
                    info!(
                        task_id = %task_id,
                        attempts = attempt,
                        "task completed"
                    );
                    return Ok(result);
                }
                TaskStatus::Processing => {}
                TaskStatus::Error => {
                    return Err(Error::UnexpectedResponse(format!(
                        "task `{task_id}` returned status `error` without an API error"
                    )));
                }
            }
        }
        Err(Error::PollingExhausted {
            task_id: task_id.to_owned(),
            attempts: polling.max_attempts,
            interval: polling.interval,
        })
    }

    /// Creates an asynchronous task and waits for its terminal result.
    ///
    /// The supplied task type is injected into the serialized parameter object.
    ///
    /// This is the recommended high-level method for asynchronous tasks.
    pub async fn solve<N, T>(&self, task_type: N, task: &T) -> Result<Solved<Solution>>
    where
        N: TaskTypeName,
        T: Serialize + ?Sized,
    {
        self.solve_with(task_type, task, self.config.polling).await
    }

    /// Creates an asynchronous task and uses per-call polling settings.
    pub async fn solve_with<N, T>(
        &self,
        task_type: N,
        task: &T,
        polling: PollingConfig,
    ) -> Result<Solved<Solution>>
    where
        N: TaskTypeName,
        T: Serialize + ?Sized,
    {
        let created = self.create_task(task_type, task).await?;
        let task_id = created.task_id;
        let request_id = created.meta.request_id;
        let result = self
            .wait_for_result_with(&task_id, polling)
            .await
            .map_err(|error| error.with_task_context(task_id.clone(), request_id.clone()))?;
        Ok(Solved {
            task_id: Some(task_id),
            // The service assigns every HTTP request its own correlation id. The one from the
            // result query describes the response the caller is holding, so it wins;
            // creation's id is the fallback that keeps this field populated.
            request_id: result.meta.request_id.or(request_id),
            // On the general entry point the solution is the raw value already, so
            // both fields come from the same place.
            solution: result.solution.clone(),
            raw: result.solution,
        })
    }

    /// Runs a synchronous task and returns its raw solution without polling.
    ///
    /// Accepts a known task type or a custom type name, with parameters that
    /// serialize to a JSON object. Uses [`ClientConfig::sync_timeout`].
    ///
    /// Like [`Self::solve`], returns [`Solved<Solution>`], preserving the raw
    /// solution, including a missing field or JSON null. Both [`Solved::task_id`]
    /// and [`Solved::request_id`] come from the synchronous response.
    pub async fn sync_solve<N, T>(&self, task_type: N, task: &T) -> Result<Solved<Solution>>
    where
        N: TaskTypeName,
        T: Serialize + ?Sized,
    {
        let result = self.create_sync_task(task_type, task).await?;
        if result.status == TaskStatus::Error {
            return Err(Error::UnexpectedResponse(
                "task result returned status `error` without an API error".to_owned(),
            ));
        }
        Ok(Solved {
            task_id: result.task_id,
            request_id: result.meta.request_id,
            solution: result.solution.clone(),
            raw: result.solution,
        })
    }

    /// Creates a synchronous API task and returns its result.
    ///
    /// The supplied task type is injected into the serialized parameter object.
    /// Use [`Self::sync_solve`] to return the same aggregate as [`Self::solve`].
    pub async fn create_sync_task<N, T>(&self, task_type: N, task: &T) -> Result<TaskResult>
    where
        N: TaskTypeName,
        T: Serialize + ?Sized,
    {
        let task_type = task_type.task_type_name();
        debug!(task_type, "creating sync task");
        let request = CreateTaskRequest {
            client_key: &self.client_key,
            app_id: self.config.app_id,
            task: TaskPayload {
                task_type,
                params: task,
            },
        };
        let result: TaskResult = self
            .post(
                &endpoint(&self.config.sync_base_url, "/createSyncTask"),
                &request,
                self.config.sync_timeout,
            )
            .await?;
        if result.status == TaskStatus::Processing {
            return Err(Error::UnexpectedResponse(
                "createSyncTask returned status `processing`".to_owned(),
            ));
        }
        info!(task_type, "sync task completed");
        Ok(result)
    }

    /// Queries the current account balance.
    pub async fn balance(&self) -> Result<f64> {
        debug!("querying account balance");
        let request = QueryBalanceRequest {
            client_key: &self.client_key,
        };
        let response: BalanceResponse = self
            .post(
                &endpoint(&self.config.async_base_url, "/getBalance"),
                &request,
                self.config.timeout,
            )
            .await?;
        Ok(response.balance)
    }

    async fn post<B, T>(&self, url: &str, body: &B, timeout: Duration) -> Result<T>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        log_request(url, body);
        let started = Instant::now();
        let response = self
            .http
            .post(url)
            .timeout(timeout)
            .header(CLIENT_KEY_HEADER, self.client_key_header.clone())
            .json(body)
            .send()
            .await?;
        let status = response.status().as_u16();
        let bytes = response.bytes().await?;
        log_response(url, status, started.elapsed(), &bytes);
        parse_api_response(status, &bytes)
    }
}

fn build_http_client(config: &ClientConfig) -> Result<reqwest::Client> {
    let mut builder = reqwest::Client::builder()
        // No timeout here: the two endpoints have different timing characteristics,
        // so each request applies its own budget over this one shared pool.
        .user_agent(config.user_agent.clone());
    if let Some(proxy) = &config.proxy {
        let proxy = reqwest::Proxy::all(proxy)
            .map_err(|error| Error::config(format!("invalid SDK proxy URL: {error}")))?;
        builder = builder.proxy(proxy);
    }
    Ok(builder.build()?)
}

mod solvers;

#[cfg(test)]
mod tests;
