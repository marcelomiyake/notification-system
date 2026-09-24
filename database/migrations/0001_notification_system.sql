CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE IF NOT EXISTS api_callers (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    api_key_sha256 TEXT NOT NULL UNIQUE,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS recipients (
    id UUID PRIMARY KEY,
    display_name TEXT NOT NULL,
    email TEXT,
    phone_number TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS devices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recipient_id UUID NOT NULL REFERENCES recipients(id) ON DELETE CASCADE,
    platform TEXT NOT NULL CHECK (platform IN ('ios', 'android')),
    device_token TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_logged_in_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (platform, device_token)
);

CREATE TABLE IF NOT EXISTS preferences (
    recipient_id UUID NOT NULL REFERENCES recipients(id) ON DELETE CASCADE,
    channel TEXT NOT NULL CHECK (channel IN ('ios_push', 'android_push', 'sms', 'email')),
    opt_in BOOLEAN NOT NULL DEFAULT FALSE,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (recipient_id, channel)
);

CREATE TABLE IF NOT EXISTS templates (
    template_id UUID NOT NULL,
    version INTEGER NOT NULL CHECK (version > 0),
    name TEXT NOT NULL,
    channel TEXT NOT NULL CHECK (channel IN ('ios_push', 'android_push', 'sms', 'email')),
    subject TEXT NOT NULL DEFAULT '',
    body TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (template_id, version)
);

CREATE TABLE IF NOT EXISTS notifications (
    id UUID PRIMARY KEY,
    caller_id UUID NOT NULL REFERENCES api_callers(id),
    recipient_id UUID NOT NULL REFERENCES recipients(id),
    channel TEXT NOT NULL CHECK (channel IN ('ios_push', 'android_push', 'sms', 'email')),
    status TEXT NOT NULL CHECK (status IN ('accepted', 'scheduled', 'queued', 'processing', 'retrying', 'sent', 'suppressed', 'failed')),
    scheduled_at TIMESTAMPTZ,
    template_id UUID,
    template_version INTEGER,
    payload JSONB NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT notifications_template_fk FOREIGN KEY (template_id, template_version)
        REFERENCES templates(template_id, version)
);
CREATE INDEX IF NOT EXISTS notifications_created_idx ON notifications(created_at DESC);
CREATE INDEX IF NOT EXISTS notifications_schedule_idx ON notifications(scheduled_at) WHERE status = 'scheduled';
CREATE INDEX IF NOT EXISTS notifications_recipient_idx ON notifications(recipient_id, created_at DESC);

CREATE TABLE IF NOT EXISTS idempotency_keys (
    caller_id UUID NOT NULL REFERENCES api_callers(id),
    idempotency_key TEXT NOT NULL CHECK (length(idempotency_key) BETWEEN 1 AND 128),
    payload_sha256 TEXT NOT NULL,
    notification_id UUID NOT NULL REFERENCES notifications(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (caller_id, idempotency_key)
);

CREATE TABLE IF NOT EXISTS outbox (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    notification_id UUID NOT NULL REFERENCES notifications(id) ON DELETE CASCADE,
    dispatch_number INTEGER NOT NULL CHECK (dispatch_number > 0),
    channel TEXT NOT NULL CHECK (channel IN ('ios_push', 'android_push', 'sms', 'email')),
    available_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    published_at TIMESTAMPTZ,
    leased_until TIMESTAMPTZ,
    publish_attempts INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (notification_id, dispatch_number)
);
CREATE INDEX IF NOT EXISTS outbox_dispatch_idx ON outbox(available_at, created_at) WHERE published_at IS NULL;

CREATE TABLE IF NOT EXISTS delivery_attempts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    notification_id UUID NOT NULL REFERENCES notifications(id) ON DELETE CASCADE,
    attempt_number INTEGER NOT NULL CHECK (attempt_number > 0),
    outcome TEXT NOT NULL CHECK (outcome IN ('started', 'accepted', 'transient_error', 'permanent_error', 'suppressed')),
    error_code TEXT,
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ,
    UNIQUE (notification_id, attempt_number)
);

CREATE TABLE IF NOT EXISTS recorded_deliveries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    notification_id UUID NOT NULL UNIQUE REFERENCES notifications(id) ON DELETE CASCADE,
    channel TEXT NOT NULL CHECK (channel IN ('ios_push', 'android_push', 'sms', 'email')),
    masked_destination TEXT NOT NULL,
    rendered_subject TEXT NOT NULL DEFAULT '',
    rendered_body TEXT NOT NULL,
    accepted_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Synthetic local fixture; never use real recipient data in this database.
INSERT INTO recipients (id, display_name, email, phone_number)
VALUES ('11111111-1111-4111-8111-111111111111', 'Alex Example', 'alex@example.test', '+15555550100')
ON CONFLICT (id) DO NOTHING;

INSERT INTO preferences (recipient_id, channel, opt_in)
SELECT '11111111-1111-4111-8111-111111111111', channel, TRUE
FROM unnest(ARRAY['ios_push', 'android_push', 'sms', 'email']) AS channel
ON CONFLICT (recipient_id, channel) DO NOTHING;

INSERT INTO devices (recipient_id, platform, device_token)
VALUES
    ('11111111-1111-4111-8111-111111111111', 'ios', 'local-ios-device-token'),
    ('11111111-1111-4111-8111-111111111111', 'android', 'local-android-device-token')
ON CONFLICT (platform, device_token) DO NOTHING;

INSERT INTO templates (template_id, version, name, channel, subject, body)
VALUES
    ('22222222-2222-4222-8222-222222222221', 1, 'Order update', 'ios_push', 'Order update', 'Your order {{order_id}} is ready.'),
    ('22222222-2222-4222-8222-222222222222', 1, 'Order update', 'android_push', 'Order update', 'Your order {{order_id}} is ready.'),
    ('22222222-2222-4222-8222-222222222223', 1, 'Order update', 'sms', '', 'Your order {{order_id}} is ready.'),
    ('22222222-2222-4222-8222-222222222224', 1, 'Order update', 'email', 'Your order {{order_id}} is ready', '<p>Your order {{order_id}} is ready.</p>')
ON CONFLICT (template_id, version) DO NOTHING;
