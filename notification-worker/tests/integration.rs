use axum::{
    body::{Body, to_bytes},
    http::Request,
};
use lapin::{
    Connection, ConnectionProperties,
    options::{BasicAckOptions, BasicGetOptions, ConfirmSelectOptions},
};
use notification_worker::{
    DeliveryDecision, DispatchMessage, WorkerState, dispatch_due_batch, ensure_topology,
    health_router, process_dispatch,
};
use serde_json::json;
use sqlx::{PgPool, Row};
use std::{
    env,
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};
use tokio::time::sleep;
use tower::ServiceExt;
use uuid::Uuid;

async fn broker() -> (Connection, lapin::Channel) {
    let url = env::var("TEST_AMQP_URL").expect("TEST_AMQP_URL is required for integration tests");
    let connection = Connection::connect(&url, ConnectionProperties::default())
        .await
        .unwrap();
    let channel = connection.create_channel().await.unwrap();
    channel
        .confirm_select(ConfirmSelectOptions::default())
        .await
        .unwrap();
    ensure_topology(&channel).await.unwrap();
    (connection, channel)
}

async fn create_notification(pool: &PgPool, channel: &str, simulation: &str) -> Uuid {
    let caller_id = Uuid::new_v4();
    sqlx::query("INSERT INTO api_callers (id, name, api_key_sha256) VALUES ($1,'worker integration test',$2)")
        .bind(caller_id).bind(format!("test-{}", Uuid::new_v4())).execute(pool).await.unwrap();
    let recipient_id = Uuid::parse_str("11111111-1111-4111-8111-111111111111").unwrap();
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO notifications (id, caller_id, recipient_id, channel, status, payload) VALUES ($1,$2,$3,$4,'accepted',$5)")
        .bind(id).bind(caller_id).bind(recipient_id).bind(channel)
        .bind(json!({"subject":"test","body":"Hello {{name}}","variables":{"name":"Alex"},"simulation":simulation}))
        .execute(pool).await.unwrap();
    sqlx::query("INSERT INTO outbox (notification_id, dispatch_number, channel, available_at) VALUES ($1,1,$2,now())")
        .bind(id).bind(channel).execute(pool).await.unwrap();
    id
}

#[sqlx::test(migrations = "../database/migrations")]
async fn outbox_confirmation_routes_by_channel_and_transient_failure_recovers(pool: PgPool) {
    let (_connection, publisher) = broker().await;
    let notification_id = create_notification(&pool, "email", "transient_failure").await;
    // Model a worker that claimed the outbox row and stopped before publishing.
    sqlx::query(
        "UPDATE outbox SET leased_until = now() - interval '1 second' WHERE notification_id = $1",
    )
    .bind(notification_id)
    .execute(&pool)
    .await
    .unwrap();
    assert_eq!(dispatch_due_batch(&pool, &publisher).await.unwrap(), 1);

    let email_delivery = publisher
        .basic_get("notifications.email".into(), BasicGetOptions::default())
        .await
        .unwrap()
        .expect("email message routed");
    let email_message: DispatchMessage = serde_json::from_slice(&email_delivery.data).unwrap();
    assert_eq!(email_message.notification_id, notification_id);
    assert_eq!(email_message.channel, "email");
    email_delivery
        .ack(BasicAckOptions::default())
        .await
        .unwrap();
    assert!(
        publisher
            .basic_get("notifications.sms".into(), BasicGetOptions::default())
            .await
            .unwrap()
            .is_none()
    );

    assert_eq!(
        process_dispatch(&pool, &email_message).await.unwrap(),
        DeliveryDecision::Ack
    );
    let status = sqlx::query_scalar::<_, String>("SELECT status FROM notifications WHERE id = $1")
        .bind(notification_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "retrying");
    let retry_outbox = sqlx::query("SELECT dispatch_number, available_at FROM outbox WHERE notification_id = $1 AND dispatch_number = 2")
        .bind(notification_id).fetch_one(&pool).await.unwrap();
    assert!(
        retry_outbox.get::<chrono::DateTime<chrono::Utc>, _>("available_at") > chrono::Utc::now()
    );
    assert_eq!(retry_outbox.get::<i32, _>("dispatch_number"), 2);

    sqlx::query(
        "UPDATE outbox SET available_at = now() WHERE notification_id = $1 AND dispatch_number = 2",
    )
    .bind(notification_id)
    .execute(&pool)
    .await
    .unwrap();
    assert_eq!(dispatch_due_batch(&pool, &publisher).await.unwrap(), 1);
    let retry_delivery = publisher
        .basic_get("notifications.email".into(), BasicGetOptions::default())
        .await
        .unwrap()
        .expect("retry routed to same email queue");
    let retry_message: DispatchMessage = serde_json::from_slice(&retry_delivery.data).unwrap();
    retry_delivery
        .ack(BasicAckOptions::default())
        .await
        .unwrap();
    assert_eq!(retry_message.dispatch_number, 2);
    assert_eq!(
        process_dispatch(&pool, &retry_message).await.unwrap(),
        DeliveryDecision::Ack
    );
    let final_state = sqlx::query("SELECT status, attempts FROM notifications WHERE id = $1")
        .bind(notification_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(final_state.get::<String, _>("status"), "sent");
    assert_eq!(final_state.get::<i32, _>("attempts"), 2);
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM recorded_deliveries WHERE notification_id = $1"
        )
        .bind(notification_id)
        .fetch_one(&pool)
        .await
        .unwrap(),
        1
    );
}

