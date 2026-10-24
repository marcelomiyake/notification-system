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
- SonarQube Cloud analyzes `notification-api` and `notification-worker` as separate monorepo projects through `.github/workflows/sonarcloud-microservices.yml`. A push to `main` starts one coverage-instrumented job per service using this repository’s `SONAR_TOKEN` secret. Confirm each analyzed revision, coverage, findings, and Quality Gate in [the verification guide](docs/verification/sonarqube.md).
- Use conventional commit messages conforming to Conventional Commits 1.0.0 for every commit.
- Do not commit credentials, tokens, kubeconfigs, or generated test data.

## Local deployment safety

- Target only the existing local Kind context `kind-kind`, namespace `notification-system`.
- App Deployments must have at least two replicas. PostgreSQL and RabbitMQ are one-instance local dependencies and do not provide production HA.
- Expose only the frontend using loopback-bound port-forwarding. Do not add an Ingress, LoadBalancer, or NodePort.
- Keep persistent state during verification unless the user explicitly asks to reset it.

## Releases

`.github/workflows/release.yml` runs [release-please](https://github.com/googleapis/release-please) on every push to `main`. From the Conventional Commits merged since the last tag, it keeps one release pull request open, titled `chore: release X.Y.Z`:

- `fix`, `perf` and `revert` make a patch release, `feat` a minor release, and a breaking change (`!`) a major release. The other types release nothing on their own and stay out of the changelog.
- It writes the version to `version.txt` and adds the release to `CHANGELOG.md`.
- Merging it tags `vX.Y.Z` and publishes the GitHub release.
- GitHub runs no workflow on that pull request, because the workflow's own token opened it. The other workflows run on `main` once it merges.

Never change the version or `CHANGELOG.md` by hand. `.release-please-manifest.json` holds the last released version, and `release-please-config.json` the settings. To force a version, add a `Release-As: X.Y.Z` footer to a commit.
