# Verification guide

This directory records commands, revisions, environments, actual outputs, and gaps. Keep secrets and recipient data out of notes.

> Documentation: [project index](../README.md) · [repository overview](../../README.md)

## Test layers

- Unit tests: each Rust service `cargo test --lib`; frontend `pnpm test:unit`.
- Integration: service integration tests require disposable PostgreSQL and RabbitMQ using `TEST_DATABASE_URL` and `TEST_AMQP_URL`; run `cargo test --test integration` per Rust service.
- Frontend E2E: `pnpm test:e2e` starts the Vite app and uses Playwright with synthetic API fixtures.
- Kind E2E: `scripts/kind-e2e.sh` validates ready replicas, loopback port-forward UI/API, queue-backed delivery, and terminal outcomes.
- SonarQube: use `.sonar/scan.sh`. Record analysis task IDs, analyzed git SHA, gate status, coverage, duplication, new blocker/critical issue counts, and hotspot review status in `sonarqube.md`.
- Lighthouse and SEO metadata: see [`lighthouse.md`](lighthouse.md). Keep browser/runtime scores separate from SonarQube and JEV results.

Do not infer coverage or throughput from a successful build. The 16 million/day number is only the chapter's workload scenario. Local benchmarks, if run, must include hardware, image revision, input shape, test duration, and observed results.
## Verification workflow

Run the checks from the repository root unless a command names a service directory.

```sh
cargo test --manifest-path notification-api/Cargo.toml --lib
cargo test --manifest-path notification-worker/Cargo.toml --lib
./scripts/integration-test.sh
pnpm --dir notification-frontend build
pnpm --dir notification-frontend test:unit
pnpm --dir notification-frontend test:e2e
./scripts/kind-deploy.sh
./scripts/kind-e2e.sh
```

The integration runner creates its own disposable PostgreSQL and RabbitMQ Compose project, bound to loopback-only ports, and deletes its volumes on exit. It does not use the persistent Kind database. The Playwright test without `E2E_BASE_URL` uses API fixtures; the Kind runner sets it and exercises the deployed browser/API/worker path.

SonarQube scopes are in `.sonar/`. `quality-gate.json` records the required gate: coverage at least 80%, duplication below 3%, no new blocker/critical issues, and all security hotspots reviewed. Run `python3 .sonar/check-gate.py` after analysis to enforce those thresholds against all three projects. The latest Sonar analysis and local gate both pass for all three services; see [`sonarqube.md`](sonarqube.md) for task IDs and measured coverage.

The current Kind environment is local-only. Its 16 million/day figure remains a planning scenario; report observed test behavior and cluster state separately.

Latest local deployment and full-flow results, including release revision and ready replica counts, are in [`kind-e2e.md`](kind-e2e.md).

## Document lifecycle

This file documents verification commands and evidence. It has no independent software build, deployment, or undeployment lifecycle; run only the verification procedures that apply to the stated project and environment.


## AI development disclaimer

> **AI development disclaimer:** This project was built entirely with GPT-6 Luna at Max effort as a proof of concept exploring how low-cost AI plans can be useful when paired with disciplined harness and loop engineering. This is project-owner attribution; repository contents do not independently verify runtime model metadata. Review AI-generated design and code before relying on them.