#[sqlx::test(migrations = "../database/migrations")]
async fn worker_restart_redelivery_is_idempotent_and_rechecks_opt_out(pool: PgPool) {
    let (_connection, publisher) = broker().await;
    let notification_id = create_notification(&pool, "email", "success").await;
    let recipient_id = Uuid::parse_str("11111111-1111-4111-8111-111111111111").unwrap();
    sqlx::query(
        "UPDATE preferences SET opt_in = FALSE WHERE recipient_id = $1 AND channel = 'email'",
    )
    .bind(recipient_id)
    .execute(&pool)
    .await
    .unwrap();
    let queued = DispatchMessage {
        notification_id,
        dispatch_number: 1,
        channel: "email".into(),
    };
    assert_eq!(
        process_dispatch(&pool, &queued).await.unwrap(),
        DeliveryDecision::Ack
    );
    let suppressed = sqlx::query("SELECT status, attempts FROM notifications WHERE id = $1")
        .bind(notification_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(suppressed.get::<String, _>("status"), "suppressed");
    assert_eq!(suppressed.get::<i32, _>("attempts"), 0);
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM recorded_deliveries WHERE notification_id = $1"
        )
        .bind(notification_id)
        .fetch_one(&pool)
        .await
        .unwrap(),
        0
    );
    sqlx::query("UPDATE outbox SET published_at = now() WHERE notification_id = $1")
        .bind(notification_id)
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query(
        "UPDATE preferences SET opt_in = TRUE WHERE recipient_id = $1 AND channel = 'email'",
    )
    .bind(recipient_id)
    .execute(&pool)
    .await
    .unwrap();
    let delivered_id = create_notification(&pool, "sms", "success").await;
    assert_eq!(dispatch_due_batch(&pool, &publisher).await.unwrap(), 1);
    let delivery = publisher
        .basic_get("notifications.sms".into(), BasicGetOptions::default())
        .await
        .unwrap()
        .expect("SMS route receives its message");
    let message: DispatchMessage = serde_json::from_slice(&delivery.data).unwrap();
    delivery.ack(BasicAckOptions::default()).await.unwrap();
    assert_eq!(message.notification_id, delivered_id);
    assert_eq!(
        process_dispatch(&pool, &message).await.unwrap(),
        DeliveryDecision::Ack
    );
    // A broker redelivery after the database commit must not record a second effect.
    assert_eq!(
        process_dispatch(&pool, &message).await.unwrap(),
        DeliveryDecision::Ack
    );
    let final_state = sqlx::query("SELECT status, attempts FROM notifications WHERE id = $1")
        .bind(delivered_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(final_state.get::<String, _>("status"), "sent");
    assert_eq!(final_state.get::<i32, _>("attempts"), 1);
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM recorded_deliveries WHERE notification_id = $1"
        )
        .bind(delivered_id)
        .fetch_one(&pool)
        .await
        .unwrap(),
        1
    );
}

