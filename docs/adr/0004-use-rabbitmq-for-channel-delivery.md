# ADR-0004: Use RabbitMQ for Channel Delivery

- **Status:** Implemented (retrospective)
- **Recorded:** 2026-09-25
- **Original decision date:** Unknown from repository evidence
- **Decision owner:** Project owner
- **Confirmation:** Current implementation is documented at the project owner’s request; historical team approval is not recorded.

> This record documents the technology in the current implementation. The options and rationale below are a retrospective comparison, not a claim that the original project formally evaluated them.

## Contents

- [Context and problem statement](#context-and-problem-statement)
- [Decision drivers](#decision-drivers)
- [Options considered](#options-considered)
- [Decision outcome](#decision-outcome)
- [Consequences](#consequences)
- [Evidence and realization](#evidence-and-realization)
- [Review triggers](#review-triggers)
- [References](#references)

## Context and problem statement

The API writes durable outbox rows in PostgreSQL. The worker publishes due records to channel-specific RabbitMQ queues and uses broker confirmation before marking publication complete. Database state remains the recovery source. No original broker selection record was found.

The scope of this decision is RabbitMQ queues as asynchronous transport for notification delivery work. The source confirms the implementation; its historical selection rationale and original option set are not recorded.

## Decision drivers

- Separate request acceptance from slower provider-adapter work.
- Isolate delivery channels and buffer bursts.
- Retain a durable replay path when publishing or worker processing is interrupted.

## Options considered

### RabbitMQ

- **Benefits:** The implemented exchanges and queues provide asynchronous routing and acknowledgments for worker delivery.
- **Costs and risks:** Adds a stateful broker, queue operations, credentials, and another failure mode.

### Poll PostgreSQL without a broker

- **Benefits:** Fewer infrastructure components and a direct durable work source.
- **Costs and risks:** Would couple delivery scheduling and concurrency to database polling and would not provide the current queue isolation.

### Kafka or another log broker

- **Benefits:** Could support retained event streams and multiple independent readers.
- **Costs and risks:** The current workflow is task delivery with one primary worker role; a log platform would add operational complexity without a demonstrated need.

## Decision outcome

Retain RabbitMQ for channel-specific delivery transport while PostgreSQL outbox rows remain authoritative and replayable.

## Consequences

### Positive

- Request handling does not wait for channel delivery completion.
- Queues isolate channel work and provide explicit publisher/consumer acknowledgment points.

### Negative and risks

- Broker messages can be redelivered; handlers must preserve idempotent state transitions.
- RabbitMQ availability and local resource use are additional operational concerns.

## Evidence and realization

- [Cargo.toml](../../notification-worker/Cargo.toml)
- [rabbitmq.yaml](../../deploy/helm/notification-system/templates/postgres.yaml)
- [values.yaml](../../deploy/helm/notification-system/values.yaml)
- [system-design.md](../system-design.md)

## Review triggers

- Reconsider if measured workload requires a different routing, retention, or fan-out model.
- Keep the transactional outbox and replay behavior explicit if the broker or delivery contract changes.

## References

- [notification-system README](../../README.md)
- [System Design](../system-design.md)
- [ADR practices](https://adr.github.io/ad-practices/)
- [ADR template guidance](https://adr.github.io/adr-templates/)
