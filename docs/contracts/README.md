# API and event contract catalog

This catalog identifies the owners, producers, and consumers of the Notification System's cross-component interfaces. The OpenAPI document and implementation remain authoritative.

> Documentation: [project index](../README.md) · [repository overview](../../README.md)

## Contents

- [Contracts](#contracts)
- [Consumer map](#consumer-map)
- [WebMCP assessment](#webmcp-assessment)
- [Compatibility and security](#compatibility-and-security)
- [Related documentation](#related-documentation)
- [Document lifecycle](#document-lifecycle)
- [AI development disclaimer](#ai-development-disclaimer)

## Contracts

| Contract | Type and authority | Owner | Producer | Consumers |
| --- | --- | --- | --- | --- |
| Notification Management API (`/api/v1`) | REST/JSON; [OpenAPI](../openapi.yaml) | `notification-system` / `notification-api` | `notification-system` / `notification-api` | `notification-system` / `notification-frontend`; other-repository and external callers: unknown (none are recorded in the tracked repositories). |
| Delivery message and channel queues | AMQP; versioned payload and routing guarantees in [System Design](../system-design.md#6-api-and-data-contracts) and worker/API source | `notification-system` / `notification-api` owns acceptance/outbox contract; `notification-worker` owns consumption/outcome contract | `notification-system` / `notification-api` outbox relay | `notification-system` / `notification-worker`; other-repository queue consumers: unknown (none are recorded in the tracked repositories). |
| PostgreSQL lifecycle schema | SQL migrations under [`database/migrations/`](../../database/migrations/) | `notification-system` / API and worker jointly own the explicitly shared notification lifecycle rows; each context owns its tables/fields | `notification-system` / `notification-api` and `notification-worker` according to the System Design | `notification-system` / `notification-api`, `notification-system` / `notification-worker`; not a public database interface. |

## Consumer map

- The operator console calls the REST API. It does not own authoritative notification, preference, or delivery state.
- The API publishes durable outbox work to RabbitMQ; the worker consumes it and persists delivery outcomes through the documented state transitions.
- Other-repository consumers for the notification HTTP and AMQP contracts are unknown; no such consumers are recorded in the tracked repositories. Untracked/external consumers are not asserted absent.

## WebMCP assessment

No WebMCP tool contract is implemented. The operator UI can dispatch notifications and read recipient/preference data, so these capabilities stay behind the existing API and operator workflow pending a separate security and authorization design.

## Compatibility and security

Use `/api/v1` and the versioned queue schema. PostgreSQL commit is the acceptance boundary; idempotency is caller-scoped. The worker is at-least-once and deduplicates state transitions. HTTP access requires the local API key; do not place credentials or recipient data in documentation or evidence.

## Related documentation

- [System Design](../system-design.md)
- [Project README](../../README.md)
- [Documentation index](../README.md)

## Document lifecycle

This contract catalog is maintained as Markdown and links to the implementation-owned interface definitions. It has no independent software build, deployment, or undeployment lifecycle. Review it when its linked contracts or consumers change.


## AI development disclaimer

> **AI development disclaimer:** This project was built entirely with GPT-6 Luna at Max effort as a proof of concept exploring how low-cost AI plans can be useful when paired with disciplined harness and loop engineering. This is project-owner attribution; repository contents do not independently verify runtime model metadata. Review AI-generated design and code before relying on them.
