//! Integration tests for response and solution modeling.

use ezcapsolver::{
    AkamaiSbsdSolution, AkamaiWebSolution, Cloudflare5sSolution, CloudflareTurnstileSolution,
    DataDomeSolution, Error, EzError, FunCaptchaClassificationSolution, FunCaptchaSolution,
    HcaptchaClassificationSolution, HcaptchaSolution, IncapsulaSolution, PerimeterXSolution,
    ReClassificationSolution, RecaptchaSolution, Solution, TaskResult, TaskStatus,
    TlsForwardSolution,
};
use serde_json::{Value, json};

#[test]
fn solution_distinguishes_missing_and_explicit_null() -> Result<(), serde_json::Error> {
    let missing: TaskResult =
        serde_json::from_value(json!({"errorId": 0, "status": "processing"}))?;
    assert!(missing.solution.is_missing());
    assert_eq!(
        serde_json::to_value(missing)?,
        json!({"errorId": 0, "status": "processing"})
    );

    let explicit_null: TaskResult =
        serde_json::from_value(json!({"errorId": 0, "status": "ready", "solution": null}))?;
    assert_eq!(explicit_null.solution.as_value(), Some(&Value::Null));
    assert_eq!(
        serde_json::to_value(explicit_null)?,
        json!({"errorId": 0, "status": "ready", "solution": null})
    );
    Ok(())
}

#[test]
fn raw_solution_can_be_decoded_without_losing_extra_fields()
-> Result<(), Box<dyn std::error::Error>> {
    let raw_solution = json!({
        "gRecaptchaResponse": "token-value",
        "sec_ch_ua": "\"Chromium\";v=\"140\"",
        "user_agent": "Mozilla/5.0",
        "workerRegion": "us-east"
    });
    let result: TaskResult = serde_json::from_value(json!({
        "errorId": 0,
        "requestId": "request-1",
        "status": "ready",
        "solution": raw_solution
    }))?;

    let solution: RecaptchaSolution = result.deserialize_solution()?;
    assert_eq!(solution.token, "token-value");
    assert_eq!(solution.user_agent, "Mozilla/5.0");
    // Typed fields are consumed by the struct, so only unmodeled keys remain.
    assert_eq!(solution.extra.get("workerRegion"), Some(&json!("us-east")));
    assert_eq!(solution.extra.len(), 1);
    assert_eq!(result.solution.as_value(), Some(&raw_solution));
    Ok(())
}

#[test]
fn a_recaptcha_solution_decodes_from_the_token_alone() -> Result<(), Box<dyn std::error::Error>> {
    // Token-only is a shape the service has been observed returning. Treating
    // sec_ch_ua and user_agent as required would fail decoding for a task that
    // was already solved and already billed.
    let result: TaskResult = serde_json::from_value(json!({
        "errorId": 0,
        "status": "ready",
        "solution": {"gRecaptchaResponse": "token-value"}
    }))?;

    let solution: RecaptchaSolution = result.deserialize_solution()?;
    assert_eq!(solution.token, "token-value");
    assert_eq!(solution.sec_ch_ua, "");
    assert_eq!(solution.user_agent, "");
    assert!(solution.extra.is_empty());
    Ok(())
}

#[test]
fn decoding_a_solution_borrows_it_and_keeps_numbers_intact()
-> Result<(), Box<dyn std::error::Error>> {
    #[derive(serde::Deserialize)]
    struct Amounts {
        exact: serde_json::Number,
        nested: Vec<serde_json::Number>,
    }

    let result: TaskResult = serde_json::from_str(
        r#"{
            "errorId": 0,
            "status": "ready",
            "solution": {"exact": 1234.5678, "nested": [0.1, 42]}
        }"#,
    )?;

    let amounts: Amounts = result.deserialize_solution()?;
    assert_eq!(amounts.exact.to_string(), "1234.5678");
    assert_eq!(amounts.nested[0].to_string(), "0.1");
    assert_eq!(amounts.nested[1].to_string(), "42");

    // Decoding borrows the raw value, so it neither consumes the solution nor
    // degrades on a second read.
    let again: Amounts = result.deserialize_solution()?;
    assert_eq!(again.exact.to_string(), amounts.exact.to_string());
    assert!(result.solution.as_value().is_some());
    Ok(())
}

