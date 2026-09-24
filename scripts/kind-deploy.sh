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
