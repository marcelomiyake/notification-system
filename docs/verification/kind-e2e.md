# Kind deployment and end-to-end verification

Run date: 2026-09-24. The working tree was based on `HEAD` `3fd545f039547f6c9f9a95772c93badd3f95ccec`; it was dirty and no commit was created.

> Project documentation index: [Documentation index](../README.md)

## Environment

- Context: `kind-kind`; cluster: `kind`; namespace: `notification-system`.
- Helm release: `notification-system`, revision 4, status `deployed`.
- Application Deployments: API 2/2, worker 2/2, frontend 2/2 ready.
- Local dependencies: PostgreSQL StatefulSet 1/1 and RabbitMQ StatefulSet 1/1 ready.
- The E2E script bound its frontend port-forward to `127.0.0.1` and cleaned it up on exit. Persistent Kind database state was preserved.

## Commands and results

1. `bash scripts/kind-deploy.sh` — passed. Rebuilt the API, worker, and frontend images from the working tree, upgraded Helm revision 4, and waited for all workloads to roll out.
2. `bash scripts/kind-e2e.sh` — passed. Playwright reported `1 passed` for the operator workflow. The deployed flow exercised iOS push, Android push, SMS, and email; scheduled send; opt-out rejection; transient retry to success on attempt 2; permanent failure on attempt 1; and idempotent duplicate/conflict behavior.
3. The script then accepted a separate scheduled notification, restarted the worker Deployment, and observed the notification move from `scheduled` to `sent` after the worker rollout.

All recipients and message content were synthetic. This verifies local behavior only; it does not benchmark or prove the ByteByteGo 16 million notifications/day planning scenario.
