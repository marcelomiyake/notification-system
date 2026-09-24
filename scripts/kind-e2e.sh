#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
context="$(kubectl config current-context)"
[[ "$context" == "kind-kind" ]] || { printf 'Expected kind-kind context, got %s\n' "$context" >&2; exit 1; }
for deployment in notification-api notification-worker notification-frontend; do
  ready="$(kubectl --context kind-kind -n notification-system get deployment "$deployment" -o jsonpath='{.status.readyReplicas}')"
  [[ "${ready:-0}" -ge 2 ]] || { printf '%s has %s ready replicas; need at least 2\n' "$deployment" "${ready:-0}" >&2; exit 1; }
done

local_port="${E2E_PORT:-18080}"
restart_id_file="$(mktemp)"
kubectl --context kind-kind -n notification-system port-forward service/notification-frontend "$local_port:80" --address 127.0.0.1 >/tmp/notification-system-port-forward.log 2>&1 &
port_forward_pid=$!
cleanup() {
  kill "$port_forward_pid" 2>/dev/null || true
  rm -f "$restart_id_file"
}
trap cleanup EXIT

for _ in $(seq 1 40); do
  if curl --silent --fail "http://127.0.0.1:$local_port/healthz" >/dev/null; then break; fi
  sleep 1
done
curl --silent --fail "http://127.0.0.1:$local_port/healthz" >/dev/null
cd "$root_dir/notification-frontend"
E2E_BASE_URL="http://127.0.0.1:$local_port" E2E_REAL_FLOW=1 pnpm test:e2e

# Leave a scheduled request durable in PostgreSQL, restart every worker pod
# before it becomes due, then prove the scheduler recovers it after restart.
python3 - "$local_port" "$restart_id_file" <<'PY'
import datetime
import json
import sys
import time
import urllib.request

base = f"http://127.0.0.1:{sys.argv[1]}/api/v1"
id_path = sys.argv[2]
headers = {
    "X-API-Key": "local-dev-api-key",
    "Idempotency-Key": f"kind-worker-restart-{time.time_ns()}",
    "Content-Type": "application/json",
}
scheduled_at = (datetime.datetime.now(datetime.timezone.utc) + datetime.timedelta(seconds=25)).isoformat().replace("+00:00", "Z")
body = json.dumps({
    "recipient_id": "11111111-1111-4111-8111-111111111111",
    "channel": "email",
    "subject": "Kind worker restart recovery",
    "body": "Synthetic scheduled message survives a worker rollout.",
    "variables": {},
    "scheduled_at": scheduled_at,
})
request = urllib.request.Request(f"{base}/notifications", data=body.encode(), headers=headers, method="POST")
with urllib.request.urlopen(request, timeout=10) as response:
    if response.status != 202:
        raise SystemExit(f"Scheduled recovery request returned HTTP {response.status}")
    notification_id = json.load(response)["notification_id"]
with open(id_path, "w", encoding="utf-8") as handle:
    handle.write(notification_id)
print(f"Scheduled recovery notification {notification_id}")
PY

kubectl --context kind-kind -n notification-system rollout restart deployment/notification-worker
kubectl --context kind-kind -n notification-system rollout status deployment/notification-worker --timeout=120s
python3 - "$local_port" "$restart_id_file" <<'PY'
import json
import sys
import time
import urllib.request

base = f"http://127.0.0.1:{sys.argv[1]}/api/v1"
with open(sys.argv[2], encoding="utf-8") as handle:
    notification_id = handle.read().strip()
headers = {"X-API-Key": "local-dev-api-key"}
deadline = time.monotonic() + 50
while time.monotonic() < deadline:
    request = urllib.request.Request(f"{base}/notifications/{notification_id}", headers=headers)
    with urllib.request.urlopen(request, timeout=5) as response:
        notification = json.load(response)["notification"]
    status = notification["status"]
    print(f"Worker-restart recovery status: {status}")
    if status == "sent":
        break
    if status in {"failed", "suppressed"}:
        raise SystemExit(f"Recovery request reached terminal status {status}")
    time.sleep(0.5)
else:
    raise SystemExit("Scheduled notification was not delivered after worker restart")
PY
