use anyhow::{Context, Result};
use axum::{Json, Router, extract::State, response::IntoResponse, routing::get};
use chrono::{Duration as ChronoDuration, Utc};
use futures_util::StreamExt;
use lapin::{
    BasicProperties, Channel as AmqpChannel, Confirmation, Connection, ConnectionProperties,
    ExchangeKind,
    options::{
        BasicAckOptions, BasicConsumeOptions, BasicPublishOptions, BasicQosOptions,
        BasicRejectOptions, ConfirmSelectOptions, ExchangeDeclareOptions, QueueBindOptions,
        QueueDeclareOptions,
    },
    types::{AMQPValue, FieldTable, LongString, ShortString},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{PgPool, Row, postgres::PgPoolOptions};
use std::{
    env,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::time::sleep;
use tracing::{error, info, warn};
use uuid::Uuid;

pub const CHANNEL_EXCHANGE: &str = "notification.channels";
pub const DEAD_EXCHANGE: &str = "notification.dead";
pub const CHANNELS: [&str; 4] = ["ios_push", "android_push", "sms", "email"];
pub const MAX_ATTEMPTS: i32 = 3;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct DispatchMessage {
    pub notification_id: Uuid,
    pub dispatch_number: i32,
    pub channel: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdapterResult {
    Accepted,
    TransientFailure,
    PermanentFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeliveryDecision {
    Ack,
    DeadLetter,
    Requeue,
}

#[derive(Clone)]
pub struct WorkerState {
    pub pool: PgPool,
    pub broker_ready: Arc<AtomicBool>,
}

pub fn queue_for_channel(channel: &str) -> Option<&'static str> {
    match channel {
        "ios_push" => Some("notifications.ios_push"),
        "android_push" => Some("notifications.android_push"),
        "sms" => Some("notifications.sms"),
        "email" => Some("notifications.email"),
        _ => None,
    }
}

pub fn dead_queue_for_channel(channel: &str) -> Option<String> {
    queue_for_channel(channel).map(|_| format!("notifications.{channel}.dlq"))
}

pub fn retry_delay_seconds(attempt_number: i32) -> i64 {
    2_i64
        .saturating_pow(attempt_number.clamp(1, 5) as u32)
        .min(30)
}

pub fn recording_adapter_result(simulation: Option<&str>, attempt: i32) -> AdapterResult {
    match simulation {
        Some("permanent_failure") => AdapterResult::PermanentFailure,
        Some("transient_failure") if attempt == 1 => AdapterResult::TransientFailure,
        _ => AdapterResult::Accepted,
    }
}

pub fn render_template(source: &str, variables: &Value) -> String {
    let mut rendered = source.to_owned();
    if let Some(values) = variables.as_object() {
        for (key, value) in values {
            let replacement = match value {
                Value::String(value) => value.clone(),
                Value::Null => String::new(),
                _ => value.to_string(),
            };
            rendered = rendered.replace(&format!("{{{{{key}}}}}"), &replacement);
        }
    }
    rendered
}

pub async fn connect_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(20)
        .connect(database_url)
        .await
}

pub async fn ensure_topology(channel: &AmqpChannel) -> Result<()> {
    channel
        .exchange_declare(
            CHANNEL_EXCHANGE.into(),
            ExchangeKind::Direct,
            ExchangeDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;
    channel
        .exchange_declare(
            DEAD_EXCHANGE.into(),
            ExchangeKind::Direct,
            ExchangeDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await?;
    for name in CHANNELS {
        let queue = queue_for_channel(name).expect("known channel");
        let dlq = dead_queue_for_channel(name).expect("known channel");
        let route = name;
        let dead_route = format!("{name}.dead");
        let mut arguments = FieldTable::default();
        arguments.insert(
            ShortString::from("x-dead-letter-exchange"),
            AMQPValue::LongString(LongString::from(DEAD_EXCHANGE)),
        );
        arguments.insert(
            ShortString::from("x-dead-letter-routing-key"),
            AMQPValue::LongString(LongString::from(dead_route.clone())),
        );
        channel
            .queue_declare(
                queue.into(),
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                arguments,
            )
            .await?;
        channel
            .queue_bind(
                queue.into(),
                CHANNEL_EXCHANGE.into(),
                route.into(),
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await?;
        channel
            .queue_declare(
                dlq.clone().into(),
                QueueDeclareOptions {
                    durable: true,
                    ..Default::default()
                },
                FieldTable::default(),
            )
            .await?;
        channel
            .queue_bind(
                dlq.into(),
                DEAD_EXCHANGE.into(),
                dead_route.into(),
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await?;
    }
    Ok(())
}

async fn publish_confirmed(
    channel: &AmqpChannel,
    routing_key: &str,
    payload: &[u8],
) -> Result<bool> {
    let confirm = channel
        .basic_publish(
            CHANNEL_EXCHANGE.into(),
            routing_key.into(),
            BasicPublishOptions {
                mandatory: true,
                ..Default::default()
            },
            payload,
            BasicProperties::default()
                .with_delivery_mode(2)
                .with_content_type("application/json".into()),
        )
        .await?
        .await?;
    Ok(matches!(confirm, Confirmation::Ack(None)))
}

pub async fn dispatch_due_batch(pool: &PgPool, channel: &AmqpChannel) -> Result<usize> {
    let mut tx = pool.begin().await?;
    let rows = sqlx::query("SELECT id, notification_id, dispatch_number, channel FROM outbox WHERE published_at IS NULL AND available_at <= now() AND (leased_until IS NULL OR leased_until < now()) ORDER BY available_at, created_at LIMIT 50 FOR UPDATE SKIP LOCKED")
        .fetch_all(&mut *tx).await?;
    for row in &rows {
        sqlx::query("UPDATE outbox SET leased_until = now() + interval '30 seconds' WHERE id = $1")
            .bind(row.get::<Uuid, _>("id"))
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;

    let mut published = 0;
    for row in rows {
        let id: Uuid = row.get("id");
        let notification_id: Uuid = row.get("notification_id");
        let dispatch_number: i32 = row.get("dispatch_number");
        let channel_name: String = row.get("channel");
        let message = DispatchMessage {
            notification_id,
            dispatch_number,
            channel: channel_name.clone(),
        };
        let payload = serde_json::to_vec(&message)?;
        let route_exists = queue_for_channel(&channel_name).is_some();
        let confirmed = if route_exists {
            match publish_confirmed(channel, &channel_name, &payload).await {
                Ok(confirmed) => confirmed,
                Err(error) => {
                    sqlx::query("UPDATE outbox SET leased_until = NULL, publish_attempts = publish_attempts + 1, last_error = $2 WHERE id = $1")
                        .bind(id).bind(error.to_string()).execute(pool).await?;
                    warn!(outbox_id = %id, "outbox publish failed");
                    continue;
                }
            }
        } else {
            false
        };
        if confirmed {
            sqlx::query("UPDATE outbox SET published_at = now(), leased_until = NULL, publish_attempts = publish_attempts + 1, last_error = NULL WHERE id = $1")
                .bind(id).execute(pool).await?;
            sqlx::query("UPDATE notifications SET status = CASE WHEN status IN ('accepted','scheduled','retrying') THEN 'queued' ELSE status END, updated_at = now() WHERE id = $1")
                .bind(notification_id).execute(pool).await?;
            published += 1;
        } else {
            sqlx::query("UPDATE outbox SET leased_until = NULL, publish_attempts = publish_attempts + 1, last_error = 'broker did not confirm a routed publish' WHERE id = $1")
                .bind(id).execute(pool).await?;
        }
    }
    Ok(published)
}

pub async fn process_dispatch(
    pool: &PgPool,
    message: &DispatchMessage,
) -> Result<DeliveryDecision> {
    if queue_for_channel(&message.channel).is_none() {
        return Ok(DeliveryDecision::DeadLetter);
    }
    let mut tx = pool.begin().await?;
    let notification = sqlx::query("SELECT id, recipient_id, channel, status, attempts, payload FROM notifications WHERE id = $1 FOR UPDATE")
        .bind(message.notification_id).fetch_optional(&mut *tx).await?;
    let Some(notification) = notification else {
        tx.commit().await?;
        return Ok(DeliveryDecision::Ack);
    };
    let status: String = notification.get("status");
    let attempts: i32 = notification.get("attempts");
    let channel_name: String = notification.get("channel");
    if channel_name != message.channel {
        tx.commit().await?;
        return Ok(DeliveryDecision::DeadLetter);
    }
    if matches!(status.as_str(), "sent" | "suppressed" | "failed") {
        tx.commit().await?;
        return Ok(DeliveryDecision::Ack);
    }
    if message.dispatch_number > attempts + 1 {
        tx.commit().await?;
        return Ok(DeliveryDecision::Requeue);
    }

    let recipient_id: Uuid = notification.get("recipient_id");
    let opted_in = sqlx::query_scalar::<_, bool>("SELECT COALESCE((SELECT opt_in FROM preferences WHERE recipient_id = $1 AND channel = $2), FALSE)")
        .bind(recipient_id).bind(&channel_name).fetch_one(&mut *tx).await?;
    if !opted_in {
        sqlx::query(
            "UPDATE notifications SET status = 'suppressed', updated_at = now() WHERE id = $1",
        )
        .bind(message.notification_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        return Ok(DeliveryDecision::Ack);
    }

    if dispatch_has_recorded_outcome(&mut tx, message, attempts).await? {
        tx.commit().await?;
        return Ok(DeliveryDecision::Ack);
    }

    let attempt = message.dispatch_number;
    let payload: Value = notification.get("payload");
    let simulation = payload.get("simulation").and_then(Value::as_str);
    let adapter_result = recording_adapter_result(simulation, attempt);
    sqlx::query("INSERT INTO delivery_attempts (notification_id, attempt_number, outcome, started_at) VALUES ($1,$2,'started',now()) ON CONFLICT (notification_id, attempt_number) DO NOTHING")
        .bind(message.notification_id).bind(attempt).execute(&mut *tx).await?;
    sqlx::query("UPDATE notifications SET status = 'processing', attempts = GREATEST(attempts,$2), updated_at = now() WHERE id = $1")
        .bind(message.notification_id).bind(attempt).execute(&mut *tx).await?;

    let variables = payload
        .get("variables")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let subject = render_template(
        payload
            .get("subject")
            .and_then(Value::as_str)
            .unwrap_or_default(),
        &variables,
    );
    let body = render_template(
        payload
            .get("body")
            .and_then(Value::as_str)
            .unwrap_or_default(),
        &variables,
    );
    match adapter_result {
        AdapterResult::Accepted => {
            let recipient = sqlx::query("SELECT email, phone_number FROM recipients WHERE id = $1")
                .bind(recipient_id)
                .fetch_one(&mut *tx)
                .await?;
            let destination = if channel_name.ends_with("push") {
                "registered device".to_owned()
            } else if channel_name == "email" {
                mask_destination(
                    recipient
                        .get::<Option<String>, _>("email")
                        .as_deref()
                        .unwrap_or(""),
                )
            } else {
                mask_destination(
                    recipient
                        .get::<Option<String>, _>("phone_number")
                        .as_deref()
                        .unwrap_or(""),
                )
            };
            sqlx::query("UPDATE delivery_attempts SET outcome = 'accepted', completed_at = now() WHERE notification_id = $1 AND attempt_number = $2")
                .bind(message.notification_id).bind(attempt).execute(&mut *tx).await?;
            sqlx::query("INSERT INTO recorded_deliveries (notification_id, channel, masked_destination, rendered_subject, rendered_body) VALUES ($1,$2,$3,$4,$5) ON CONFLICT (notification_id) DO NOTHING")
                .bind(message.notification_id).bind(&channel_name).bind(destination).bind(subject).bind(body).execute(&mut *tx).await?;
            sqlx::query("UPDATE notifications SET status = 'sent', last_error = NULL, updated_at = now() WHERE id = $1")
                .bind(message.notification_id).execute(&mut *tx).await?;
            tx.commit().await?;
            Ok(DeliveryDecision::Ack)
        }
        AdapterResult::TransientFailure => {
            let error_code = "simulated_transient_provider_error";
            sqlx::query("UPDATE delivery_attempts SET outcome = 'transient_error', error_code = $3, completed_at = now() WHERE notification_id = $1 AND attempt_number = $2")
                .bind(message.notification_id).bind(attempt).bind(error_code).execute(&mut *tx).await?;
            if attempt < MAX_ATTEMPTS {
                let next_dispatch = attempt + 1;
                let retry_at = Utc::now() + ChronoDuration::seconds(retry_delay_seconds(attempt));
                sqlx::query("INSERT INTO outbox (notification_id, dispatch_number, channel, available_at) VALUES ($1,$2,$3,$4) ON CONFLICT (notification_id, dispatch_number) DO NOTHING")
                    .bind(message.notification_id).bind(next_dispatch).bind(&channel_name).bind(retry_at).execute(&mut *tx).await?;
                sqlx::query("UPDATE notifications SET status = 'retrying', last_error = $2, updated_at = now() WHERE id = $1")
                    .bind(message.notification_id).bind(error_code).execute(&mut *tx).await?;
                tx.commit().await?;
                Ok(DeliveryDecision::Ack)
            } else {
                sqlx::query("UPDATE notifications SET status = 'failed', last_error = $2, updated_at = now() WHERE id = $1")
                    .bind(message.notification_id).bind(error_code).execute(&mut *tx).await?;
                tx.commit().await?;
                Ok(DeliveryDecision::DeadLetter)
            }
        }
        AdapterResult::PermanentFailure => {
            let error_code = "simulated_permanent_provider_error";
            sqlx::query("UPDATE delivery_attempts SET outcome = 'permanent_error', error_code = $3, completed_at = now() WHERE notification_id = $1 AND attempt_number = $2")
                .bind(message.notification_id).bind(attempt).bind(error_code).execute(&mut *tx).await?;
            sqlx::query("UPDATE notifications SET status = 'failed', last_error = $2, updated_at = now() WHERE id = $1")
                .bind(message.notification_id).bind(error_code).execute(&mut *tx).await?;
            tx.commit().await?;
            Ok(DeliveryDecision::DeadLetter)
        }
    }
}

async fn dispatch_has_recorded_outcome(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    message: &DispatchMessage,
    attempts: i32,
) -> Result<bool> {
    if message.dispatch_number > attempts {
        return Ok(false);
    }
    let outcome = sqlx::query_scalar::<_, String>(
        "SELECT outcome FROM delivery_attempts WHERE notification_id = $1 AND attempt_number = $2",
    )
    .bind(message.notification_id)
    .bind(message.dispatch_number)
    .fetch_optional(&mut **tx)
    .await?;
    Ok(has_recorded_outcome(
        message.dispatch_number,
        attempts,
        outcome.as_deref(),
    ))
}

fn mask_destination(value: &str) -> String {
    if let Some((name, domain)) = value.split_once('@') {
        format!("{}***@{}", name.chars().next().unwrap_or('*'), domain)
    } else {
        let suffix: String = value
            .chars()
            .rev()
            .take(2)
            .collect::<String>()
            .chars()
            .rev()
            .collect();
        format!("***{suffix}")
    }
}

pub async fn consume_channel(
    pool: PgPool,
    connection: &Connection,
    channel_name: &'static str,
) -> Result<()> {
    let channel = connection.create_channel().await?;
    channel.basic_qos(10, BasicQosOptions::default()).await?;
    let queue = queue_for_channel(channel_name).expect("known channel");
    let mut consumer = channel
        .basic_consume(
            queue.into(),
            format!("worker-{channel_name}").into(),
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;
    info!(channel = channel_name, queue, "consumer started");
    while let Some(delivery) = consumer.next().await {
        let delivery = delivery?;
        match serde_json::from_slice::<DispatchMessage>(&delivery.data) {
            Ok(message) => match process_dispatch(&pool, &message).await {
                Ok(DeliveryDecision::Ack) => {
                    delivery.ack(BasicAckOptions::default()).await?;
                }
                Ok(DeliveryDecision::DeadLetter) => {
                    delivery
                        .reject(BasicRejectOptions { requeue: false })
                        .await?;
                }
                Ok(DeliveryDecision::Requeue) => {
                    delivery
                        .reject(BasicRejectOptions { requeue: true })
                        .await?;
                }
                Err(error) => {
                    error!(
                        channel = channel_name,
                        "delivery processing failed: {error:#}"
                    );
                    delivery
                        .reject(BasicRejectOptions { requeue: true })
                        .await?;
                }
            },
            Err(_) => {
                delivery
                    .reject(BasicRejectOptions { requeue: false })
                    .await?;
            }
        }
    }
    Ok(())
}

pub async fn outbox_loop(pool: PgPool, channel: AmqpChannel) {
    loop {
        if let Err(error) = dispatch_due_batch(&pool, &channel).await {
            warn!("outbox dispatch loop error: {error:#}");
        }
        sleep(Duration::from_millis(250)).await;
    }
}

async fn health(State(state): State<WorkerState>) -> (axum::http::StatusCode, Json<Value>) {
    let broker_ready = state.broker_ready.load(Ordering::Relaxed);
    let status = if broker_ready {
        axum::http::StatusCode::OK
    } else {
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    };
    (
        status,
        Json(
            json!({"status":if broker_ready { "ok" } else { "degraded" },"service":"notification-worker","rabbitmq":broker_ready}),
        ),
    )
}
async fn ready(State(state): State<WorkerState>) -> (axum::http::StatusCode, Json<Value>) {
    let database_ready = sqlx::query("SELECT 1").execute(&state.pool).await.is_ok();
    let broker_ready = state.broker_ready.load(Ordering::Relaxed);
    let status = if database_ready && broker_ready {
        axum::http::StatusCode::OK
    } else {
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    };
    (
        status,
        Json(json!({"database":database_ready,"rabbitmq":broker_ready})),
    )
}

async fn metrics(State(state): State<WorkerState>) -> axum::response::Response {
    let counts = match sqlx::query("SELECT outcome, count(*)::bigint AS count FROM delivery_attempts GROUP BY outcome ORDER BY outcome")
        .fetch_all(&state.pool)
        .await {
            Ok(counts) => counts,
            Err(_) => return (axum::http::StatusCode::SERVICE_UNAVAILABLE, "metrics temporarily unavailable").into_response(),
        };
    let unpublished: i64 =
        match sqlx::query_scalar("SELECT count(*) FROM outbox WHERE published_at IS NULL")
            .fetch_one(&state.pool)
            .await
        {
            Ok(count) => count,
            Err(_) => {
                return (
                    axum::http::StatusCode::SERVICE_UNAVAILABLE,
                    "metrics temporarily unavailable",
                )
                    .into_response();
            }
        };
    let mut output = String::from(
        "# HELP notification_worker_delivery_attempts Delivery attempts by outcome.\n# TYPE notification_worker_delivery_attempts gauge\n",
    );
    for row in counts {
        let outcome: String = row.get("outcome");
        let count: i64 = row.get("count");
        output.push_str(&format!(
            "notification_worker_delivery_attempts{{outcome=\"{outcome}\"}} {count}\n"
        ));
    }
    output.push_str("# HELP notification_worker_outbox_unpublished Rows waiting for broker publication.\n# TYPE notification_worker_outbox_unpublished gauge\n");
    output.push_str(&format!(
        "notification_worker_outbox_unpublished {unpublished}\n"
    ));
    output.push_str("# HELP notification_worker_rabbitmq_ready Whether the worker's RabbitMQ connection is ready.\n# TYPE notification_worker_rabbitmq_ready gauge\n");
    output.push_str(&format!(
        "notification_worker_rabbitmq_ready {}\n",
        u8::from(state.broker_ready.load(Ordering::Relaxed))
    ));
    (
        axum::http::StatusCode::OK,
        [(
            axum::http::header::CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8",
        )],
        output,
    )
        .into_response()
}

pub fn health_router(state: WorkerState) -> Router {
    Router::new()
        .route("/healthz", get(health))
        .route("/readyz", get(ready))
        .route("/metrics", get(metrics))
        .with_state(state)
}

pub async fn connect_broker_with_retry(amqp_url: &str) -> Result<Connection> {
    let mut delay = Duration::from_secs(1);
    loop {
        match Connection::connect(amqp_url, ConnectionProperties::default()).await {
            Ok(connection) => return Ok(connection),
            Err(error) => {
                warn!("RabbitMQ connection not ready; retrying: {error}");
                sleep(delay).await;
                delay = (delay * 2).min(Duration::from_secs(15));
            }
        }
    }
}

pub async fn run_worker() -> Result<()> {
    let database_url = env::var("DATABASE_URL").context("DATABASE_URL is required")?;
    let amqp_url = env::var("AMQP_URL").context("AMQP_URL is required")?;
    let pool = connect_pool(&database_url).await?;
    sqlx::migrate!("../database/migrations").run(&pool).await?;
    let connection = Arc::new(connect_broker_with_retry(&amqp_url).await?);
    let publisher = connection.create_channel().await?;
    publisher
        .confirm_select(ConfirmSelectOptions::default())
        .await?;
    ensure_topology(&publisher).await?;
    let broker_ready = Arc::new(AtomicBool::new(true));
    let state = WorkerState {
        pool: pool.clone(),
        broker_ready: broker_ready.clone(),
    };
    let health_address = env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8081".into());
    let listener = tokio::net::TcpListener::bind(&health_address).await?;
    tokio::spawn(async move {
        if let Err(error) = axum::serve(listener, health_router(state)).await {
            error!("health server stopped: {error}");
        }
    });
    tokio::spawn(outbox_loop(pool.clone(), publisher));
    for channel_name in CHANNELS {
        let pool = pool.clone();
        let connection = connection.clone();
        let broker_ready = broker_ready.clone();
        tokio::spawn(async move {
            if let Err(error) = consume_channel(pool, &connection, channel_name).await {
                broker_ready.store(false, Ordering::Relaxed);
                error!(channel = channel_name, "consumer stopped: {error:#}");
            }
        });
    }
    info!("notification worker started with four isolated channel consumers");
    tokio::signal::ctrl_c().await?;
    broker_ready.store(false, Ordering::Relaxed);
    Ok(())
}

fn has_recorded_outcome(dispatch_number: i32, attempts: i32, outcome: Option<&str>) -> bool {
    dispatch_number <= attempts && outcome.is_some_and(|value| value != "started")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use sqlx::postgres::PgConnectOptions;
    use tower::ServiceExt;

    #[test]
    fn every_channel_has_an_isolated_queue_and_dead_letter_queue() {
        for channel in CHANNELS {
            assert_eq!(
                queue_for_channel(channel),
                Some(match channel {
                    "ios_push" => "notifications.ios_push",
                    "android_push" => "notifications.android_push",
                    "sms" => "notifications.sms",
                    "email" => "notifications.email",
                    _ => unreachable!("CHANNELS contains known values"),
                })
            );
            assert_eq!(
                dead_queue_for_channel(channel).as_deref(),
                Some(match channel {
                    "ios_push" => "notifications.ios_push.dlq",
                    "android_push" => "notifications.android_push.dlq",
                    "sms" => "notifications.sms.dlq",
                    "email" => "notifications.email.dlq",
                    _ => unreachable!("CHANNELS contains known values"),
                })
            );
        }
        assert_eq!(queue_for_channel("invalid"), None);
        assert_eq!(dead_queue_for_channel("invalid"), None);
    }

    #[test]
    fn retry_delay_is_exponential_and_bounded() {
        assert_eq!(retry_delay_seconds(1), 2);
        assert_eq!(retry_delay_seconds(2), 4);
        assert_eq!(retry_delay_seconds(5), 30);
        assert_eq!(retry_delay_seconds(20), 30);
    }

    #[test]
    fn local_adapter_has_success_transient_then_success_and_permanent_modes() {
        assert_eq!(
            recording_adapter_result(Some("transient_failure"), 1),
            AdapterResult::TransientFailure
        );
        assert_eq!(
            recording_adapter_result(Some("transient_failure"), 2),
            AdapterResult::Accepted
        );
        assert_eq!(
            recording_adapter_result(Some("permanent_failure"), 1),
            AdapterResult::PermanentFailure
        );
        assert_eq!(recording_adapter_result(None, 1), AdapterResult::Accepted);
    }

    #[test]
    fn duplicate_dispatches_are_acknowledged_only_after_a_recorded_outcome() {
        assert!(has_recorded_outcome(1, 2, Some("accepted")));
        assert!(has_recorded_outcome(2, 2, Some("transient_error")));
        assert!(!has_recorded_outcome(1, 2, Some("started")));
        assert!(!has_recorded_outcome(1, 2, None));
        assert!(!has_recorded_outcome(3, 2, Some("accepted")));
    }

    #[tokio::test]
    async fn broker_disconnect_marks_worker_liveness_unhealthy() {
        let pool = PgPoolOptions::new().connect_lazy_with(
            PgConnectOptions::new()
                .host("127.0.0.1")
                .port(5432)
                .username("notification-test")
                .database("unused"),
        );
        let broker_ready = Arc::new(AtomicBool::new(true));
        let app = health_router(WorkerState {
            pool,
            broker_ready: broker_ready.clone(),
        });
        let request = || Request::get("/healthz").body(Body::empty()).unwrap();

        let healthy = app.clone().oneshot(request()).await.unwrap();
        assert_eq!(healthy.status(), axum::http::StatusCode::OK);

        broker_ready.store(false, Ordering::Relaxed);
        let disconnected = app.oneshot(request()).await.unwrap();
        assert_eq!(
            disconnected.status(),
            axum::http::StatusCode::SERVICE_UNAVAILABLE
        );
    }

    #[tokio::test]
    async fn readiness_and_metrics_report_database_unavailability() {
        let pool = PgPoolOptions::new()
            .acquire_timeout(Duration::from_millis(100))
            .connect_lazy_with(
                PgConnectOptions::new()
                    .host("127.0.0.1")
                    .port(1)
                    .username("notification-test")
                    .database("unused"),
            );
        let app = health_router(WorkerState {
            pool,
            broker_ready: Arc::new(AtomicBool::new(true)),
        });

        let readiness = app
            .clone()
            .oneshot(Request::get("/readyz").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(
            readiness.status(),
            axum::http::StatusCode::SERVICE_UNAVAILABLE
        );

        let metrics = app
            .oneshot(Request::get("/metrics").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(
            metrics.status(),
            axum::http::StatusCode::SERVICE_UNAVAILABLE
        );
    }

    #[test]
    fn renders_values_without_evaluating_template_expressions() {
        let rendered = render_template(
            "Hello {{name}}; order {{id}}",
            &json!({"name":"Alex","id":23}),
        );
        assert_eq!(rendered, "Hello Alex; order 23");
        assert_eq!(render_template("{{unknown}}", &json!({})), "{{unknown}}");
    }
}
