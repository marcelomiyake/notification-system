#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
context="$(kubectl config current-context)"
if [[ "$context" != "kind-kind" ]]; then
  printf 'Refusing local deploy: expected context kind-kind, got %s\n' "$context" >&2
  exit 1
fi
kind get clusters | grep -qx 'kind' || { printf 'Expected existing Kind cluster "kind".\n' >&2; exit 1; }
kubectl get nodes --context kind-kind

namespace=notification-system
credentials_secret=notification-system-credentials
legacy_secret=notification-system-local
credential_keys=(POSTGRES_PASSWORD RABBITMQ_DEFAULT_PASS CALLER_API_KEY)
kubectl --context kind-kind create namespace "$namespace" --dry-run=client -o yaml | kubectl --context kind-kind apply -f - >/dev/null

has_credential_keys() {
  local secret_name="$1" key encoded
  for key in "${credential_keys[@]}"; do
    encoded="$(kubectl --context kind-kind -n "$namespace" get secret "$secret_name" -o "jsonpath={.data.${key}}" 2>/dev/null || true)"
    if [[ -z "$encoded" ]]; then
      return 1
    fi
  done
}

create_credentials_secret() {
  local secret_dir="$1"
  kubectl --context kind-kind -n "$namespace" create secret generic "$credentials_secret" \
    --from-file="POSTGRES_PASSWORD=$secret_dir/POSTGRES_PASSWORD" \
    --from-file="RABBITMQ_DEFAULT_PASS=$secret_dir/RABBITMQ_DEFAULT_PASS" \
    --from-file="CALLER_API_KEY=$secret_dir/CALLER_API_KEY" \
    --dry-run=client -o yaml | kubectl --context kind-kind apply -f - >/dev/null
}

if kubectl --context kind-kind -n "$namespace" get secret "$credentials_secret" >/dev/null 2>&1; then
  if ! has_credential_keys "$credentials_secret"; then
    printf 'Secret %s/%s must contain POSTGRES_PASSWORD, RABBITMQ_DEFAULT_PASS, and CALLER_API_KEY.\n' "$namespace" "$credentials_secret" >&2
    exit 1
  fi
elif kubectl --context kind-kind -n "$namespace" get secret "$legacy_secret" >/dev/null 2>&1; then
  umask 077
  credential_dir="$(mktemp -d)"
  trap 'rm -rf "$credential_dir"' EXIT
  for key in "${credential_keys[@]}"; do
    legacy_value="$(kubectl --context kind-kind -n "$namespace" get secret "$legacy_secret" -o "jsonpath={.data.${key}}")"
    if [[ -z "$legacy_value" ]]; then
      printf 'Legacy Secret %s/%s is missing %s; restore it before upgrading.\n' "$namespace" "$legacy_secret" "$key" >&2
      exit 1
    fi
    printf '%s' "$legacy_value" | base64 --decode > "$credential_dir/$key"
  done
  create_credentials_secret "$credential_dir"
  trap - EXIT
  rm -rf "$credential_dir"
  printf 'Migrated local credentials to Secret %s/%s.\n' "$namespace" "$credentials_secret"
elif kubectl --context kind-kind -n "$namespace" get pvc -o name | grep -q .; then
  printf 'Persistent data exists in namespace %s but Secret %s/%s is missing. Restore the original database and RabbitMQ credentials before upgrading.\n' "$namespace" "$namespace" "$credentials_secret" >&2
  exit 1
else
  umask 077
  credential_dir="$(mktemp -d)"
  trap 'rm -rf "$credential_dir"' EXIT
  for key in "${credential_keys[@]}"; do
    openssl rand -hex 32 > "$credential_dir/$key"
  done
  create_credentials_secret "$credential_dir"
  trap - EXIT
  rm -rf "$credential_dir"
  printf 'Generated unique local credentials in Secret %s/%s.\n' "$namespace" "$credentials_secret"
fi

docker build -f "$root_dir/notification-api/Dockerfile" -t notification-api:local "$root_dir"
docker build -f "$root_dir/notification-worker/Dockerfile" -t notification-worker:local "$root_dir"
docker build -f "$root_dir/notification-frontend/Dockerfile" -t notification-frontend:local "$root_dir"

kind load docker-image --name kind notification-api:local
kind load docker-image --name kind notification-worker:local
kind load docker-image --name kind notification-frontend:local

helm upgrade --install notification-system "$root_dir/deploy/helm/notification-system" \
  --namespace notification-system --create-namespace --kube-context kind-kind
for resource in statefulset/notification-postgres statefulset/notification-rabbitmq deployment/notification-api deployment/notification-worker deployment/notification-frontend; do
  kubectl --context kind-kind -n notification-system rollout status "$resource" --timeout=180s
done
