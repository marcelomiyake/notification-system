use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::str::FromStr;
use thiserror::Error;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Channel {
    IosPush,
    AndroidPush,
    Sms,
    Email,
}

impl Channel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::IosPush => "ios_push",
            Self::AndroidPush => "android_push",
            Self::Sms => "sms",
            Self::Email => "email",
        }
    }
}

impl FromStr for Channel {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "ios_push" => Ok(Self::IosPush),
            "android_push" => Ok(Self::AndroidPush),
            "sms" => Ok(Self::Sms),
            "email" => Ok(Self::Email),
            _ => Err(DomainError::InvalidChannel),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SimulationBehavior {
    Success,
    TransientFailure,
    PermanentFailure,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NotificationRequest {
    pub recipient_id: uuid::Uuid,
    pub channel: Channel,
    pub template_id: Option<uuid::Uuid>,
    pub template_version: Option<i32>,
    #[serde(default = "empty_variables")]
    pub variables: Value,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub simulation: Option<SimulationBehavior>,
}

fn empty_variables() -> Value {
    serde_json::json!({})
}

impl NotificationRequest {
    pub fn validate(
        &self,
        now: DateTime<Utc>,
        simulation_enabled: bool,
    ) -> Result<(), DomainError> {
        let has_template = self.template_id.is_some();
        let has_body = self
            .body
            .as_ref()
            .is_some_and(|body| !body.trim().is_empty());
        if has_template == has_body {
            return Err(DomainError::ContentChoice);
        }
        if self.template_version.is_some() && !has_template {
            return Err(DomainError::TemplateVersionWithoutTemplate);
        }
        if self.scheduled_at.is_some_and(|scheduled| scheduled <= now) {
            return Err(DomainError::ScheduleMustBeFuture);
        }
        if self.body.as_ref().is_some_and(|body| body.len() > 10_000)
            || self
                .subject
                .as_ref()
                .is_some_and(|subject| subject.len() > 200)
        {
            return Err(DomainError::ContentTooLong);
        }
        if self.simulation.is_some() && !simulation_enabled {
            return Err(DomainError::SimulationDisabled);
        }
        if !self.variables.is_object() {
            return Err(DomainError::VariablesMustBeObject);
        }
        Ok(())
    }

    pub fn payload_hash(&self) -> Result<String, serde_json::Error> {
        let bytes = serde_json::to_vec(self)?;
        Ok(hex::encode(Sha256::digest(bytes)))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeliveryErrorKind {
    Transient,
    Permanent,
}

pub const MAX_ATTEMPTS: i32 = 3;

pub fn retry_delay_seconds(attempt_number: i32) -> u64 {
    2_u64
        .saturating_pow(attempt_number.clamp(1, 5) as u32)
        .min(30)
}

pub fn should_retry(kind: DeliveryErrorKind, attempt_number: i32) -> bool {
    kind == DeliveryErrorKind::Transient && attempt_number < MAX_ATTEMPTS
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotificationStatus {
    Accepted,
    Scheduled,
    Queued,
    Processing,
    Retrying,
    Sent,
    Suppressed,
    Failed,
}

impl NotificationStatus {
    pub fn can_transition_to(self, next: Self) -> bool {
        use NotificationStatus::*;
        matches!(
            (self, next),
            (Accepted, Queued | Processing | Suppressed | Failed)
                | (Scheduled, Queued | Suppressed | Failed)
                | (Queued, Processing | Suppressed | Failed)
                | (Processing, Sent | Retrying | Suppressed | Failed)
                | (Retrying, Queued | Processing | Suppressed | Failed)
        )
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DomainError {
    #[error("unsupported channel")]
    InvalidChannel,
    #[error("provide exactly one of template_id or body")]
    ContentChoice,
    #[error("template_version requires template_id")]
    TemplateVersionWithoutTemplate,
    #[error("scheduled_at must be in the future")]
    ScheduleMustBeFuture,
    #[error("content exceeds supported size")]
    ContentTooLong,
    #[error("simulation controls are disabled")]
    SimulationDisabled,
    #[error("variables must be a JSON object")]
    VariablesMustBeObject,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn valid_request() -> NotificationRequest {
        NotificationRequest {
            recipient_id: uuid::Uuid::nil(),
            channel: Channel::Email,
            template_id: None,
            template_version: None,
            variables: json!({"name": "Alex"}),
            subject: Some("Hello".into()),
            body: Some("Hello {{name}}".into()),
            scheduled_at: None,
            simulation: None,
        }
    }

    #[test]
    fn requires_exactly_one_content_source() {
        let mut request = valid_request();
        request.template_id = Some(uuid::Uuid::new_v4());
        assert_eq!(
            request.validate(Utc::now(), false),
            Err(DomainError::ContentChoice)
        );
        request.template_id = None;
        request.body = None;
        assert_eq!(
            request.validate(Utc::now(), false),
            Err(DomainError::ContentChoice)
        );
        request.body = Some("inline".into());
        assert!(request.validate(Utc::now(), false).is_ok());
    }

    #[test]
    fn validates_schedule_and_simulation_controls() {
        let mut request = valid_request();
        request.template_id = Some(uuid::Uuid::new_v4());
        request.body = None;
        request.scheduled_at = Some(Utc::now() - chrono::Duration::seconds(1));
        assert_eq!(
            request.validate(Utc::now(), true),
            Err(DomainError::ScheduleMustBeFuture)
        );
        request.scheduled_at = None;
        request.simulation = Some(SimulationBehavior::TransientFailure);
        assert_eq!(
            request.validate(Utc::now(), false),
            Err(DomainError::SimulationDisabled)
        );
    }

    #[test]
    fn transient_retries_are_bounded_and_permanent_errors_do_not_retry() {
        assert!(should_retry(DeliveryErrorKind::Transient, 1));
        assert!(should_retry(DeliveryErrorKind::Transient, 2));
        assert!(!should_retry(DeliveryErrorKind::Transient, 3));
        assert!(!should_retry(DeliveryErrorKind::Permanent, 1));
        assert_eq!(retry_delay_seconds(1), 2);
        assert_eq!(retry_delay_seconds(4), 16);
    }

    #[test]
    fn lifecycle_rejects_terminal_state_reopening() {
        assert!(NotificationStatus::Accepted.can_transition_to(NotificationStatus::Processing));
        assert!(NotificationStatus::Processing.can_transition_to(NotificationStatus::Retrying));
        assert!(!NotificationStatus::Sent.can_transition_to(NotificationStatus::Processing));
        assert!(!NotificationStatus::Suppressed.can_transition_to(NotificationStatus::Queued));
    }

    #[test]
    fn request_hash_is_stable_for_equal_structured_payloads() {
        let request = valid_request();
        let duplicate =
            serde_json::from_value::<NotificationRequest>(serde_json::to_value(&request).unwrap())
                .unwrap();
        assert_eq!(
            request.payload_hash().unwrap(),
            duplicate.payload_hash().unwrap()
        );
    }
}
