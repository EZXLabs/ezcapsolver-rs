//! Builder compatibility with task defaults and wire serialization.

use ezcapsolver::{
    AkamaiSbsdTask, AkamaiWebTask, Cloudflare5sTask, CloudflareTurnstileTask, DataDomeJsType,
    DataDomeStep, DataDomeTagsTask, DataDomeTask, ExtraFields, FunCaptchaClassificationTask,
    FunCaptchaTask, HcaptchaClassificationTask, HcaptchaTask, IncapsulaTask, PerimeterXTask,
    RecaptchaV2ClassificationTask, RecaptchaV2Task, RecaptchaV2TaskBuilder, RecaptchaV3Task,
    TlsForwardTask, TlsHttpMethod,
};
use serde_json::json;

#[test]
fn every_task_builder_preserves_existing_defaults() -> Result<(), serde_json::Error> {
    // Compare with the existing construction path so per-model defaults cannot drift.
    macro_rules! assert_matches_literal {
        ($task:ident { $($field:ident: $value:expr),* $(,)? }) => {
            let built = $task::builder()$(.$field($value))*.build();
            let literal = $task {
                $($field: $value.into(),)*
                ..Default::default()
            };
            assert_eq!(
                serde_json::to_value(built)?,
                serde_json::to_value(literal)?,
                "{} builder defaults changed the wire payload",
                stringify!($task),
            );
        };
    }

    assert_matches_literal!(RecaptchaV2Task {
        website_url: "https://example.com",
        website_key: "site-key",
    });
    assert_matches_literal!(RecaptchaV3Task {
        website_url: "https://example.com",
        website_key: "site-key",
    });
    assert_matches_literal!(RecaptchaV2ClassificationTask {
        image: "image-data",
        question: "question",
    });
    assert_matches_literal!(FunCaptchaTask {
        website_url: "https://example.com",
        website_key: "site-key",
    });
    assert_matches_literal!(FunCaptchaClassificationTask {
        image: "image-data",
        question: "question",
    });
    assert_matches_literal!(HcaptchaTask {
        website_url: "https://example.com",
        website_key: "site-key",
        lang: "en-US",
        invisible: false,
    });
    assert_matches_literal!(HcaptchaClassificationTask {});
    assert_matches_literal!(Cloudflare5sTask {
        website_url: "https://example.com",
        proxy: "http://127.0.0.1:8080",
    });
    assert_matches_literal!(CloudflareTurnstileTask {
        website_url: "https://example.com",
        website_key: "site-key",
    });
    assert_matches_literal!(AkamaiWebTask {
        page_url: "https://example.com",
        v3_url: "https://example.com/v3.js",
        ua: "user-agent",
        lang: "en",
        index: 0_i32,
    });
    assert_matches_literal!(AkamaiSbsdTask {
        page_url: "https://example.com",
        sbsd_url: "https://example.com/script.js",
        bm_so: "state",
        ua: "user-agent",
        lang: "en",
        script_base64: "script-data",
    });
    assert_matches_literal!(DataDomeTask {
        html_b64: "html-data",
    });
    assert_matches_literal!(DataDomeTagsTask {
        ddk: "key",
        bpc: 1_u64,
        referer: "https://example.com",
        ua: "user-agent",
    });
    assert_matches_literal!(IncapsulaTask {});
    assert_matches_literal!(PerimeterXTask {
        website_key: "site-key",
    });
    assert_matches_literal!(TlsForwardTask {
        tls_type: "fingerprint",
        proxy: "http://127.0.0.1:8080",
        url: "https://example.com",
    });
    Ok(())
}

#[test]
fn string_setters_accept_borrowed_owned_and_optional_values() -> Result<(), serde_json::Error> {
    let website_url = "https://example.com".to_owned();
    let website_key = "site-key".to_owned();
    let proxy = Some("http://127.0.0.1:8080");
    let title: Option<String> = None;
    let builder: RecaptchaV2TaskBuilder = RecaptchaV2Task::builder();
    let task = builder
        .website_url(&website_url)
        .website_key(website_key)
        .s("challenge-data")
        .maybe_sa(Some("anchor".to_owned()))
        .maybe_proxy(proxy)
        .maybe_website_title(title)
        .build();

    assert_eq!(task.website_url, website_url);
    assert_eq!(task.website_key, "site-key");
    assert_eq!(task.s.as_deref(), Some("challenge-data"));
    assert_eq!(task.sa.as_deref(), Some("anchor"));
    assert_eq!(task.proxy.as_deref(), proxy);
    assert!(task.website_title.is_none());
    assert_eq!(
        serde_json::to_value(task)?,
        json!({
            "websiteURL": "https://example.com",
            "websiteKey": "site-key",
            "isInvisible": false,
            "s": "challenge-data",
            "sa": "anchor",
            "proxy": "http://127.0.0.1:8080"
        })
    );
    Ok(())
}

