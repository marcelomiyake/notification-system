pub mod domain;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use domain::{Channel, NotificationRequest};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row, postgres::PgPoolOptions};
use std::{
    collections::{HashMap, VecDeque},
    str::FromStr,
    sync::{Arc, Mutex},
    time::Instant,
};
use thiserror::Error;
use uuid::Uuid;

pub const SYNTHETIC_RECIPIENT_ID: &str = "11111111-1111-4111-8111-111111111111";
pub const DEMO_CALLER_ID: &str = "99999999-9999-4999-8999-999999999999";

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    rate_limiter: Arc<Mutex<HashMap<Uuid, VecDeque<Instant>>>>,
    pub simulation_enabled: bool,
}

impl AppState {
    pub fn new(pool: PgPool, simulation_enabled: bool) -> Self {
        Self {
            pool,
            rate_limiter: Arc::new(Mutex::new(HashMap::new())),
            simulation_enabled,
        }
    }

    fn allow_request(&self, caller: Uuid) -> bool {
        self.allow_request_at(caller, Instant::now())
    }

    fn allow_request_at(&self, caller: Uuid, now: Instant) -> bool {
        let mut buckets = self
            .rate_limiter
            .lock()
            .expect("rate limiter mutex poisoned");
        let bucket = buckets.entry(caller).or_default();
        while bucket
            .front()
            .is_some_and(|seen| now.duration_since(*seen).as_secs() >= 60)
        {
            bucket.pop_front();
        }
        if bucket.len() >= 120 {
            return false;
        }
        bucket.push_back(now);
        true
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(health))
        .route("/readyz", get(ready))
        .route("/metrics", get(metrics))
        .route(
            "/api/v1/notifications",
            post(create_notification).get(list_notifications),
        )
        .route(
            "/api/v1/notifications/{notification_id}",
            get(get_notification),
        )
        .route("/api/v1/recipients", get(list_recipients))
        .route(
            "/api/v1/recipients/{recipient_id}",
            axum::routing::put(update_recipient),
        )
        .route(
            "/api/v1/recipients/{recipient_id}/preferences/{channel}",
            get(get_preference).put(put_preference),
        )
        .route(
            "/api/v1/recipients/{recipient_id}/devices",
            post(register_device),
        )
        .route(
            "/api/v1/templates",
            get(list_templates).post(create_template),
        )
        .with_state(state)
}

#[derive(Debug, Error)]
enum ApiError {
    #[error("unauthorized")]
    Unauthorized,
    #[error("rate limit exceeded")]
    RateLimited,
    #[error("not found")]
    NotFound,
    #[error("invalid request: {0}")]
    BadRequest(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("dependency unavailable")]
    Unavailable,
    #[error("internal server error")]
    Internal,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let message = match &self {
            Self::BadRequest(message) | Self::Conflict(message) => message.clone(),
            _ => self.to_string(),
        };
        (status, Json(json!({"error": message}))).into_response()
    }
}

fn key_hash(raw: &str) -> String {
    hex::encode(Sha256::digest(raw.as_bytes()))
}

async fn authenticate(headers: &HeaderMap, state: &AppState) -> Result<Uuid, ApiError> {
    let raw = headers
        .get("x-api-key")
        .and_then(|value| value.to_str().ok())
        .ok_or(ApiError::Unauthorized)?;
    let row =
        sqlx::query("SELECT id FROM api_callers WHERE api_key_sha256 = $1 AND enabled = TRUE")
            .bind(key_hash(raw))
            .fetch_optional(&state.pool)
            .await
            .map_err(|_| ApiError::Unavailable)?
            .ok_or(ApiError::Unauthorized)?;
    Ok(row.get("id"))
}

async fn health() -> Json<Value> {
    Json(json!({"status":"ok","service":"notification-api"}))
}

async fn ready(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    sqlx::query("SELECT 1")
        .execute(&state.pool)
        .await
        .map_err(|_| ApiError::Unavailable)?;
    Ok(Json(json!({"status":"ready"})))
}

