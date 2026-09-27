# SonarQube Cloud verification

`notification-api` and `notification-worker` each have a dedicated SonarQube Cloud project. The [GitHub Actions workflow](https://github.com/marcelomiyake/notification-system/actions/workflows/sonarcloud-microservices.yml) runs one test, coverage, and analysis job per service on pushes to `main`; `workflow_dispatch` supports a manual rerun. Organization-level auto-import of new GitHub repositories is disabled, so project analysis is managed by this workflow.

Each matrix job runs `cargo llvm-cov --lcov --output-path target/coverage/lcov.info -- --test-threads=1` from the service directory, then imports the report using `cargo sonar-scanner`. The repository secret is named `SONAR_TOKEN`. GitHub Actions provisions PostgreSQL and RabbitMQ for the integration tests. The service entrypoints `src/main.rs` are excluded from the coverage denominator; no other coverage exclusions are configured. No local Sonar scan script is used.

## Projects and coverage

Coverage below is SonarCloud's overall line coverage for `main`, not new-code or local coverage. The baseline is the latest Cloud result before the coverage-test updates; the current column is the latest result after them. Values are from the 2026-09-27 snapshot.

| Microservice | SonarCloud project | Before | Current | Change |
| --- | --- | ---: | ---: | ---: |
| `notification-api` | [project](https://sonarcloud.io/project/overview?id=marcelomiyake_notification-system_notification-api) | 96.8% | 98.8% | +2.0 pp |
| `notification-worker` | [project](https://sonarcloud.io/project/overview?id=marcelomiyake_notification-system_notification-worker) | 81.5% | 82.6% | +1.1 pp |

Both projects have a passing Quality Gate, zero open or confirmed issues, zero hotspots awaiting review, zero bugs, zero vulnerabilities, zero code smells, and 0.0% duplicated lines.

## Verification

Confirm the latest workflow completed for the pushed commit and that each project's `main` analysis matches that revision. Review coverage, active issues, security hotspots, duplication, and the Quality Gate. Project links above open the live dashboards; the workflow link shows its run history. Local tests and coverage reports do not replace a completed Cloud analysis.
