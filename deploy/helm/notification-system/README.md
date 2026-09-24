# notification-system Helm chart

This chart is scoped to the existing local Kind cluster and `notification-system` namespace. It creates ClusterIP-only services; there is no Ingress, NodePort, or LoadBalancer. Port-forward only the frontend to loopback.

> Documentation: [project index](../../../docs/README.md) · [repository overview](../../../README.md)

The API, worker, and frontend Deployments each default to two replicas. PostgreSQL and RabbitMQ run as single StatefulSet replicas with local-path PVCs. This is a learning cluster, not production HA, backup, or throughput evidence. Helm values contain deliberately weak synthetic local credentials; never reuse them elsewhere.

From repository root: `scripts/kind-deploy.sh`.

Manual inspection: `kubectl -n notification-system get deploy,pods,svc,pvc`. To open the UI: `kubectl -n notification-system port-forward service/notification-frontend 8080:80 --address 127.0.0.1`.

## Build and use prerequisites

- **Build/deploy:** use the root project README for source-image build commands and the chart instructions below for the Helm release. Required tools are Docker, the supported local Kubernetes cluster/context, kubectl, and Helm.
- **Use:** the cluster release must be ready; use the loopback browser/port-forward instructions in the root README. See the resource table for each pod container budget.


## Resource budgets and undeploy

All five workload containers have CPU and memory requests and limits. See the [Kubernetes resource budget](../../../docs/kubernetes-resources.md); the configurable values are in `values.yaml`. PostgreSQL and RabbitMQ use `Retain` PVC policies. Remove the release while preserving claims with:

```sh
helm uninstall notification-system --namespace notification-system
```

Deleting the namespace or PVCs removes database/queue data. See the [documentation index](../../../docs/README.md).


## AI development disclaimer

> **AI development disclaimer:** This project was built entirely with GPT-6 Luna at Max effort as a proof of concept exploring how low-cost AI plans can be useful when paired with disciplined harness and loop engineering. This is project-owner attribution; repository contents do not independently verify runtime model metadata. Review AI-generated design and code before relying on them.