#[sqlx::test(migrations = "../database/migrations")]
async fn permanent_failure_is_rejected_to_the_channel_dead_letter_queue(pool: PgPool) {
    let (connection, publisher) = broker().await;
    let connection = std::sync::Arc::new(connection);
    let consumer_connection = connection.clone();
    let consumer_pool = pool.clone();
    let consumer_task = tokio::spawn(async move {
        notification_worker::consume_channel(consumer_pool, &consumer_connection, "email").await
    });
    let notification_id = create_notification(&pool, "email", "permanent_failure").await;
    assert_eq!(dispatch_due_batch(&pool, &publisher).await.unwrap(), 1);
    for _ in 0..60 {
        let status =
            sqlx::query_scalar::<_, String>("SELECT status FROM notifications WHERE id = $1")
                .bind(notification_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        if status == "failed" {
            break;
        }
        sleep(Duration::from_millis(100)).await;
    }
    let status = sqlx::query_scalar::<_, String>("SELECT status FROM notifications WHERE id = $1")
        .bind(notification_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "failed");
    let dlq = publisher
        .basic_get("notifications.email.dlq".into(), BasicGetOptions::default())
        .await
        .unwrap();
    assert!(dlq.is_some(), "permanent failures land in the email DLQ");
    if let Some(delivery) = dlq {
        delivery.ack(BasicAckOptions::default()).await.unwrap();
    }
    consumer_task.abort();
}

#[sqlx::test(migrations = "../database/migrations")]
async fn readiness_and_metrics_report_database_and_delivery_state(pool: PgPool) {
    let notification_id = create_notification(&pool, "email", "success").await;
    sqlx::query("INSERT INTO delivery_attempts (notification_id, attempt_number, outcome, started_at) VALUES ($1,1,'accepted',now())")
        .bind(notification_id)
        .execute(&pool)
        .await
        .unwrap();

    let broker_ready = Arc::new(AtomicBool::new(true));
    let app = health_router(WorkerState {
        pool,
        broker_ready: broker_ready.clone(),
    });

    let readiness = app
        .clone()
        .oneshot(Request::get("/readyz").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(readiness.status(), axum::http::StatusCode::OK);

    let metrics = app
        .clone()
        .oneshot(Request::get("/metrics").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(metrics.status(), axum::http::StatusCode::OK);
    let content = to_bytes(metrics.into_body(), usize::MAX).await.unwrap();
    let metrics = String::from_utf8(content.to_vec()).unwrap();
    assert!(metrics.contains("notification_worker_delivery_attempts{outcome=\"accepted\"} 1"));
    assert!(metrics.contains("notification_worker_outbox_unpublished 1"));
    assert!(metrics.contains("notification_worker_rabbitmq_ready 1"));

    broker_ready.store(false, std::sync::atomic::Ordering::Relaxed);
    let degraded_readiness = app
        .oneshot(Request::get("/readyz").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(
        degraded_readiness.status(),
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    );
}

#[sqlx::test(migrations = "../database/migrations")]
async fn dispatch_validation_dead_letters_unknown_channels_and_requeues_future_attempts(
    pool: PgPool,
) {
    let unknown_channel = DispatchMessage {
        notification_id: Uuid::new_v4(),
        dispatch_number: 1,
        channel: "pager".into(),
    };
    assert_eq!(
        process_dispatch(&pool, &unknown_channel).await.unwrap(),
        DeliveryDecision::DeadLetter
    );

    let missing_notification = DispatchMessage {
        notification_id: Uuid::new_v4(),
        dispatch_number: 1,
        channel: "email".into(),
    };
    assert_eq!(
        process_dispatch(&pool, &missing_notification)
            .await
            .unwrap(),
        DeliveryDecision::Ack
    );

    let notification_id = create_notification(&pool, "email", "success").await;
    let wrong_channel = DispatchMessage {
        notification_id,
        dispatch_number: 1,
        channel: "sms".into(),
    };
    assert_eq!(
        process_dispatch(&pool, &wrong_channel).await.unwrap(),
        DeliveryDecision::DeadLetter
    );

    let future_attempt = DispatchMessage {
        notification_id,
        dispatch_number: 2,
        channel: "email".into(),
    };
    assert_eq!(
        process_dispatch(&pool, &future_attempt).await.unwrap(),
        DeliveryDecision::Requeue
    );

    sqlx::query("UPDATE notifications SET status = 'failed' WHERE id = $1")
        .bind(notification_id)
        .execute(&pool)
        .await
        .unwrap();
    let terminal_redelivery = DispatchMessage {
        notification_id,
        dispatch_number: 1,
        channel: "email".into(),
    };
    assert_eq!(
        process_dispatch(&pool, &terminal_redelivery).await.unwrap(),
        DeliveryDecision::Ack
    );
}
