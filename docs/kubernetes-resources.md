# Kubernetes CPU and memory budgets

The Helm chart's configured CPU and memory requests and limits are listed below. These are local chart defaults, not measured consumption or production sizing claims.


> Project documentation index: [Documentation index](README.md)


## Workload budgets

| Pod/workload and container | CPU request | Memory request | CPU limit | Memory limit | Source |
| --- | ---: | ---: | ---: | ---: | --- |
| `notification-api` / `api` | 50m | 64Mi | 500m | 256Mi | [Chart values](../deploy/helm/notification-system/values.yaml) |
| `notification-worker` / `worker` | 50m | 64Mi | 500m | 256Mi | [Chart values](../deploy/helm/notification-system/values.yaml) |
| `notification-frontend` / `frontend` | 25m | 32Mi | 200m | 128Mi | [Chart values](../deploy/helm/notification-system/values.yaml) |
| `notification-postgres` / `postgres` | 100m | 128Mi | 500m | 512Mi | [Chart values](../deploy/helm/notification-system/values.yaml) |
| `notification-rabbitmq` / `rabbitmq` | 100m | 256Mi | 500m | 768Mi | [Chart values](../deploy/helm/notification-system/values.yaml) |

The application uses one PostgreSQL and one RabbitMQ pod in the local chart. Each application Deployment has two replicas. The PVC storage requests (`1Gi` each) are separate from these CPU/memory budgets.

## Kubernetes resource management practice

Set CPU and memory `requests` and `limits` on every container in every Pod, including init containers and sidecars. Requests guide scheduling and reserve baseline capacity; CPU limits may throttle, and memory limits can trigger OOM termination. Measure representative usage, leave startup/burst headroom, monitor throttling and restarts, and right-size deliberately. Treat the listed values as local development defaults, not production sizing guidance. PVC storage requests are separate from container budgets.

## Build, use, and persistence

The [root README](../README.md) documents build, deploy, use, and undeploy commands with their persistent-data effects. The [Helm chart guide](../deploy/helm/notification-system/README.md) is the deployment owner. This resource inventory is configuration guidance and does not itself deploy workloads.