#[test]
fn a_turnstile_solution_keeps_its_replay_headers() -> Result<(), Box<dyn std::error::Error>> {
    let result: TaskResult = serde_json::from_value(json!({
        "errorId": 0,
        "status": "ready",
        "solution": {"header": {}, "token": "1.QgKUTW9RBSQ.KIWFJgK4fMu9Zm7pGx2ONA.f2bafd24"}
    }))?;

    let solution: CloudflareTurnstileSolution = result.deserialize_solution()?;
    assert!(solution.token.starts_with("1.QgKUTW9RBSQ"));
    // An empty header map is the common case and must not be treated as absent.
    assert!(solution.header.is_empty());
    assert!(solution.extra.is_empty());
    Ok(())
}

#[test]
fn a_cloudflare_5s_solution_decodes_every_documented_field()
-> Result<(), Box<dyn std::error::Error>> {
    let result: TaskResult = serde_json::from_value(json!({
        "errorId": 0,
        "status": "ready",
        "solution": {
            "header": {"accept-language": "zh-CN,zh;q=0.9", "priority": "u=0, i"},
            "cookies": {"_cfuvid": "sKZAZRcbB1dPlOJmIfiBBtymnGJ.fX3nwYuTp92usUg"},
            "tlsVersion": "chrome149",
            "body": "",
            "sToken": ""
        }
    }))?;

    let solution: Cloudflare5sSolution = result.deserialize_solution()?;
    assert_eq!(solution.tls_version, "chrome149");
    assert_eq!(solution.header["priority"], "u=0, i");
    assert!(solution.cookies["_cfuvid"].starts_with("sKZAZRcbB1"));
    assert!(solution.body.is_empty());
    assert!(solution.s_token.is_empty());
    assert!(solution.extra.is_empty());
    Ok(())
}

#[test]
fn missing_solution_returns_a_specific_error() {
    let error = Solution::Missing.deserialize::<Value>().err();
    assert!(matches!(
        error,
        Some(Error::UnexpectedResponse(message)) if message.contains("does not contain a solution")
    ));
}

#[test]
fn every_task_status_round_trips() -> Result<(), serde_json::Error> {
    for (wire, status) in [
        ("processing", TaskStatus::Processing),
        ("ready", TaskStatus::Ready),
        ("error", TaskStatus::Error),
    ] {
        assert_eq!(serde_json::from_value::<TaskStatus>(json!(wire))?, status);
        assert_eq!(serde_json::to_value(&status)?, json!(wire));
    }
    Ok(())
}

#[test]
fn an_unrecognised_status_is_rejected() {
    // The service reports only the three statuses above, so anything else is a
    // contract violation and must surface instead of being silently accepted.
    let error = serde_json::from_value::<TaskStatus>(json!("queued")).unwrap_err();

    let message = error.to_string();
    assert!(message.contains("queued"), "{message}");
    assert!(message.contains("processing"), "{message}");
}

#[test]
fn api_error_display_contains_service_code_and_description() {
    let error = EzError {
        task_id: Some("task-1".to_owned()),
        error_code: Some("ERROR_TASK_NOT_FOUND".to_owned()),
        error_description: Some("Task does not exist".to_owned()),
        request_id: Some("request-1".to_owned()),
        errors: Default::default(),
        http_status: 404,
    };

    let display = error.to_string();
    assert!(display.contains("ERROR_TASK_NOT_FOUND"));
    assert!(display.contains("Task does not exist"));
    assert!(display.contains("HTTP 404"));
}

