# notification-worker

Rust delivery worker for scheduling, transactional-outbox publication, four RabbitMQ channel queues, preference rechecks, bounded retries, dead letters, and local recording adapters. Each deployment replica claims outbox rows with PostgreSQL row locks and `SKIP LOCKED`; each channel queue has independent consumers.

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
AMQP_URL=amqp://notification:notification@127.0.0.1:5672/%2f cargo run
```

The HTTP health listener uses port 8081. Durable channel queues route `ios_push`, `android_push`, `sms`, and `email` independently. `*.dlq` queues receive permanent/exhausted messages. Broker-confirmed messages are marked published in the outbox; DB recovery will republish any ambiguous delivery, so consumer state must remain idempotent.

## Tests

- `cargo test --lib` — per-channel routing, template rendering, retry/backoff decisions, local adapter behavior, and health/metrics dependency failures.
- `DATABASE_URL=... TEST_AMQP_URL=... cargo test --test integration` — disposable PostgreSQL/RabbitMQ outbox confirmation, channel isolation, transient/permanent failures, dead letters, worker recovery, duplicate delivery, preference rechecks, and invalid/stale dispatch handling. The repository runner provisions both dependencies with `../scripts/integration-test.sh`; missing broker configuration fails the suite instead of silently skipping it.
- `cargo fmt --check && cargo clippy --all-targets -- -D warnings` — formatting and static checks.

Retries are at-least-once. A real provider could accept a message before a worker loses its response; real adapters need provider idempotency/reconciliation. The recording adapter does not reach a device, carrier, or mailbox.

## Image

From repository root: `docker build -f notification-worker/Dockerfile -t notification-worker:local .`.

## Component ownership, prerequisites, and lifecycle

- **Owner:** `notification-system` / `notification-worker`.
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
