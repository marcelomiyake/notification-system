# ADR-0005: Run Notification Services on Kubernetes

- **Status:** Implemented (retrospective)
- **Recorded:** 2026-09-25
- **Original decision date:** Unknown from repository evidence
- **Decision owner:** Project owner
- **Confirmation:** Current implementation is documented at the project owner’s request; historical team approval is not recorded.

> This record documents the technology in the current implementation. The options and rationale below are a retrospective comparison, not a claim that the original project formally evaluated them.

## Contents

- [Context and problem statement](#context-and-problem-statement)
- [Decision drivers](#decision-drivers)
- [Options considered](#options-considered)
- [Decision outcome](#decision-outcome)
- [Consequences](#consequences)
- [Evidence and realization](#evidence-and-realization)
- [Review triggers](#review-triggers)
- [References](#references)

## Context and problem statement

The repository deploys API, worker, frontend, PostgreSQL, and RabbitMQ workloads to a local Kind cluster. Applications use Deployments and Services; stateful dependencies retain persistent claims. This is a local learning profile, not evidence of production availability.

The scope of this decision is Kubernetes as the deployment runtime for the local notification application. The source confirms the implementation; its historical selection rationale and original option set are not recorded.

## Decision drivers

- Exercise service discovery, probes, rollout, and multi-replica application behavior.
- Keep API, worker, frontend, and stateful dependencies as separately configured workloads.
- Make CPU and memory requests and limits explicit for every pod container.

## Options considered

### Kubernetes on local Kind

- **Benefits:** Matches the checked-in workload resources and supports local validation of Kubernetes deployment behavior.
- **Costs and risks:** Requires Docker, Kind, kubectl, cluster storage, and resource budgeting; one local cluster does not prove high availability.

### Docker Compose

- **Benefits:** Simpler workstation startup for a small stack.
- **Costs and risks:** Would not exercise the checked-in Kubernetes workload model or its service, probe, and PVC behavior.

### Run processes directly on the host

- **Benefits:** Minimal container orchestration overhead.
- **Costs and risks:** Does not reproduce the service isolation, discovery, and deployment topology used by this project.

## Decision outcome

Keep Kubernetes resources as the application deployment model and use the existing local Kind workflow for development. Do not treat the local cluster as production sizing or HA evidence.

## Consequences

### Positive

- The local workflow exercises Kubernetes Services, Deployments, persistent storage, readiness, and application replica configuration.
- Pod resource requests and limits are reviewed as part of deployment configuration.

### Negative and risks

- Local Kind has constrained resources and a limited failure domain.
- Kubernetes adds setup and troubleshooting overhead compared with a single-process or Compose workflow.

## Evidence and realization

- [templates](../../deploy/helm/notification-system/templates)
- [kubernetes-resources.md](../kubernetes-resources.md)
- [system-design.md](../system-design.md)
- [README.md](../../README.md)

## Review triggers

- Reconsider if the supported deployment target changes or if the project no longer needs Kubernetes-specific validation.
- Before using a remote cluster, review credentials, network exposure, secrets, storage, and resource budgets.

## References

- [notification-system README](../../README.md)
- [System Design](../system-design.md)
- [ADR practices](https://adr.github.io/ad-practices/)
- [ADR template guidance](https://adr.github.io/adr-templates/)
