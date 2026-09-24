use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use notification_api::{AppState, SYNTHETIC_RECIPIENT_ID, router, seed_caller};
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

async fn call(
    app: axum::Router,
    method: &str,
    uri: &str,
    api_key: Option<&str>,
    idempotency_key: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut request = Request::builder().method(method).uri(uri);
    if let Some(api_key) = api_key {
        request = request.header("x-api-key", api_key);
    }
    if let Some(idempotency_key) = idempotency_key {
        request = request.header("idempotency-key", idempotency_key);
    }
    let request_body = if let Some(body) = body {
        request = request.header("content-type", "application/json");
        Body::from(body.to_string())
    } else {
        Body::empty()
    };
    let response = app
        .oneshot(request.body(request_body).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = serde_json::from_slice(&body).unwrap_or_else(|_| json!({}));
    (status, body)
}

async fn post(app: axum::Router, key: &str, idem: &str, body: Value) -> (StatusCode, Value) {
    call(
        app,
        "POST",
        "/api/v1/notifications",
        Some(key),
        Some(idem),
        Some(body),
    )
    .await
}

#[sqlx::test(migrations = "../database/migrations")]
async fn acceptance_is_durable_idempotent_and_conflicts_on_changed_payload(pool: PgPool) {
    let key = format!("test-{}", Uuid::new_v4());
    seed_caller(&pool, &key).await.unwrap();
    let app = router(AppState::new(pool.clone(), true));
    let request = json!({
        "recipient_id": SYNTHETIC_RECIPIENT_ID,
        "channel": "email",
        "subject": "Integration test",
        "body": "Synthetic message",
        "variables": {}
    });
    let (first_status, first) = post(app.clone(), &key, "same-request", request.clone()).await;
    assert_eq!(first_status, StatusCode::ACCEPTED);
    let (repeat_status, repeated) = post(app.clone(), &key, "same-request", request.clone()).await;
    assert_eq!(repeat_status, StatusCode::ACCEPTED);
    assert_eq!(first["notification_id"], repeated["notification_id"]);

    let changed = json!({
        "recipient_id": SYNTHETIC_RECIPIENT_ID,
        "channel": "email",
        "subject": "Changed",
        "body": "Synthetic message",
        "variables": {}
    });
    let (conflict_status, _) = post(app, &key, "same-request", changed).await;
    assert_eq!(conflict_status, StatusCode::CONFLICT);

    let notification_id = first["notification_id"].as_str().unwrap();
    let counts = sqlx::query("SELECT (SELECT count(*) FROM notifications WHERE id = $1::uuid) AS notifications, (SELECT count(*) FROM outbox WHERE notification_id = $1::uuid) AS outbox")
        .bind(notification_id).fetch_one(&pool).await.unwrap();
    use sqlx::Row;
    assert_eq!(counts.get::<i64, _>("notifications"), 1);
    assert_eq!(counts.get::<i64, _>("outbox"), 1);
}

#[sqlx::test(migrations = "../database/migrations")]
async fn acceptance_rejects_opted_out_channel_and_pins_template_version(pool: PgPool) {
    let key = format!("test-{}", Uuid::new_v4());
    seed_caller(&pool, &key).await.unwrap();
    let recipient = Uuid::parse_str(SYNTHETIC_RECIPIENT_ID).unwrap();
    sqlx::query(
        "UPDATE preferences SET opt_in = FALSE WHERE recipient_id = $1 AND channel = 'sms'",
    )
    .bind(recipient)
    .execute(&pool)
    .await
    .unwrap();
    let app = router(AppState::new(pool.clone(), true));
    let sms = json!({"recipient_id":recipient,"channel":"sms","subject":"","body":"Synthetic SMS","variables":{}});
    let (status, _) = post(app.clone(), &key, "opt-out", sms).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let template_id = Uuid::parse_str("22222222-2222-4222-8222-222222222224").unwrap();
    let email = json!({"recipient_id":recipient,"channel":"email","template_id":template_id,"variables":{"order_id":"A-100"}});
    let (status, accepted) = post(app, &key, "template-pin", email).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    let row = sqlx::query("SELECT template_version, payload ->> 'body' AS body FROM notifications WHERE id = $1::uuid")
        .bind(accepted["notification_id"].as_str().unwrap()).fetch_one(&pool).await.unwrap();
    use sqlx::Row;
    assert_eq!(row.get::<i32, _>("template_version"), 1);
    assert_eq!(
        row.get::<String, _>("body"),
        "<p>Your order {{order_id}} is ready.</p>"
    );
}

#[sqlx::test(migrations = "../database/migrations")]
async fn status_and_recipient_routes_require_auth_and_return_only_masked_destinations(
    pool: PgPool,
) {
    let health = call(
        router(AppState::new(pool.clone(), true)),
        "GET",
        "/healthz",
        None,
        None,
        None,
    )
    .await;
    assert_eq!(health.0, StatusCode::OK);
    assert_eq!(health.1["service"], "notification-api");

    let readiness = call(
        router(AppState::new(pool.clone(), true)),
        "GET",
        "/readyz",
        None,
        None,
        None,
    )
    .await;
    assert_eq!(readiness.0, StatusCode::OK);
    assert_eq!(readiness.1["status"], "ready");

    let metrics = call(
        router(AppState::new(pool.clone(), true)),
        "GET",
        "/metrics",
        None,
        None,
        None,
    )
    .await;
    assert_eq!(metrics.0, StatusCode::OK);

    let key = format!("test-{}", Uuid::new_v4());
    seed_caller(&pool, &key).await.unwrap();
    let app = router(AppState::new(pool.clone(), true));
    let (unauthorized, _) = call(app.clone(), "GET", "/api/v1/recipients", None, None, None).await;
    assert_eq!(unauthorized, StatusCode::UNAUTHORIZED);

    let (recipient_status, recipients) = call(
        app.clone(),
        "GET",
        "/api/v1/recipients",
        Some(&key),
        None,
        None,
    )
    .await;
    assert_eq!(recipient_status, StatusCode::OK);
    assert_eq!(recipients["items"][0]["email_masked"], "a***@example.test");
    let serialized = recipients.to_string();
    assert!(!serialized.contains("alex@example.test"));
    assert!(!serialized.contains("+15555550100"));

    let request = json!({
        "recipient_id": SYNTHETIC_RECIPIENT_ID,
        "channel": "email",
        "subject": "Status lookup",
        "body": "Synthetic content",
        "variables": {}
    });
    let (accepted_status, accepted) = post(app.clone(), &key, "status-lookup", request).await;
    assert_eq!(accepted_status, StatusCode::ACCEPTED);
    let notification_id = accepted["notification_id"].as_str().unwrap();
    let notification_id = Uuid::parse_str(notification_id).unwrap();
    sqlx::query("INSERT INTO delivery_attempts (notification_id, attempt_number, outcome, started_at) VALUES ($1,1,'started',now())")
        .bind(notification_id)
        .execute(&pool)
        .await
        .unwrap();

    let (detail_status, detail) = call(
        app.clone(),
        "GET",
        &format!("/api/v1/notifications/{notification_id}"),
        Some(&key),
        None,
        None,
    )
    .await;
    assert_eq!(detail_status, StatusCode::OK);
    assert_eq!(detail["attempts"][0]["outcome"], "started");
    assert!(detail.get("payload").is_none());

    let (list_status, list) = call(
        app.clone(),
        "GET",
        &format!(
            "/api/v1/notifications?recipient_id={SYNTHETIC_RECIPIENT_ID}&status=accepted&limit=0"
        ),
        Some(&key),
        None,
        None,
    )
    .await;
    assert_eq!(list_status, StatusCode::OK);
    assert_eq!(list["items"].as_array().unwrap().len(), 1);

    let (filtered_status, filtered) = call(
        app.clone(),
        "GET",
        "/api/v1/notifications?status=sent&limit=101",
        Some(&key),
        None,
        None,
    )
    .await;
    assert_eq!(filtered_status, StatusCode::OK);
    assert_eq!(filtered["items"].as_array().unwrap().len(), 0);

    let (missing_status, _) = call(
        app,
        "GET",
        &format!("/api/v1/notifications/{}", Uuid::new_v4()),
        Some(&key),
        None,
        None,
    )
    .await;
    assert_eq!(missing_status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../database/migrations")]
async fn preference_recipient_and_device_routes_validate_and_persist(pool: PgPool) {
    let key = format!("test-{}", Uuid::new_v4());
    seed_caller(&pool, &key).await.unwrap();
    let app = router(AppState::new(pool.clone(), true));
    let recipient_id = SYNTHETIC_RECIPIENT_ID;

    let (preference_status, preference) = call(
        app.clone(),
        "GET",
        &format!("/api/v1/recipients/{recipient_id}/preferences/email"),
        Some(&key),
        None,
        None,
    )
    .await;
    assert_eq!(preference_status, StatusCode::OK);
    assert_eq!(preference["opt_in"], true);

    let (invalid_channel, _) = call(
        app.clone(),
        "GET",
        &format!("/api/v1/recipients/{recipient_id}/preferences/pager"),
        Some(&key),
        None,
        None,
    )
    .await;
    assert_eq!(invalid_channel, StatusCode::BAD_REQUEST);

    let missing_recipient = Uuid::new_v4();
    let (missing_preference, _) = call(
        app.clone(),
        "GET",
        &format!("/api/v1/recipients/{missing_recipient}/preferences/email"),
        Some(&key),
        None,
        None,
    )
    .await;
    assert_eq!(missing_preference, StatusCode::NOT_FOUND);

    let (updated_status, updated) = call(
        app.clone(),
        "PUT",
        &format!("/api/v1/recipients/{recipient_id}/preferences/email"),
        Some(&key),
        None,
        Some(json!({"opt_in": false})),
    )
    .await;
    assert_eq!(updated_status, StatusCode::OK);
    assert_eq!(updated["opt_in"], false);

    let invalid_recipient = call(
        app.clone(),
        "PUT",
        &format!("/api/v1/recipients/{missing_recipient}/preferences/email"),
        Some(&key),
        None,
        Some(json!({"opt_in": true})),
    )
    .await;
    assert_eq!(invalid_recipient.0, StatusCode::NOT_FOUND);

    for request in [
        json!({"display_name":"  ","email":"alex@example.test","phone_number":"+15555550100"}),
        json!({"display_name":"Alex","email":"not-an-address","phone_number":"+15555550100"}),
        json!({"display_name":"Alex","email":"alex@example.test","phone_number":"x".repeat(33)}),
    ] {
        let (status, _) = call(
            app.clone(),
            "PUT",
            &format!("/api/v1/recipients/{recipient_id}"),
            Some(&key),
            None,
            Some(request),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    let (updated_recipient_status, updated_recipient) = call(
        app.clone(),
        "PUT",
        &format!("/api/v1/recipients/{recipient_id}"),
        Some(&key),
        None,
        Some(json!({"display_name":"Alexandra Example","email":"alexandra@example.test","phone_number":"+15555550199"})),
    )
    .await;
    assert_eq!(updated_recipient_status, StatusCode::OK);
    assert_eq!(updated_recipient["email_masked"], "a***@example.test");
    assert_eq!(updated_recipient["phone_masked"], "***99");

    let (missing_update, _) = call(
        app.clone(),
        "PUT",
        &format!("/api/v1/recipients/{missing_recipient}"),
        Some(&key),
        None,
        Some(json!({"display_name":"Alex","email":null,"phone_number":null})),
    )
    .await;
    assert_eq!(missing_update, StatusCode::NOT_FOUND);

    for request in [
        json!({"platform":"web","device_token":"token-1"}),
        json!({"platform":"ios","device_token":"  "}),
        json!({"platform":"android","device_token":"x".repeat(513)}),
    ] {
        let (status, _) = call(
            app.clone(),
            "POST",
            &format!("/api/v1/recipients/{recipient_id}/devices"),
            Some(&key),
            None,
            Some(request),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    let device_request = json!({"platform":"ios","device_token":"local-test-device-token"});
    let (created_status, created) = call(
        app.clone(),
        "POST",
        &format!("/api/v1/recipients/{recipient_id}/devices"),
        Some(&key),
        None,
        Some(device_request.clone()),
    )
    .await;
    let (repeated_status, repeated) = call(
        app.clone(),
        "POST",
        &format!("/api/v1/recipients/{recipient_id}/devices"),
        Some(&key),
        None,
        Some(device_request),
    )
    .await;
    assert_eq!(created_status, StatusCode::CREATED);
    assert_eq!(repeated_status, StatusCode::CREATED);
    assert_eq!(created["device_id"], repeated["device_id"]);

    let (missing_device, _) = call(
        app,
        "POST",
        &format!("/api/v1/recipients/{missing_recipient}/devices"),
        Some(&key),
        None,
        Some(json!({"platform":"ios","device_token":"local-test-device-token"})),
    )
    .await;
    assert_eq!(missing_device, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "../database/migrations")]
async fn templates_are_immutable_versioned_and_validated(pool: PgPool) {
    let key = format!("test-{}", Uuid::new_v4());
    seed_caller(&pool, &key).await.unwrap();
    let app = router(AppState::new(pool, true));
    let template_id = Uuid::new_v4();
    let path = "/api/v1/templates";

    let (list_status, listed) = call(app.clone(), "GET", path, Some(&key), None, None).await;
    assert_eq!(list_status, StatusCode::OK);
    assert_eq!(listed["items"].as_array().unwrap().len(), 4);

    let request = json!({
        "id": template_id,
        "name": "Account notice",
        "channel": "email",
        "subject": "Version one",
        "body": "Hello {{name}}"
    });
    let (first_status, first) = call(
        app.clone(),
        "POST",
        path,
        Some(&key),
        None,
        Some(request.clone()),
    )
    .await;
    assert_eq!(first_status, StatusCode::CREATED);
    assert_eq!(first["version"], 1);

    let mut second_request = request.clone();
    second_request["subject"] = json!("Version two");
    let (second_status, second) = call(
        app.clone(),
        "POST",
        path,
        Some(&key),
        None,
        Some(second_request),
    )
    .await;
    assert_eq!(second_status, StatusCode::CREATED);
    assert_eq!(second["version"], 2);

    let (latest_status, latest) = call(app.clone(), "GET", path, Some(&key), None, None).await;
    assert_eq!(latest_status, StatusCode::OK);
    let latest_item = latest["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["template_id"] == template_id.to_string())
        .unwrap();
    assert_eq!(latest_item["version"], 2);

    for invalid in [
        json!({"name":" ","channel":"email","subject":"","body":"body"}),
        json!({"name":"name","channel":"email","subject":"","body":" "}),
        json!({"name":"name","channel":"email","subject":"x".repeat(201),"body":"body"}),
    ] {
        let (status, _) = call(app.clone(), "POST", path, Some(&key), None, Some(invalid)).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    let invalid_channel = json!({"name":"name","channel":"pager","subject":"","body":"body"});
    let (invalid_status, _) = call(
        app.clone(),
        "POST",
        path,
        Some(&key),
        None,
        Some(invalid_channel),
    )
    .await;
    assert_eq!(invalid_status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test(migrations = "../database/migrations")]
async fn notification_acceptance_validates_headers_content_and_scheduling(pool: PgPool) {
    let key = format!("test-{}", Uuid::new_v4());
    seed_caller(&pool, &key).await.unwrap();
    let enabled_app = router(AppState::new(pool.clone(), true));
    let body = json!({
        "recipient_id": SYNTHETIC_RECIPIENT_ID,
        "channel": "email",
        "subject": "Scheduled",
        "body": "Synthetic scheduled message",
        "variables": {}
    });

    let (missing_auth, _) = call(
        enabled_app.clone(),
        "POST",
        "/api/v1/notifications",
        None,
        Some("header-test"),
        Some(body.clone()),
    )
    .await;
    assert_eq!(missing_auth, StatusCode::UNAUTHORIZED);
    let (invalid_auth, _) = call(
        enabled_app.clone(),
        "POST",
        "/api/v1/notifications",
        Some("unknown-key"),
        Some("header-test"),
        Some(body.clone()),
    )
    .await;
    assert_eq!(invalid_auth, StatusCode::UNAUTHORIZED);
    let (missing_idempotency, _) = call(
        enabled_app.clone(),
        "POST",
        "/api/v1/notifications",
        Some(&key),
        None,
        Some(body.clone()),
    )
    .await;
    assert_eq!(missing_idempotency, StatusCode::BAD_REQUEST);
    let (empty_idempotency, _) = call(
        enabled_app.clone(),
        "POST",
        "/api/v1/notifications",
        Some(&key),
        Some(""),
        Some(body.clone()),
    )
    .await;
    assert_eq!(empty_idempotency, StatusCode::BAD_REQUEST);
    let (oversized_idempotency, _) = call(
        enabled_app.clone(),
        "POST",
        "/api/v1/notifications",
        Some(&key),
        Some(&"x".repeat(129)),
        Some(body.clone()),
    )
    .await;
    assert_eq!(oversized_idempotency, StatusCode::BAD_REQUEST);

    let invalid_content =
        json!({"recipient_id":SYNTHETIC_RECIPIENT_ID,"channel":"email","variables":{}});
    let (content_status, _) =
        post(enabled_app.clone(), &key, "content-choice", invalid_content).await;
    assert_eq!(content_status, StatusCode::BAD_REQUEST);

    let invalid_template_version = json!({
        "recipient_id": SYNTHETIC_RECIPIENT_ID,
        "channel": "email",
        "template_version": 1,
        "subject": "Subject",
        "body": "Body",
        "variables": {}
    });
    let (version_status, _) = post(
        enabled_app.clone(),
        &key,
        "version-without-template",
        invalid_template_version,
    )
    .await;
    assert_eq!(version_status, StatusCode::BAD_REQUEST);

    let missing_template = json!({
        "recipient_id": SYNTHETIC_RECIPIENT_ID,
        "channel": "email",
        "template_id": Uuid::new_v4(),
        "variables": {}
    });
    let (template_status, _) = post(
        enabled_app.clone(),
        &key,
        "missing-template",
        missing_template,
    )
    .await;
    assert_eq!(template_status, StatusCode::NOT_FOUND);

    let missing_recipient = json!({
        "recipient_id": Uuid::new_v4(),
        "channel": "email",
        "subject": "Subject",
        "body": "Body",
        "variables": {}
    });
    let (recipient_status, _) = post(
        enabled_app.clone(),
        &key,
        "missing-recipient",
        missing_recipient,
    )
    .await;
    assert_eq!(recipient_status, StatusCode::NOT_FOUND);

    let scheduled_at = (chrono::Utc::now() + chrono::Duration::minutes(10)).to_rfc3339();
    let mut scheduled_body = body.clone();
    scheduled_body["scheduled_at"] = json!(scheduled_at);
    let (scheduled_status, scheduled) =
        post(enabled_app, &key, "scheduled-request", scheduled_body).await;
    assert_eq!(scheduled_status, StatusCode::ACCEPTED);
    assert_eq!(scheduled["status"], "scheduled");

    let disabled_app = router(AppState::new(pool, false));
    let mut simulation_body = body;
    simulation_body["simulation"] = json!("success");
    let (simulation_status, _) =
        post(disabled_app, &key, "simulation-disabled", simulation_body).await;
    assert_eq!(simulation_status, StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "../database/migrations")]
async fn notification_acceptance_enforces_per_caller_rate_limit(pool: PgPool) {
    let key = format!("test-{}", Uuid::new_v4());
    seed_caller(&pool, &key).await.unwrap();
    let app = router(AppState::new(pool, true));
    let request = json!({
        "recipient_id": SYNTHETIC_RECIPIENT_ID,
        "channel": "email",
        "subject": "Rate limit test",
        "body": "Synthetic content",
        "variables": {}
    });

    for attempt in 1..=120 {
        let (status, _) = post(
            app.clone(),
            &key,
            &format!("rate-{attempt}"),
            request.clone(),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::ACCEPTED,
            "request {attempt} should remain within the limit"
        );
    }
    let (status, _) = post(app, &key, "rate-121", request).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
}
