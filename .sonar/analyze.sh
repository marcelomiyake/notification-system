#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
compose_file="$root_dir/deploy/test/docker-compose.yaml"
project="notification-system-sonar-tests"
db_url="postgres://notification_test:notification_test@127.0.0.1:15432/postgres"
amqp_url="amqp://notification_test:notification_test@127.0.0.1:5673/%2f"

docker compose -p "$project" -f "$compose_file" down --volumes --remove-orphans >/dev/null 2>&1 || true
cleanup() {
  docker compose -p "$project" -f "$compose_file" down --volumes --remove-orphans
}
trap cleanup EXIT
docker compose -p "$project" -f "$compose_file" up --detach --wait

mkdir -p "$root_dir/target"
(cd "$root_dir/notification-api" && cargo clippy --all-targets --locked --message-format=json -- -D warnings > target/sonar-clippy.json)
(cd "$root_dir/notification-worker" && cargo clippy --all-targets --locked --message-format=json -- -D warnings > target/sonar-clippy.json)
(cd "$root_dir/notification-api" && DATABASE_URL="$db_url" cargo llvm-cov --all-targets --locked --lcov --output-path target/sonar-rust-lcov.info)
(cd "$root_dir/notification-worker" && DATABASE_URL="$db_url" TEST_AMQP_URL="$amqp_url" cargo llvm-cov --all-targets --locked --lcov --output-path target/sonar-rust-lcov.info -- --test-threads=1)
for report in \
  "$root_dir/notification-api/target/sonar-rust-lcov.info" \
  "$root_dir/notification-worker/target/sonar-rust-lcov.info"; do
  cat "$report"
  printf '\n'
done > "$root_dir/target/sonar-rust-lcov.info"
python3 "$root_dir/.sonar/normalize-rust-lcov.py"

(cd "$root_dir/notification-frontend" && pnpm exec vitest run --coverage.enabled=true --coverage.reporter=lcov --coverage.reportsDirectory=coverage)
SONAR_HOST_URL="${SONAR_HOST_URL:-http://127.0.0.1:9000}" "$root_dir/.sonar/scan.sh"
