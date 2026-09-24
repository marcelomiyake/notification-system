# SonarQube verification

The three service projects are `notification-api`, `notification-worker`, and `notification-frontend`. The local SonarQube Community Build is 26.9.0.129388 at `http://127.0.0.1:9000`. The latest analyses ran with `.sonar/analyze.sh` on 2026-09-25 against `HEAD` `41d9dac121e157671a82ddce1b3f194edd41269d`; the worktree was dirty and no commit was created.

> Project documentation index: [Documentation index](../README.md)

## Latest analysis

All three analysis tasks uploaded successfully. Coverage is the project-wide measure. SonarQube reports all three server quality gates as `OK` and zero open issues.

| Project | Coverage | Line coverage | Branch coverage | Duplication | Open issues | Bugs / vulnerabilities / code smells | Security hotspots | Server gate |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| `notification-api` | 93.9% | 93.9% (667/710) | Not reported | 0.0% | 0 | 0 / 0 / 0 | 0 | OK |
| `notification-worker` | 80.6% | 80.6% (485/602) | Not reported | 0.0% | 0 | 0 / 0 / 0 | 0 | OK |
| `notification-frontend` | 86.8% | 88.9% (209/235) | 82.5% (99/120) | 0.0% | 0 | 0 / 0 / 0 | 0 | OK |

Analysis task IDs for the 2026-09-25 run:

- API: `136694e0-c262-4b71-a943-c8f82f7afff8`
- Worker: `f3767d92-15e2-4734-a645-e1b990b9cb89`
- Frontend: `1a5dce86-9d51-40b8-b184-3a74292d890c`

The prior report showed API 50.9%, worker 78.1%, and frontend 85.5%. The current report improves them by 43.0, 2.5, and 1.3 percentage points respectively. API/worker handlers now have PostgreSQL-backed route and failure-path tests; frontend tests cover the API wrapper and accessible success state.

The previous scan had three active frontend findings: an unused `Preference` import, the non-native `role="status"` success message, and an unnamed operator `<aside>`. They are fixed and SonarQube marks all three `FIXED`/`CLOSED`. The latest active-issue query returns zero for every project. There are no security hotspots requiring review.

The intended repository gate is recorded in [`.sonar/quality-gate.json`](../../.sonar/quality-gate.json): at least 80% coverage, less than 3% duplication, zero new blocker/critical issues, and all security hotspots reviewed. `python3 .sonar/check-gate.py` exited 0 and reported `PASS` for all three projects. The current server gates also report `OK` for all three projects.

The analysis runner completed Rust Clippy, coverage-instrumented unit and integration tests against disposable PostgreSQL/RabbitMQ, frontend Vitest coverage, and all three Sonar scans. The measured tests were API 9 unit + 7 integration, worker 7 unit + 5 integration, and frontend 6 unit tests. Sonar coverage is collected from these reports; it is not inferred from successful builds. The analysis scopes are the configured component source directories; repository Markdown and files outside those scopes are not included in the code coverage measures. The frontend LCOV importer warned that it could not resolve `playwright.config.ts`; that file is outside the configured application source scope and the warning did not prevent the scan from completing.