#[test]
fn builder_defaults_can_be_overridden() {
    let v3 = RecaptchaV3Task::builder()
        .website_url("https://example.com")
        .website_key("site-key")
        .is_invisible(false)
        .page_action("action")
        .build();
    assert!(!v3.is_invisible);
    assert_eq!(v3.page_action.as_deref(), Some("action"));

    let classification = RecaptchaV2ClassificationTask::builder()
        .image("image-data")
        .question("question")
        .size(1)
        .build();
    assert_eq!(classification.size, 1);

    let data_dome = DataDomeTask::builder()
        .html_b64("html-data")
        .step(DataDomeStep::Two)
        .image("image-data")
        .build();
    assert_eq!(data_dome.step, DataDomeStep::Two);
    assert_eq!(data_dome.image, "image-data");

    let tags = DataDomeTagsTask::builder()
        .ddk("key")
        .bpc(2)
        .referer("https://example.com")
        .ua("user-agent")
        .jstype(DataDomeJsType::Le)
        .cid("session")
        .build();
    assert_eq!(tags.jstype, DataDomeJsType::Le);
    assert_eq!(tags.cid, "session");
}

#[test]
fn optional_collections_and_json_values_keep_their_wire_shape() -> Result<(), serde_json::Error> {
    let task = TlsForwardTask::builder()
        .tls_type("fingerprint")
        .proxy("http://127.0.0.1:8080")
        .url("https://example.com")
        .method(TlsHttpMethod::Post)
        .headers(ExtraFields::from([(
            "accept".into(),
            json!("application/json"),
        )]))
        .maybe_cookies(None)
        .headers_order("accept")
        .body(json!({"data": [1, 2]}))
        .extra(ExtraFields::from([("futureFlag".into(), json!(true))]))
        .build();
    assert_eq!(
        serde_json::to_value(task)?,
        json!({
            "tls_type": "fingerprint",
            "proxy": "http://127.0.0.1:8080",
            "url": "https://example.com",
            "method": "POST",
            "headers": {"accept": "application/json"},
            "headers_order": "accept",
            "body": {"data": [1, 2]},
            // Always sent: the builder defaults it to false rather than
            // leaving the worker to guess how `body` is encoded.
            "body_raw": false,
            "futureFlag": true
        })
    );

    let classification = HcaptchaClassificationTask::builder()
        .images(vec!["first".to_owned(), "second".to_owned()])
        .maybe_anchors(Some(vec!["anchor".to_owned()]))
        .question("question")
        .build();
    assert_eq!(
        serde_json::to_value(classification)?,
        json!({"images": ["first", "second"], "anchors": ["anchor"], "question": "question"})
    );
    Ok(())
}

#[test]
fn runtime_none_uses_the_models_special_default() {
    let task = RecaptchaV3Task::builder()
        .website_url("https://example.com")
        .website_key("site-key")
        .maybe_is_invisible(None)
        .build();
    assert!(task.is_invisible);

    let classification = RecaptchaV2ClassificationTask::builder()
        .image("image-data")
        .question("question")
        .maybe_size(None)
        .build();
    assert_eq!(classification.size, 4);
}

#[test]
fn model_defaults_stay_inside_the_service_value_ranges() {
    // A derived Default would leave `bpc` at zero, and the service validates it
    // as one or greater — so the struct-literal path would build a request that
    // is rejected before a worker ever sees it. The builder is safe either way,
    // because `bpc` has no builder default and must be set explicitly.
    assert!(
        DataDomeTagsTask::default().bpc >= 1,
        "bpc must default inside the range the service accepts",
    );

    // The two models whose documented default is not the Rust zero value.
    assert!(RecaptchaV3Task::default().is_invisible);
    assert_eq!(RecaptchaV2ClassificationTask::default().size, 4);

    // Closed enums default to the value that starts each flow.
    assert_eq!(DataDomeTask::default().step, DataDomeStep::One);
    assert_eq!(DataDomeTagsTask::default().jstype, DataDomeJsType::Ch);
    assert_eq!(TlsForwardTask::default().method, TlsHttpMethod::Get);
}