async fn metrics(State(state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    let counts = sqlx::query("SELECT status, count(*)::bigint AS count FROM notifications GROUP BY status ORDER BY status")
        .fetch_all(&state.pool)
        .await
        .map_err(|_| ApiError::Unavailable)?;
    let mut output = String::from(
        "# HELP notification_api_notifications Current notification records by lifecycle status.\n# TYPE notification_api_notifications gauge\n",
    );
    for row in counts {
        let status: String = row.get("status");
        let count: i64 = row.get("count");
        output.push_str(&format!(
            "notification_api_notifications{{status=\"{status}\"}} {count}\n"
        ));
    }
    Ok((
        [(
            header::CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8",
        )],
        output,
    ))
}

#[derive(Serialize)]
struct AcceptedNotification {
    notification_id: Uuid,
    status: String,
    created_at: DateTime<Utc>,
}

async fn create_notification(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<NotificationRequest>,
) -> Result<(StatusCode, Json<AcceptedNotification>), ApiError> {
    let caller_id = authenticate(&headers, &state).await?;
    if !state.allow_request(caller_id) {
        return Err(ApiError::RateLimited);
    }
    let idem_key = headers
        .get("idempotency-key")
        .and_then(|value| value.to_str().ok())
        .filter(|value| {
            !value.is_empty()
                && value.len() <= 128
                && value.bytes().all(|byte| (0x21..=0x7e).contains(&byte))
        })
        .ok_or_else(|| {
            ApiError::BadRequest(
                "Idempotency-Key must contain 1-128 visible ASCII characters".into(),
            )
        })?;
    request
        .validate(Utc::now(), state.simulation_enabled)
        .map_err(|error| ApiError::BadRequest(error.to_string()))?;
    let payload_hash = request.payload_hash().map_err(|_| ApiError::Internal)?;

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|_| ApiError::Unavailable)?;
    // Serialize reuse of one scoped key across API replicas before checking/creating it.
    sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1), hashtext($2))")
        .bind(caller_id.to_string())
        .bind(idem_key)
        .execute(&mut *tx)
        .await
        .map_err(|_| ApiError::Unavailable)?;
    if let Some(existing) = sqlx::query(
        "SELECT n.id, n.status, n.created_at, k.payload_sha256 FROM idempotency_keys k JOIN notifications n ON n.id = k.notification_id WHERE k.caller_id = $1 AND k.idempotency_key = $2",
    )
    .bind(caller_id)
    .bind(idem_key)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| ApiError::Unavailable)?
    {
        let existing_hash: String = existing.get("payload_sha256");
        if existing_hash != payload_hash {
            return Err(ApiError::Conflict("idempotency key already belongs to a different request".into()));
        }
        let accepted = AcceptedNotification {
            notification_id: existing.get("id"),
            status: existing.get("status"),
            created_at: existing.get("created_at"),
        };
        tx.commit().await.map_err(|_| ApiError::Unavailable)?;
        return Ok((StatusCode::ACCEPTED, Json(accepted)));
    }

    let recipient_id = request.recipient_id;
    let recipient_exists =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM recipients WHERE id = $1)")
            .bind(recipient_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|_| ApiError::Unavailable)?;
    if !recipient_exists {
        return Err(ApiError::NotFound);
    }
    let opted_in = sqlx::query_scalar::<_, bool>("SELECT COALESCE((SELECT opt_in FROM preferences WHERE recipient_id = $1 AND channel = $2), FALSE)")
        .bind(recipient_id).bind(request.channel.as_str()).fetch_one(&mut *tx).await.map_err(|_| ApiError::Unavailable)?;
    if !opted_in {
        return Err(ApiError::BadRequest(
            "recipient has opted out of this channel".into(),
        ));
    }

    let (template_id, template_version, subject, body) = if let Some(template_id) =
        request.template_id
    {
        let row = if let Some(version) = request.template_version {
            sqlx::query("SELECT version, subject, body FROM templates WHERE template_id = $1 AND version = $2 AND channel = $3")
                .bind(template_id).bind(version).bind(request.channel.as_str()).fetch_optional(&mut *tx).await.map_err(|_| ApiError::Unavailable)?
        } else {
            sqlx::query("SELECT version, subject, body FROM templates WHERE template_id = $1 AND channel = $2 ORDER BY version DESC LIMIT 1")
                .bind(template_id).bind(request.channel.as_str()).fetch_optional(&mut *tx).await.map_err(|_| ApiError::Unavailable)?
        }.ok_or(ApiError::NotFound)?;
        (
            Some(template_id),
            Some(row.get::<i32, _>("version")),
            row.get::<String, _>("subject"),
            row.get::<String, _>("body"),
        )
    } else {
        (
            None,
            None,
            request.subject.clone().unwrap_or_default(),
            request.body.clone().unwrap_or_default(),
        )
    };

    let id = Uuid::new_v4();
    let status = if request.scheduled_at.is_some() {
        "scheduled"
    } else {
        "accepted"
    };
    let created_at = Utc::now();
    let payload = json!({
        "subject": subject,
        "body": body,
        "variables": request.variables,
        "simulation": request.simulation,
    });
    sqlx::query("INSERT INTO notifications (id, caller_id, recipient_id, channel, status, scheduled_at, template_id, template_version, payload, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$10)")
        .bind(id).bind(caller_id).bind(recipient_id).bind(request.channel.as_str()).bind(status)
        .bind(request.scheduled_at).bind(template_id).bind(template_version).bind(payload).bind(created_at)
        .execute(&mut *tx).await.map_err(|_| ApiError::Unavailable)?;
    sqlx::query("INSERT INTO idempotency_keys (caller_id, idempotency_key, payload_sha256, notification_id) VALUES ($1,$2,$3,$4)")
        .bind(caller_id).bind(idem_key).bind(payload_hash).bind(id).execute(&mut *tx).await.map_err(|_| ApiError::Unavailable)?;
    sqlx::query("INSERT INTO outbox (notification_id, dispatch_number, channel, available_at) VALUES ($1,1,$2,COALESCE($3, now()))")
        .bind(id).bind(request.channel.as_str()).bind(request.scheduled_at).execute(&mut *tx).await.map_err(|_| ApiError::Unavailable)?;
    tx.commit().await.map_err(|_| ApiError::Unavailable)?;
    Ok((
        StatusCode::ACCEPTED,
        Json(AcceptedNotification {
            notification_id: id,
            status: status.into(),
            created_at,
        }),
    ))
}