/// Workers add response fields more often than the SDK ships, so an unmodelled key
/// has to land in the pass-through map rather than be dropped.
#[test]
fn every_solution_model_keeps_unmodelled_worker_fields() -> Result<(), serde_json::Error> {
    let akamai_web: AkamaiWebSolution = serde_json::from_value(json!({
        "payload": "sensor",
        "encodedata": "state",
        "futureField": 1
    }))?;
    assert_eq!(akamai_web.extra["futureField"], json!(1));

    let akamai_sbsd: AkamaiSbsdSolution =
        serde_json::from_value(json!({"payload": "p", "futureField": "x"}))?;
    assert_eq!(akamai_sbsd.extra["futureField"], json!("x"));

    let incapsula: IncapsulaSolution =
        serde_json::from_value(json!({"status": 200, "data": "{}", "futureField": true}))?;
    assert_eq!(incapsula.extra["futureField"], json!(true));

    let data_dome: DataDomeSolution =
        serde_json::from_value(json!({"url": "https://example.com", "futureField": []}))?;
    assert_eq!(data_dome.extra["futureField"], json!([]));

    let classification: ReClassificationSolution =
        serde_json::from_value(json!({"type": "single", "hasObject": true, "futureField": 2}))?;
    assert!(classification.is_single());
    assert!(classification.has_object);
    assert_eq!(classification.extra["futureField"], json!(2));
    Ok(())
}

/// The last round of the multi-round Akamai flow carries no `encodedata`, and one
/// absent field must not make the whole result undecodable.
#[test]
fn a_missing_optional_solution_field_still_decodes() -> Result<(), serde_json::Error> {
    let solution: AkamaiWebSolution = serde_json::from_value(json!({"payload": "sensor"}))?;

    assert_eq!(solution.payload, "sensor");
    assert!(solution.encodedata.is_empty());
    Ok(())
}

