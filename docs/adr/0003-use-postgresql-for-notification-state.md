# ADR-0003: Use PostgreSQL for Notification State

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

The API persists requests, preferences, templates, idempotency, and outbox state in PostgreSQL. The worker claims due rows and records attempts. The broker is a transport buffer; it is not the durable business record. No original database selection record was found.

The scope of this decision is PostgreSQL as the durable source of truth for notification state and the transactional outbox. The source confirms the implementation; its historical selection rationale and original option set are not recorded.

## Decision drivers

- Commit accepted notification state and its outbox event atomically.
- Enforce relational constraints and support the lifecycle queries used by API and workers.
- Coordinate multiple worker replicas without introducing another authoritative store.

## Options considered

### PostgreSQL

- **Benefits:** Relational transactions, constraints, migrations, and row-locking support the implemented acceptance and outbox flows.
- **Costs and risks:** The local chart uses a single database instance; availability and write capacity are bounded by that deployment.

### Another relational database

- **Benefits:** Could provide similar transactional and relational guarantees.
- **Costs and risks:** Would require a driver, migration, operational, and test changes with no repository evidence that PostgreSQL is insufficient.

### Document or key-value database

- **Benefits:** Could fit particular high-volume access patterns.
- **Costs and risks:** The current workflow spans request and outbox records; preserving atomicity and worker claims would require a redesigned consistency protocol.

## Decision outcome

Keep PostgreSQL as the authoritative store for notification lifecycle state. RabbitMQ remains a recoverable delivery transport, not a replacement for committed database state.

## Consequences

### Positive

- Accepted requests and outbox records share a transaction boundary.
- Relational constraints and row locking express idempotency and worker-claim invariants in the current design.

### Negative and risks

- A database outage stops acceptance and durable dispatch; the local single instance is not highly available.
- Schema migrations and SQL remain part of each service’s operational contract.

## Evidence and realization

- [0001_notification_system.sql](../../database/migrations/0001_notification_system.sql)
- [values.yaml](../../deploy/helm/notification-system/values.yaml)
- [database-model.md](../database-model.md)
- [system-design.md](../system-design.md)

## Review triggers

- Reassess after measured database saturation, a change in durability requirements, or a need for independent data scaling.
- If replacing or splitting the database, document transaction, outbox, idempotency, and worker-claim semantics in a superseding ADR.

## References

- [notification-system README](../../README.md)
- [System Design](../system-design.md)
- [ADR practices](https://adr.github.io/ad-practices/)
- [ADR template guidance](https://adr.github.io/adr-templates/)