#[derive(Deserialize)]
struct ListQuery {
    recipient_id: Option<Uuid>,
    status: Option<String>,
    limit: Option<i64>,
}

#[derive(Serialize, sqlx::FromRow)]
struct NotificationSummary {
    id: Uuid,
    recipient_id: Uuid,
    channel: String,
    status: String,
    attempts: i32,
    scheduled_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    last_error: Option<String>,
}

async fn list_notifications(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> Result<Json<Value>, ApiError> {
    let caller = authenticate(&headers, &state).await?;
    let limit = query.limit.unwrap_or(25).clamp(1, 100);
    let rows = sqlx::query_as::<_, NotificationSummary>("SELECT id, recipient_id, channel, status, attempts, scheduled_at, created_at, updated_at, last_error FROM notifications WHERE caller_id = $1 AND ($2::uuid IS NULL OR recipient_id = $2) AND ($3::text IS NULL OR status = $3) ORDER BY created_at DESC LIMIT $4")
        .bind(caller).bind(query.recipient_id).bind(query.status).bind(limit).fetch_all(&state.pool).await.map_err(|_| ApiError::Unavailable)?;
    Ok(Json(json!({"items":rows})))
}

async fn get_notification(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, ApiError> {
    let caller = authenticate(&headers, &state).await?;
    let row = sqlx::query_as::<_, NotificationSummary>("SELECT id, recipient_id, channel, status, attempts, scheduled_at, created_at, updated_at, last_error FROM notifications WHERE id = $1 AND caller_id = $2")
        .bind(id).bind(caller).fetch_optional(&state.pool).await.map_err(|_| ApiError::Unavailable)?.ok_or(ApiError::NotFound)?;
    let attempts = sqlx::query("SELECT attempt_number, outcome, error_code, started_at, completed_at FROM delivery_attempts WHERE notification_id = $1 ORDER BY attempt_number")
        .bind(id).fetch_all(&state.pool).await.map_err(|_| ApiError::Unavailable)?;
    let attempt_values: Vec<Value> = attempts
        .into_iter()
        .map(|attempt| {
            json!({
                "attempt_number": attempt.get::<i32,_>("attempt_number"),
                "outcome": attempt.get::<String,_>("outcome"),
                "error_code": attempt.get::<Option<String>,_>("error_code"),
                "started_at": attempt.get::<DateTime<Utc>,_>("started_at"),
                "completed_at": attempt.get::<Option<DateTime<Utc>>,_>("completed_at"),
            })
        })
        .collect();
    Ok(Json(json!({"notification":row,"attempts":attempt_values})))
}

#[derive(Serialize)]
struct PreferenceView {
    recipient_id: Uuid,
    channel: String,
    opt_in: bool,
}

async fn get_preference(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((recipient_id, channel)): Path<(Uuid, String)>,
) -> Result<Json<PreferenceView>, ApiError> {
    authenticate(&headers, &state).await?;
    let channel =
        Channel::from_str(&channel).map_err(|error| ApiError::BadRequest(error.to_string()))?;
    let exists =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM recipients WHERE id = $1)")
            .bind(recipient_id)
            .fetch_one(&state.pool)
            .await
            .map_err(|_| ApiError::Unavailable)?;
    if !exists {
        return Err(ApiError::NotFound);
    }
    let opt_in = sqlx::query_scalar::<_, bool>("SELECT COALESCE((SELECT opt_in FROM preferences WHERE recipient_id = $1 AND channel = $2), FALSE)")
        .bind(recipient_id).bind(channel.as_str()).fetch_one(&state.pool).await.map_err(|_| ApiError::Unavailable)?;
    Ok(Json(PreferenceView {
        recipient_id,
        channel: channel.as_str().into(),
        opt_in,
    }))
}

#[derive(Deserialize)]
struct PreferenceRequest {
    opt_in: bool,
}

async fn put_preference(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((recipient_id, channel)): Path<(Uuid, String)>,
    Json(request): Json<PreferenceRequest>,
) -> Result<Json<PreferenceView>, ApiError> {
    authenticate(&headers, &state).await?;
    let channel =
        Channel::from_str(&channel).map_err(|error| ApiError::BadRequest(error.to_string()))?;
    let recipient_exists =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM recipients WHERE id = $1)")
            .bind(recipient_id)
            .fetch_one(&state.pool)
            .await
            .map_err(|_| ApiError::Unavailable)?;
    if !recipient_exists {
        return Err(ApiError::NotFound);
    }
    let result = sqlx::query("INSERT INTO preferences (recipient_id, channel, opt_in, updated_at) VALUES ($1,$2,$3,now()) ON CONFLICT (recipient_id, channel) DO UPDATE SET opt_in = EXCLUDED.opt_in, updated_at = now()")
        .bind(recipient_id).bind(channel.as_str()).bind(request.opt_in).execute(&state.pool).await.map_err(|_| ApiError::Unavailable)?;
    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }
    Ok(Json(PreferenceView {
        recipient_id,
        channel: channel.as_str().into(),
        opt_in: request.opt_in,
    }))
}