/// New worker result types retain their fields without requiring an SDK update.
#[test]
fn an_unknown_classification_discriminant_is_preserved() -> Result<(), serde_json::Error> {
    let raw = json!({"type": "rotate", "angle": 137});
    let solution: ReClassificationSolution = serde_json::from_value(raw)?;

    assert_eq!(solution.r#type, "rotate");
    assert!(!solution.is_multi());
    assert!(!solution.is_single());
    assert_eq!(solution.extra.len(), 1);
    assert_eq!(solution.extra["angle"], json!(137));
    assert_eq!(
        serde_json::to_value(solution)?,
        json!({"type": "rotate", "hasObject": false, "objects": [], "angle": 137})
    );
    Ok(())
}

#[test]
fn classification_results_expose_fields_directly() -> Result<(), serde_json::Error> {
    let multi: ReClassificationSolution =
        serde_json::from_value(json!({"type": "multi", "objects": [0, 3, 7]}))?;
    assert_eq!(multi.r#type, "multi");
    assert!(multi.is_multi());
    assert!(!multi.is_single());
    assert_eq!(multi.objects, vec![0, 3, 7]);
    assert!(!multi.has_object);
    assert!(multi.extra.is_empty());

    for has_object in [true, false] {
        let single: ReClassificationSolution =
            serde_json::from_value(json!({"type": "single", "hasObject": has_object}))?;
        assert_eq!(single.r#type, "single");
        assert!(single.is_single());
        assert!(!single.is_multi());
        assert_eq!(single.has_object, has_object);
        assert!(single.objects.is_empty());
        assert!(single.extra.is_empty());
    }
    Ok(())
}

#[test]
fn classification_defaults_do_not_infer_the_result_kind() -> Result<(), serde_json::Error> {
    let empty: ReClassificationSolution = serde_json::from_value(json!({}))?;
    assert_eq!(empty, ReClassificationSolution::default());
    assert!(!empty.is_multi());
    assert!(!empty.is_single());

    let without_type: ReClassificationSolution =
        serde_json::from_value(json!({"objects": [2], "hasObject": true}))?;
    assert_eq!(
        without_type,
        ReClassificationSolution {
            has_object: true,
            objects: vec![2],
            ..Default::default()
        }
    );
    assert!(!without_type.is_multi());
    assert!(!without_type.is_single());
    Ok(())
}

#[test]
fn classification_fields_round_trip_regardless_of_type() -> Result<(), serde_json::Error> {
    for kind in ["multi", "single", "future"] {
        let raw = json!({
            "type": kind,
            "hasObject": true,
            "objects": [1, 4],
            "confidence": {"score": 0.9}
        });
        let solution: ReClassificationSolution = serde_json::from_value(raw.clone())?;
        assert_eq!(solution.r#type, kind);
        assert!(solution.has_object);
        assert_eq!(solution.objects, vec![1, 4]);
        assert_eq!(solution.extra.len(), 1);
        assert_eq!(solution.extra["confidence"], json!({"score": 0.9}));
        assert_eq!(serde_json::to_value(solution)?, raw);
    }
    Ok(())
}

#[test]
fn classification_decode_errors_preserve_the_raw_value() -> Result<(), Box<dyn std::error::Error>> {
    for value in [
        json!([0, 3]),
        Value::Null,
        json!({"type": "multi", "objects": ["invalid"]}),
        json!({"type": "single", "hasObject": "invalid"}),
    ] {
        match Solution::Value(value.clone()).deserialize::<ReClassificationSolution>() {
            Err(Error::SolutionDecode { raw, .. }) => assert_eq!(*raw, value),
            other => return Err(format!("expected a solution decode error, got {other:?}").into()),
        }
    }
    Ok(())
}

/// The convenience methods consume the TaskResult, so on a decode failure the
/// error is the only place the raw value survives.
#[test]
fn a_decode_failure_carries_the_raw_solution() {
    let solution = Solution::Value(json!({"unexpected": "shape"}));

    let Err(Error::SolutionDecode { raw, .. }) = solution.deserialize::<RecaptchaSolution>() else {
        panic!("expected the decode to fail");
    };
    assert_eq!(*raw, json!({"unexpected": "shape"}));

    let rendered = Error::SolutionDecode {
        source: serde_json::from_str::<RecaptchaSolution>("{}").unwrap_err(),
        raw: Box::new(json!({"unexpected": "shape"})),
    }
    .to_string();
    assert!(
        rendered.contains(r#"{"unexpected":"shape"}"#),
        "the raw value is what diagnosing a decode failure needs, so it has to \
         appear in the message: {rendered}"
    );
}

/// PerimeterX returns its clearance cookies as top-level solution fields, not
/// nested under a `cookies` object.
#[test]
fn a_perimeter_x_solution_reads_top_level_cookies() -> Result<(), Box<dyn std::error::Error>> {
    let result: TaskResult = serde_json::from_value(json!({
        "errorId": 0,
        "errorCode": null,
        "errorDescription": null,
        "solution": {"_px3": "px3-value", "_pxvid": "vid-value", "_pxde": "de-value"},
        "status": "ready"
    }))?;
    let solution: PerimeterXSolution = result.deserialize_solution()?;

    assert_eq!(solution.px3, "px3-value");
    assert_eq!(solution.pxvid, "vid-value");
    assert_eq!(solution.pxde, "de-value");

    // Only `_px3` is the core output; the rest going missing must not fail the
    // decode.
    let minimal: PerimeterXSolution = serde_json::from_value(json!({"_px3": "only"}))?;
    assert_eq!(minimal.px3, "only");
    assert!(minimal.pxvid.is_empty());
    Ok(())
}

/// Two DataDome types and two steps now decode into one structure.
///
/// Step 1 used to return a bare string and now wraps it in a `{"url": ...}` object.
#[test]
fn both_data_dome_steps_decode_into_one_structure() -> Result<(), Box<dyn std::error::Error>> {
    let step_one: DataDomeSolution = serde_json::from_value(
        json!({"url": "https://geo.captcha-delivery.com/captcha/?initialCid=x"}),
    )?;
    assert_eq!(
        step_one.url.as_deref(),
        Some("https://geo.captcha-delivery.com/captcha/?initialCid=x")
    );
    assert!(step_one.kind.is_none());

    let step_two: DataDomeSolution = serde_json::from_value(json!({
        "kind": "slider",
        "url": "https://geo.captcha-delivery.com/captcha/check",
        "body": "payload",
        "futureField": 1
    }))?;
    assert_eq!(step_two.kind.as_deref(), Some("slider"));
    assert_eq!(step_two.body.as_deref(), Some("payload"));
    assert_eq!(step_two.extra["futureField"], json!(1));
    Ok(())
}

/// A pass-through value never rewrites a field the result model decoded.
///
/// `Solved` is documented as serializable for logging and storage, so a model
/// that let `extra` shadow a declared field would round-trip a stored record
/// into a different one — swapping a real token for whatever sits in the map.
/// Every model is covered because each reaches this behaviour through its own
/// attributes, so missing one would not be a compile error.
#[test]
fn no_solution_lets_a_pass_through_value_shadow_a_declared_field() -> Result<(), serde_json::Error>
{
    macro_rules! assert_declared_fields_win {
        ($($ty:ty => $sample:expr),+ $(,)?) => {$({
            let mut solution: $ty = serde_json::from_value($sample)?;
            let declared = serde_json::to_value(&solution)?;
            let keys: Vec<String> = declared
                .as_object()
                .expect("a result model serializes as an object")
                .keys()
                .cloned()
                .collect();

            // Put every key the model produces into extra, then check that none of
            // them was displaced on the way back out.
            for key in &keys {
                solution.extra.insert(key.clone(), json!("SHADOWED"));
            }
            solution.extra.insert("brandNewField".to_owned(), json!("kept"));

            let rendered = serde_json::to_string(&solution)?;
            let merged: Value = serde_json::from_str(&rendered)?;
            for key in &keys {
                assert_eq!(
                    merged[key], declared[key],
                    "{}: extra overwrote the decoded field `{key}`",
                    stringify!($ty)
                );
                assert_eq!(
                    rendered.matches(&format!("\"{key}\"")).count(), 1,
                    "{}: `{key}` was emitted more than once",
                    stringify!($ty)
                );
            }
            assert_eq!(
                merged["brandNewField"], json!("kept"),
                "{}: a non-colliding pass-through key was dropped",
                stringify!($ty)
            );
        })+};
    }

    assert_declared_fields_win!(
        RecaptchaSolution => json!({"gRecaptchaResponse": "t", "sec_ch_ua": "a", "user_agent": "b"}),
        FunCaptchaSolution => json!({"token": "t"}),
        FunCaptchaClassificationSolution => json!({}),
        HcaptchaClassificationSolution => json!({}),
        CloudflareTurnstileSolution => json!({"token": "t", "header": {}}),
        Cloudflare5sSolution => json!({
            "header": {}, "cookies": {}, "tlsVersion": "v", "body": "b", "sToken": "s"
        }),
        HcaptchaSolution => json!({"generated_pass_UUID": "u"}),
        PerimeterXSolution => json!({"_px3": "p"}),
        AkamaiWebSolution => json!({"payload": "p"}),
        AkamaiSbsdSolution => json!({"payload": "p"}),
        TlsForwardSolution => json!({
            "status": 200, "code": 200, "headers": {}, "cookies": {}, "body": "b"
        }),
        ReClassificationSolution => json!({}),
        DataDomeSolution => json!({}),
        IncapsulaSolution => json!({"data": "{}"}),
    );
    Ok(())
}

/// The codes that count towards a ban come from @ApiDefenses on
/// AsyncTaskController. Retrying them only reaches the ban sooner.
#[test]
fn credential_errors_are_told_apart_from_retryable_ones() {
    let of = |code: Option<&str>| EzError {
        request_id: None,
        task_id: None,
        error_code: code.map(str::to_owned),
        error_description: None,
        errors: Default::default(),
        http_status: 500,
    };

    for code in [
        "ERROR_KEY_DOES_NOT_EXIST",
        "ERROR_KEY_NOT_AVAILABLE",
        "ERROR_ZERO_BALANCE",
    ] {
        let error = of(Some(code));
        assert!(
            error.is_authentication_error(),
            "{code} triggers the ban counter"
        );
        assert!(error.is_terminal(), "{code} never clears on its own");
    }

    // A synchronous worker fault and a rate limit both clear on their own, so
    // neither may be marked terminal.
    for code in [
        "ERROR_SERVICE_UNAVAILABLE",
        "ERROR_SERVICE_TIMEOUT",
        "ERROR_REQUEST_LIMIT",
        "ERROR_REQUEST_BANNED",
        "ERROR_INTERNAL_SERVER_ERROR",
    ] {
        let error = of(Some(code));
        assert!(
            !error.is_authentication_error(),
            "{code} is not a credential fault"
        );
        assert!(!error.is_terminal(), "{code} can clear on its own");
    }

    assert!(!of(Some("ERROR_WEBSITE_NOT_ALLOWED")).is_authentication_error());
    assert!(of(Some("ERROR_WEBSITE_NOT_ALLOWED")).is_terminal());
    // An unrecognised code is never terminal: it must not talk a caller out of a
    // retry that would have succeeded.
    assert!(!of(Some("ERROR_MINTED_NEXT_RELEASE")).is_terminal());
    assert!(!of(None).is_terminal());

    // The polling loop retries exactly the rate-limited set, so it has to stay
    // narrower than "everything that is not terminal". A code leaking in would
    // make the loop poll on past a task that has genuinely failed.
    for code in ["ERROR_REQUEST_LIMIT", "ERROR_REQUEST_BANNED"] {
        assert!(
            of(Some(code)).is_rate_limited(),
            "{code} is a throttled query"
        );
    }
    for code in [
        "ERROR_SERVICE_UNAVAILABLE",
        "ERROR_SERVICE_TIMEOUT",
        "ERROR_INTERNAL_SERVER_ERROR",
        "ERROR_TASK_NOT_EXIST",
        "ERROR_ZERO_BALANCE",
        "ERROR_MINTED_NEXT_RELEASE",
    ] {
        assert!(
            !of(Some(code)).is_rate_limited(),
            "{code} is not a throttled query"
        );
    }
    assert!(!of(None).is_rate_limited());
}

/// Whether an error carries a billed task is one question a caller asks once. Which
/// variant happens to hold the id is not their concern.
#[test]
fn the_identifier_of_a_billed_task_is_reachable_from_one_place() {
    use std::{collections::BTreeMap, time::Duration};

    let interrupted = Error::WaitInterrupted {
        task_id: "task-1".to_owned(),
        request_id: None,
        source: Box::new(Error::UnexpectedResponse("broken".to_owned())),
    };
    assert_eq!(interrupted.task_id(), Some("task-1"));

    let exhausted = Error::PollingExhausted {
        task_id: "task-2".to_owned(),
        attempts: 3,
        interval: Duration::from_secs(1),
    };
    assert_eq!(exhausted.task_id(), Some("task-2"));

    let api = Error::from(ezcapsolver::EzError {
        request_id: None,
        task_id: Some("task-3".to_owned()),
        error_code: Some("ERROR_TASK_NOT_EXIST".to_owned()),
        error_description: None,
        errors: BTreeMap::new(),
        http_status: 500,
    });
    assert_eq!(api.task_id(), Some("task-3"));

    // Failing before creation means no task and no charge, so there is nothing to
    // recover.
    assert_eq!(
        Error::UnexpectedResponse("no task was ever created".to_owned()).task_id(),
        None
    );
}
