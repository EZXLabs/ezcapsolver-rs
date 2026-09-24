//! Integration tests for task type and request parameter serialization.

use ezcapsolver::{
    AkamaiSbsdTask, AkamaiWebTask, Cloudflare5sTask, CloudflareTurnstileTask, DataDomeJsType,
    DataDomeStep, DataDomeTagsTask, DataDomeTask, FunCaptchaClassificationTask, FunCaptchaTask,
    HcaptchaClassificationTask, HcaptchaTask, IncapsulaTask, PerimeterXTask,
    RecaptchaV2ClassificationTask, RecaptchaV2Task, RecaptchaV3Task, TaskMode, TaskType,
    TlsForwardTask, TlsHttpMethod,
};
use serde_json::{Value, json};

#[test]
fn known_task_types_have_canonical_names_and_modes() {
    assert_eq!(TaskType::KNOWN.len(), 23);
    assert_eq!(
        TaskType::KNOWN
            .iter()
            .filter(|task_type| task_type.mode() == Some(TaskMode::Async))
            .count(),
        14
    );
    assert_eq!(
        TaskType::KNOWN
            .iter()
            .filter(|task_type| task_type.mode() == Some(TaskMode::Sync))
            .count(),
        9
    );

    let expected = [
        "ReCaptchaV2TaskProxyless",
        "ReCaptchaV2TaskProxylessS9",
        "ReCaptchaV2STaskProxyless",
        "ReCaptchaV2EnterpriseTaskProxyless",
        "ReCaptchaV2SEnterpriseTaskProxyless",
        "ReCaptchaV2Classification",
        "ReCaptchaV3TaskProxyless",
        "ReCaptchaV3TaskProxylessS9",
        "ReCaptchaV3EnterpriseTaskProxyless",
        "ReCaptchaV3EnterpriseTaskProxylessS9",
        "FuncaptchaTaskProxyless",
        "FunCaptchaClassification",
        "PerimeterX",
        "HCaptcha",
        "HCaptchaClassification",
        "AkamaiWEBTaskProxyless",
        "AkamaiSBSDTaskProxyless",
        "TlsTask",
        "CloudFlare5STask",
        "CloudFlareTurnstileTask",
        "DataDomeTaskProxyless",
        "DataDomeTagsTaskProxyless",
        "IncapsulaTaskProxyless",
    ];
    let actual: Vec<_> = TaskType::KNOWN.iter().map(TaskType::as_str).collect();
    assert_eq!(actual, expected);
}

#[test]
fn all_task_parameter_structs_implement_default() {
    fn assert_default<T: Default>() {}

    assert_default::<AkamaiWebTask>();
    assert_default::<AkamaiSbsdTask>();
    assert_default::<Cloudflare5sTask>();
    assert_default::<CloudflareTurnstileTask>();
    assert_default::<DataDomeTask>();
    assert_default::<DataDomeTagsTask>();
    assert_default::<FunCaptchaTask>();
    assert_default::<FunCaptchaClassificationTask>();
    assert_default::<HcaptchaTask>();
    assert_default::<HcaptchaClassificationTask>();
    assert_default::<IncapsulaTask>();
    assert_default::<PerimeterXTask>();
    assert_default::<RecaptchaV2Task>();
    assert_default::<RecaptchaV3Task>();
    assert_default::<RecaptchaV2ClassificationTask>();
    assert_default::<TlsForwardTask>();
}

#[test]
fn task_parameters_can_be_built_with_struct_literals() {
    let recaptcha_v2 = RecaptchaV2Task {
        website_url: "https://example.com".to_owned(),
        website_key: "site-key".to_owned(),
        is_invisible: true,
        s: Some("challenge-s".to_owned()),
        ..Default::default()
    };
    assert_eq!(recaptcha_v2.website_key, "site-key");
    assert_eq!(recaptcha_v2.s.as_deref(), Some("challenge-s"));

    let recaptcha_v3 = RecaptchaV3Task {
        website_url: "https://example.com".to_owned(),
        website_key: "site-key".to_owned(),
        page_action: Some("login".to_owned()),
        ..Default::default()
    };
    assert!(recaptcha_v3.is_invisible);
    assert_eq!(recaptcha_v3.page_action.as_deref(), Some("login"));
}