#[derive(Serialize, sqlx::FromRow)]
struct RecipientView {
    id: Uuid,
    display_name: String,
    email_masked: Option<String>,
    phone_masked: Option<String>,
}

#[derive(Deserialize)]
struct RecipientRequest {
    display_name: String,
    email: Option<String>,
    phone_number: Option<String>,
}

async fn update_recipient(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(recipient_id): Path<Uuid>,
    Json(request): Json<RecipientRequest>,
) -> Result<Json<RecipientView>, ApiError> {
    authenticate(&headers, &state).await?;
    if request.display_name.trim().is_empty() || request.display_name.len() > 120 {
        return Err(ApiError::BadRequest(
            "display_name must be 1-120 characters".into(),
        ));
    }
    if request
        .email
        .as_ref()
        .is_some_and(|value| value.len() > 320 || !value.contains('@'))
    {
        return Err(ApiError::BadRequest(
            "email is not a supported address".into(),
        ));
    }
    if request
        .phone_number
        .as_ref()
        .is_some_and(|value| value.len() > 32)
    {
        return Err(ApiError::BadRequest(
            "phone_number exceeds 32 characters".into(),
        ));
    }
    let row = sqlx::query("UPDATE recipients SET display_name = $2, email = $3, phone_number = $4, updated_at = now() WHERE id = $1 RETURNING id, display_name, email, phone_number")
        .bind(recipient_id).bind(request.display_name).bind(request.email).bind(request.phone_number)
        .fetch_optional(&state.pool).await.map_err(|_| ApiError::Unavailable)?.ok_or(ApiError::NotFound)?;
    let email: Option<String> = row.get("email");
    let phone: Option<String> = row.get("phone_number");
    Ok(Json(RecipientView {
        id: row.get("id"),
        display_name: row.get("display_name"),
        email_masked: email.map(|value| mask_destination(&value)),
        phone_masked: phone.map(|value| mask_destination(&value)),
    }))
}

