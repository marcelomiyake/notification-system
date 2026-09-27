#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
compose_file="$root_dir/deploy/test/docker-compose.yaml"
project="notification-system-tests"
db_url="postgres://notification_test:notification_test@127.0.0.1:15432/postgres"
amqp_url="amqp://notification_test:notification_test@127.0.0.1:5673/%2f"

docker compose -p "$project" -f "$compose_file" down --volumes --remove-orphans >/dev/null 2>&1 || true
cleanup() {
  docker compose -p "$project" -f "$compose_file" down --volumes --remove-orphans
}
trap cleanup EXIT
docker compose -p "$project" -f "$compose_file" up --detach --wait

(cd "$root_dir/notification-api" && DATABASE_URL="$db_url" cargo test --test integration)
(cd "$root_dir/notification-worker" && DATABASE_URL="$db_url" TEST_AMQP_URL="$amqp_url" cargo test --test integration -- --test-threads=1)
