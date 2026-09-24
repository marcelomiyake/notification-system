# notification-api

Rust/Axum HTTP API for the Notification Management bounded context. It handles the local caller API key, a per-process rate limit, recipient/device data, preferences, immutable template versions, idempotent acceptance, and status queries. PostgreSQL acceptance writes notification state, the idempotency record, and the transactional outbox row in one transaction.

> Documentation: [project index](../docs/README.md) · [repository overview](../README.md)

## Contents

- [Run locally](#run-locally)
- [Tests](#tests)
- [Image](#image)
- [Component ownership, prerequisites, and lifecycle](#component-ownership-prerequisites-and-lifecycle)
- [AI development disclaimer](#ai-development-disclaimer)

## Run locally

```sh
DATABASE_URL=postgres://notification:notification@127.0.0.1:5432/notification \
CALLER_API_KEY=local-dev-api-key cargo run
```

The service applies migrations on startup and inserts one synthetic recipient. It listens on `0.0.0.0:8080`. See [`../docs/openapi.yaml`](../docs/openapi.yaml) for the contract.

## Tests

- `cargo test --lib` — domain validation, canonical idempotency hashing, transitions, and retry classification.
- `DATABASE_URL=... cargo test --test integration` — SQLx creates disposable PostgreSQL databases for authentication, per-caller rate limits, acceptance/idempotency, status and metrics routes, masked recipient data, preferences, device registration, template versions, validation, and scheduling. The repository runner provisions PostgreSQL and RabbitMQ with `../scripts/integration-test.sh`.
- `cargo fmt --check && cargo clippy --all-targets -- -D warnings` — formatting and static checks.

Do not count integration tests as passed unless they ran against PostgreSQL. Local simulation adapters do not exist in this service.

## Image

From repository root: `docker build -f notification-api/Dockerfile -t notification-api:local .`.

## Component ownership, prerequisites, and lifecycle

- **Owner:** `notification-system` / `notification-api`.
- **API, event, and data contract owners/producers/consumers:** see the [contract catalog](../docs/contracts/README.md) for each authoritative interface.
- **Parent architecture:** [System Design](../docs/system-design.md).

### Build prerequisites

Use this component’s pinned toolchain and lockfile/wrapper. The supported versions and complete local build environment are listed in the [root README](../README.md).

### Use prerequisites

This component is used as part of the parent system. Start its required local dependencies and use the supported local access path described in the [root README](../README.md).

### Build, verify, deploy, undeploy, and use

Build, run, and verification commands for this component are documented above. There is no independent release lifecycle for this component.
The parent Helm release owns deployment and removal; follow the [chart guide](../deploy/helm/notification-system/README.md) and [root deployment lifecycle](../README.md). Uninstall removes application workloads while retained PVCs keep their data; deleting the namespace or claims purges persistent data.

[Documentation index](../docs/README.md)


## AI development disclaimer

> **AI development disclaimer:** This project was built entirely with GPT-6 Luna at Max effort as a proof of concept exploring how low-cost AI plans can be useful when paired with disciplined harness and loop engineering. This is project-owner attribution; repository contents do not independently verify runtime model metadata. Review AI-generated design and code before relying on them.