async fn list_recipients(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    authenticate(&headers, &state).await?;
    let rows = sqlx::query(
        "SELECT id, display_name, email, phone_number FROM recipients ORDER BY display_name",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| ApiError::Unavailable)?;
    let recipients: Vec<RecipientView> = rows
        .into_iter()
        .map(|row| {
            let email: Option<String> = row.get("email");
            let phone: Option<String> = row.get("phone_number");
            RecipientView {
                id: row.get("id"),
                display_name: row.get("display_name"),
                email_masked: email.map(|value| mask_destination(&value)),
                phone_masked: phone.map(|value| mask_destination(&value)),
            }
        })
        .collect();
    Ok(Json(json!({"items":recipients})))
}

fn mask_destination(value: &str) -> String {
    if let Some((name, domain)) = value.split_once('@') {
        let visible = name.chars().next().unwrap_or('*');
        format!("{}***@{}", visible, domain)
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

#[derive(Deserialize)]
struct DeviceRequest {
    platform: String,
    device_token: String,
}

async fn register_device(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(recipient_id): Path<Uuid>,
    Json(request): Json<DeviceRequest>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    authenticate(&headers, &state).await?;
    if !matches!(request.platform.as_str(), "ios" | "android")
        || request.device_token.trim().is_empty()
        || request.device_token.len() > 512
    {
        return Err(ApiError::BadRequest(
            "platform must be ios or android and device_token must be 1-512 characters".into(),
        ));
    }
    let row = sqlx::query("INSERT INTO devices (recipient_id, platform, device_token) SELECT id,$2,$3 FROM recipients WHERE id = $1 ON CONFLICT (platform, device_token) DO UPDATE SET last_logged_in_at = now() RETURNING id")
        .bind(recipient_id).bind(request.platform).bind(request.device_token).fetch_optional(&state.pool).await.map_err(|_| ApiError::Unavailable)?.ok_or(ApiError::NotFound)?;
    Ok((
        StatusCode::CREATED,
        Json(json!({"device_id":row.get::<Uuid,_>("id"),"registered":true})),
    ))
}

#[derive(Serialize, sqlx::FromRow)]
struct TemplateView {
    template_id: Uuid,
    version: i32,
    name: String,
    channel: String,
    subject: String,
    body: String,
}

async fn list_templates(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    authenticate(&headers, &state).await?;
    let rows = sqlx::query_as::<_, TemplateView>("SELECT DISTINCT ON (template_id) template_id, version, name, channel, subject, body FROM templates ORDER BY template_id, version DESC")
        .fetch_all(&state.pool).await.map_err(|_| ApiError::Unavailable)?;
    Ok(Json(json!({"items":rows})))
}

#[derive(Deserialize)]
struct TemplateRequest {
    id: Option<Uuid>,
    name: String,
    channel: Channel,
    subject: String,
    body: String,
}

async fn create_template(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<TemplateRequest>,
) -> Result<(StatusCode, Json<TemplateView>), ApiError> {
    authenticate(&headers, &state).await?;
    if request.name.trim().is_empty()
        || request.name.len() > 120
        || request.body.trim().is_empty()
        || request.body.len() > 10_000
        || request.subject.len() > 200
    {
        return Err(ApiError::BadRequest(
            "template name, subject, or body exceeds supported constraints".into(),
        ));
    }
    let id = request.id.unwrap_or_else(Uuid::new_v4);
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|_| ApiError::Unavailable)?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1), hashtext($2))")
        .bind(id.to_string())
        .bind("template-version")
        .execute(&mut *tx)
        .await
        .map_err(|_| ApiError::Unavailable)?;
    let max_version = sqlx::query_scalar::<_, Option<i32>>(
        "SELECT MAX(version) FROM templates WHERE template_id = $1",
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|_| ApiError::Unavailable)?;
    let version = max_version.unwrap_or(0) + 1;
    sqlx::query("INSERT INTO templates (template_id, version, name, channel, subject, body) VALUES ($1,$2,$3,$4,$5,$6)")
        .bind(id).bind(version).bind(&request.name).bind(request.channel.as_str()).bind(&request.subject).bind(&request.body).execute(&mut *tx).await.map_err(|_| ApiError::Unavailable)?;
    tx.commit().await.map_err(|_| ApiError::Unavailable)?;
    Ok((
        StatusCode::CREATED,
        Json(TemplateView {
            template_id: id,
            version,
            name: request.name,
            channel: request.channel.as_str().into(),
            subject: request.subject,
            body: request.body,
        }),
    ))
}