#[test]
fn recaptcha_serializes_parameters_and_pass_through_values() -> Result<(), serde_json::Error> {
    let mut task = RecaptchaV2Task {
        website_url: "https://example.com/challenge".to_owned(),
        website_key: "site-key".to_owned(),
        is_invisible: true,
        ..Default::default()
    };
    task.extra.insert("futureFlag".to_owned(), json!(true));

    assert_eq!(
        serde_json::to_value(task)?,
        json!({
            "websiteURL": "https://example.com/challenge",
            "websiteKey": "site-key",
            "isInvisible": true,
            "futureFlag": true
        })
    );
    Ok(())
}

/// A pass-through key never overwrites a field the model already produced.
///
/// The method a caller reached for is what chose the declared value, so letting
/// `extra` win would send a parameter the call site never mentions — and the
/// task is billed either way. A key the declared fields did *not* emit, such as
/// an unset optional, is still free for `extra` to supply.
#[test]
fn pass_through_values_never_shadow_declared_fields() -> Result<(), serde_json::Error> {
    let mut task = RecaptchaV2Task {
        website_url: "https://example.com/challenge".to_owned(),
        website_key: "declared-key".to_owned(),
        is_invisible: false,
        // `s` stays unset: it is optional and is skipped when serialising.
        ..Default::default()
    };
    for (key, value) in [
        ("websiteURL", json!("https://evil.example")),
        ("websiteKey", json!("extra-key")),
        ("isInvisible", json!(true)),
        ("s", json!("from-extra")),
        ("brandNew", json!(42)),
    ] {
        task.extra.insert(key.to_owned(), value);
    }

    assert_eq!(
        serde_json::to_value(task)?,
        json!({
            "websiteURL": "https://example.com/challenge",
            "websiteKey": "declared-key",
            "isInvisible": false,
            "s": "from-extra",
            "brandNew": 42
        })
    );
    Ok(())
}

/// Serializing a model twice over produces each key once.
///
/// A flattened map would emit the colliding key a second time instead of
/// dropping it, which parses back to whichever copy the reader happens to keep.
#[test]
fn colliding_pass_through_keys_are_not_emitted_twice() -> Result<(), serde_json::Error> {
    let mut task = RecaptchaV2Task {
        website_url: "https://example.com".to_owned(),
        website_key: "declared-key".to_owned(),
        ..Default::default()
    };
    task.extra
        .insert("websiteKey".to_owned(), json!("extra-key"));

    let rendered = serde_json::to_string(&task)?;
    assert_eq!(rendered.matches("\"websiteKey\"").count(), 1);
    assert!(rendered.contains("\"websiteKey\":\"declared-key\""));
    Ok(())
}

