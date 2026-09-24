# Repository agent guidance

> Human guide: [README.md](README.md) · [Documentation index](docs/README.md)

## Source of truth and scope

- Keep [`docs/system-design.md`](docs/system-design.md) canonical. Update it when requirements, behavior, ownership, risks, or verification results change; keep its DOCX companion aligned.
- Treat the ByteByteGo chapter's 16 million messages/day as a design scenario, not measured capacity. Cite measured Kind results separately.
- Keep the implementation local and synthetic. Do not add public ingress or live provider credentials.
- Keep the shared README attribution exact: project owner attribution says the work used GPT-6 Luna at Max effort as a proof of concept; repository contents do not independently verify runtime model metadata.

## Boundaries

- `notification-api` owns Notification Management: caller authentication/rate limiting, recipient/contact/device data, preferences, template versions, request validation/idempotency, and accepted notification state.
- `notification-worker` owns Delivery: scheduling, transactional outbox dispatch, channel routing, delivery attempts, retries, dead letters, and recorded outcomes.
- `notification-frontend` is an operator client. Keep provider credentials and authoritative policy out of browser code.
- PostgreSQL is the source of truth. Use the outbox for durable handoff to RabbitMQ. Each channel has an isolated queue.
- New operations that cross these boundaries need a documented API/event contract and focused tests.

## Changes and verification

- Put each service's code and tests in its service directory. Keep per-service instructions current in that service's `AGENTS.md` and `README.md`.
- Add or update unit, integration, and end-to-end coverage for changed behavior. Never use real recipient data or real provider adapters in automated tests.
- Do not report a check as passed unless it was run. Record command, revision, environment, and result in `docs/verification/` for Kind and Sonar analyses.
- Maintain separate SonarQube projects for the three applications. The intended gate is >=80% coverage, <3% duplication, no new blocker/critical issues, and reviewed hotspots; distinguish an unavailable analyzer from a passing gate.
- Use conventional commit messages conforming to Conventional Commits 1.0.0 for every commit.
- Do not commit credentials, tokens, kubeconfigs, or generated test data.

## Local deployment safety

- Target only the existing local Kind context `kind-kind`, namespace `notification-system`.
- App Deployments must have at least two replicas. PostgreSQL and RabbitMQ are one-instance local dependencies and do not provide production HA.
- Expose only the frontend using loopback-bound port-forwarding. Do not add an Ingress, LoadBalancer, or NodePort.
- Keep persistent state during verification unless the user explicitly asks to reset it.