pub async fn connect_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(15)
        .connect(database_url)
        .await
}

pub async fn seed_caller(pool: &PgPool, api_key: &str) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO api_callers (id, name, api_key_sha256) VALUES ($1,'local synthetic caller',$2) ON CONFLICT (api_key_sha256) DO UPDATE SET enabled = TRUE")
        .bind(Uuid::parse_str(DEMO_CALLER_ID).expect("valid fixture UUID"))
        .bind(key_hash(api_key))
        .execute(pool).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use sqlx::postgres::PgConnectOptions;

    #[test]
    fn masks_email_and_phone_destinations() {
        assert_eq!(mask_destination("alex@example.test"), "a***@example.test");
        assert_eq!(mask_destination("+15555550100"), "***00");
    }

    #[test]
    fn caller_key_hash_is_one_way_and_deterministic() {
        assert_eq!(key_hash("local-dev-api-key"), key_hash("local-dev-api-key"));
        assert_ne!(key_hash("local-dev-api-key"), "local-dev-api-key");
    }

    #[tokio::test]
    async fn rate_limit_is_caller_scoped_and_expires_after_one_minute() {
        let pool = PgPoolOptions::new().connect_lazy_with(
            PgConnectOptions::new()
                .host("127.0.0.1")
                .port(5432)
                .username("notification-test")
                .database("unused"),
        );
        let state = AppState::new(pool, true);
        let caller = Uuid::new_v4();
        let other_caller = Uuid::new_v4();
        let now = Instant::now();

        for _ in 0..120 {
            assert!(state.allow_request_at(caller, now));
        }
        assert!(!state.allow_request_at(caller, now));
        assert!(state.allow_request_at(other_caller, now));
        assert!(state.allow_request_at(caller, now + std::time::Duration::from_secs(60)));
    }

    #[tokio::test]
    async fn api_errors_map_to_safe_http_status_and_messages() {
        let cases = [
            (
                ApiError::Unauthorized,
                StatusCode::UNAUTHORIZED,
                "unauthorized",
            ),
            (
                ApiError::RateLimited,
                StatusCode::TOO_MANY_REQUESTS,
                "rate limit exceeded",
            ),
            (ApiError::NotFound, StatusCode::NOT_FOUND, "not found"),
            (
                ApiError::BadRequest("invalid request".into()),
                StatusCode::BAD_REQUEST,
                "invalid request",
            ),
            (
                ApiError::Conflict("duplicate key".into()),
                StatusCode::CONFLICT,
                "duplicate key",
            ),
            (
                ApiError::Unavailable,
                StatusCode::SERVICE_UNAVAILABLE,
                "dependency unavailable",
            ),
            (
                ApiError::Internal,
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal server error",
            ),
        ];

        for (error, expected_status, expected_message) in cases {
            let response = error.into_response();
            assert_eq!(response.status(), expected_status);
            let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let payload: Value = serde_json::from_slice(&body).unwrap();
            assert_eq!(payload["error"], expected_message);
        }
    }
}