/// Every model resists shadowing, not just the ones with a case written by hand.
///
/// A model reaches this behaviour through two edits that are easy to apply to
/// fifteen of sixteen types — the `remote` attribute and the macro entry — so
/// the guard is driven off `Default` and covers all of them at once.
#[test]
fn no_model_lets_a_pass_through_value_shadow_a_declared_field() -> Result<(), serde_json::Error> {
    macro_rules! assert_declared_fields_win {
        ($($ty:ty),+ $(,)?) => {$({
            let mut task = <$ty>::default();
            let declared = serde_json::to_value(&task)?;
            let keys: Vec<String> = declared
                .as_object()
                .expect("a task model serializes as an object")
                .keys()
                .cloned()
                .collect();

            // Put every key the model produces into extra, then check that none of
            // them was displaced.
            for key in &keys {
                task.extra.insert(key.clone(), json!("SHADOWED"));
            }
            task.extra.insert("brandNewParam".to_owned(), json!("kept"));

            let merged = serde_json::to_value(&task)?;
            for key in &keys {
                assert_eq!(
                    merged[key], declared[key],
                    "{}: extra overwrote the declared field `{key}`",
                    stringify!($ty)
                );
            }
            assert_eq!(
                merged["brandNewParam"], json!("kept"),
                "{}: a non-colliding pass-through key was dropped",
                stringify!($ty)
            );
        })+};
    }

    assert_declared_fields_win!(
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
    Ok(())
}

#[test]
fn mixed_naming_strategies_match_the_service_contract() -> Result<(), serde_json::Error> {
    let tls = TlsForwardTask {
        tls_type: "chrome_131".to_owned(),
        proxy: "http://user:pass@127.0.0.1:8080".to_owned(),
        method: TlsHttpMethod::Post,
        url: "https://example.com/api".to_owned(),
        headers: None,
        headers_order: Some("host,user-agent".to_owned()),
        cookies: None,
        body: None,
        body_raw: false,
        extra: Default::default(),
    };
    assert_eq!(
        serde_json::to_value(tls)?,
        json!({
            "tls_type": "chrome_131",
            "proxy": "http://user:pass@127.0.0.1:8080",
            "method": "POST",
            "url": "https://example.com/api",
            "headers_order": "host,user-agent",
            // Stays snake_case like its neighbours: this model has no
            // rename_all, unlike every other task.
            "body_raw": false
        })
    );

    let data_dome = DataDomeTask {
        html_b64: "PGh0bWw+".to_owned(),
        step: DataDomeStep::Two,
        image: String::new(),
        referer: None,
        parent_url: Some("https://example.com/parent".to_owned()),
        equipment: None,
        extra: Default::default(),
    };
    assert_eq!(
        serde_json::to_value(data_dome)?,
        json!({
            "html_b64": "PGh0bWw+",
            "step": "2",
            "image": "",
            "parent_url": "https://example.com/parent"
        })
    );

    let incapsula = IncapsulaTask {
        script: "script source".to_owned(),
        script_url: "https://example.com/reese84.js".to_owned(),
        page_url: "https://example.com".to_owned(),
        accept_language: Some("en-US,en;q=0.9".to_owned()),
        ua: "Mozilla/5.0".to_owned(),
        pow: Some("pow-token".to_owned()),
        ..Default::default()
    };
    assert_eq!(
        serde_json::to_value(incapsula)?,
        json!({
            "script": "script source",
            "scriptUrl": "https://example.com/reese84.js",
            "pageUrl": "https://example.com",
            "acceptLanguage": "en-US,en;q=0.9",
            "ua": "Mozilla/5.0",
            "pow": "pow-token"
        })
    );
    Ok(())
}

#[test]
fn enum_backed_task_fields_use_service_values() -> Result<(), serde_json::Error> {
    let tags = DataDomeTagsTask {
        ddk: "dd-key".to_owned(),
        jstype: DataDomeJsType::Ch,
        cid: String::new(),
        bpc: 1,
        referer: "https://example.com/path".to_owned(),
        ua: "Mozilla/5.0".to_owned(),
        fields: Default::default(),
        extra: Default::default(),
    };
    let value = serde_json::to_value(tags)?;
    assert_eq!(value["jstype"], Value::String("ch".to_owned()));
    assert_eq!(value["fields"], json!({}));
    assert_eq!(
        value,
        json!({
            "ddk": "dd-key",
            "jstype": "ch",
            "cid": "",
            "bpc": 1,
            "referer": "https://example.com/path",
            "ua": "Mozilla/5.0",
            "fields": {}
        })
    );
    Ok(())
}

#[test]
fn every_remaining_request_model_matches_the_wire_contract() -> Result<(), serde_json::Error> {
    let recaptcha_v3 = RecaptchaV3Task {
        website_url: "https://example.com".to_owned(),
        website_key: "site-key".to_owned(),
        page_action: Some("login".to_owned()),
        website_title: Some("Example".to_owned()),
        check_field: Some("field".to_owned()),
        ..Default::default()
    };
    assert_eq!(
        serde_json::to_value(recaptcha_v3)?,
        json!({
            "websiteURL": "https://example.com",
            "websiteKey": "site-key",
            "isInvisible": true,
            "pageAction": "login",
            "websiteTitle": "Example",
            "checkField": "field"
        })
    );

    let classification = RecaptchaV2ClassificationTask {
        image: "aW1n".to_owned(),
        question: "/m/0k4j".to_owned(),
        ..Default::default()
    };
    assert_eq!(
        serde_json::to_value(classification)?,
        json!({"image": "aW1n", "question": "/m/0k4j", "size": 4})
    );

    let funcaptcha = FunCaptchaTask {
        website_url: "https://example.com".to_owned(),
        website_key: "site-key".to_owned(),
        data: Some(r#"{"blob":"abc"}"#.to_owned()),
        api_js_subdomain: Some("client-api".to_owned()),
        proxy: Some("http://127.0.0.1:8080:user:pass".to_owned()),
        cn: true,
        ..Default::default()
    };
    assert_eq!(
        serde_json::to_value(funcaptcha)?,
        json!({
            "websiteURL": "https://example.com",
            "websiteKey": "site-key",
            "data": r#"{"blob":"abc"}"#,
            "funcaptchaApiJSSubdomain": "client-api",
            "proxy": "http://127.0.0.1:8080:user:pass",
            "cn": true
        })
    );

    let funcaptcha_classification = FunCaptchaClassificationTask {
        image: "aW1n".to_owned(),
        question: "rotate".to_owned(),
        ..Default::default()
    };
    assert_eq!(
        serde_json::to_value(funcaptcha_classification)?,
        json!({"image": "aW1n", "question": "rotate"})
    );

    let hcaptcha = HcaptchaTask {
        website_url: "https://example.com".to_owned(),
        website_key: "site-key".to_owned(),
        lang: "en-US".to_owned(),
        invisible: true,
        rq_data: Some("rq".to_owned()),
        ..Default::default()
    };
    assert_eq!(
        serde_json::to_value(hcaptcha)?,
        json!({
            "websiteURL": "https://example.com",
            "websiteKey": "site-key",
            "lang": "en-US",
            // No `is` prefix, unlike the reCAPTCHA types, and all lowercase
            // `rqdata`, unlike the Cloudflare types' `rqData`.
            "invisible": true,
            "rqdata": "rq"
        })
    );

    let hcaptcha_classification = HcaptchaClassificationTask {
        images: Some(vec!["aW1n".to_owned()]),
        question: Some("bicycle".to_owned()),
        ..Default::default()
    };
    assert_eq!(
        serde_json::to_value(hcaptcha_classification)?,
        json!({"images": ["aW1n"], "question": "bicycle"})
    );

    let perimeter_x = PerimeterXTask {
        website_key: "PX-app-id".to_owned(),
        invisible: true,
        ..Default::default()
    };
    assert_eq!(
        serde_json::to_value(perimeter_x)?,
        json!({"websiteKey": "PX-app-id", "invisible": true})
    );

    let akamai_web = AkamaiWebTask {
        page_url: "https://example.com".to_owned(),
        v3_url: "https://example.com/v3.js".to_owned(),
        ua: "Mozilla/5.0".to_owned(),
        lang: "en".to_owned(),
        index: 1,
        abck: "abck-value".to_owned(),
        bmsz: "bmsz-value".to_owned(),
        script_base64: "c2NyaXB0".to_owned(),
        encode_data: "state".to_owned(),
        ..Default::default()
    };
    assert_eq!(
        serde_json::to_value(akamai_web)?,
        json!({
            "pageUrl": "https://example.com",
            "v3Url": "https://example.com/v3.js",
            "ua": "Mozilla/5.0",
            "lang": "en",
            "index": 1,
            "abck": "abck-value",
            "bmsz": "bmsz-value",
            "script_base64": "c2NyaXB0",
            "encodeData": "state"
        })
    );

    let akamai_sbsd = AkamaiSbsdTask {
        page_url: "https://example.com".to_owned(),
        sbsd_url: "https://example.com/sbsd.js".to_owned(),
        bm_so: "bm-so".to_owned(),
        ua: "Mozilla/5.0".to_owned(),
        lang: "en".to_owned(),
        script_base64: "c2NyaXB0".to_owned(),
        ..Default::default()
    };
    assert_eq!(
        serde_json::to_value(akamai_sbsd)?,
        json!({
            "pageUrl": "https://example.com",
            "sbsdUrl": "https://example.com/sbsd.js",
            "bmSo": "bm-so",
            "ua": "Mozilla/5.0",
            "lang": "en",
            "script_base64": "c2NyaXB0"
        })
    );

    let cloudflare_5s = Cloudflare5sTask {
        website_url: "https://example.com".to_owned(),
        proxy: "http://user:pass@127.0.0.1:8080".to_owned(),
        rq_data: Some(
            [("cType".to_owned(), json!("managed"))]
                .into_iter()
                .collect(),
        ),
        ..Default::default()
    };
    assert_eq!(
        serde_json::to_value(cloudflare_5s)?,
        json!({
            "websiteURL": "https://example.com",
            "proxy": "http://user:pass@127.0.0.1:8080",
            "rqData": {"cType": "managed"}
        })
    );

    let turnstile = CloudflareTurnstileTask {
        website_url: "https://example.com".to_owned(),
        website_key: "0x4AAA".to_owned(),
        ..Default::default()
    };
    assert_eq!(
        serde_json::to_value(turnstile)?,
        json!({"websiteURL": "https://example.com", "websiteKey": "0x4AAA"})
    );
    Ok(())
}
