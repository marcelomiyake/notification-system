# ADR-0006: Package the Notification Deployment with Helm

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

The repository keeps Kubernetes templates and configurable values in one application chart. The local scripts install or upgrade a named release and uninstall it while leaving persistent claims under the documented retention behavior. No original packaging selection record was found.

The scope of this decision is A Helm chart as the installation and upgrade interface for the local Kubernetes release. The source confirms the implementation; its historical selection rationale and original option set are not recorded.

## Decision drivers

- Keep workload templates and local resource settings together.
- Make install, upgrade, rollback, and application removal reproducible.
- Expose replica, image, and resource configuration without maintaining separate handwritten copies.

## Options considered

### Helm chart

- **Benefits:** The current chart templates the application and dependencies and provides a named release lifecycle.
- **Costs and risks:** Templating and chart values add a layer that operators must understand; storage cleanup still requires explicit care.

### Raw Kubernetes manifests or Kustomize

- **Benefits:** Fewer template concepts and clear per-environment patches.
- **Costs and risks:** Would require a different release/upgrade workflow and duplicated or patched configuration for current settings.

### Docker Compose

- **Benefits:** Familiar local container lifecycle.
- **Costs and risks:** Does not package or manage the selected Kubernetes workloads.

## Decision outcome

Keep the Helm chart as the repository’s Kubernetes packaging and release interface. Persistent-volume deletion remains an explicit data-destruction operation, separate from release uninstall.

## Consequences

### Positive

- The application stack is installed and upgraded through a versioned, reviewable chart.
- Values expose the configured replicas and resource budgets in one place.

### Negative and risks

- Template changes require careful rendered-manifest review.
- Uninstall and namespace deletion have different effects on persistent data and must remain documented.

## Evidence and realization

- [Chart.yaml](../../deploy/helm/notification-system/Chart.yaml)
- [values.yaml](../../deploy/helm/notification-system/values.yaml)
- [templates](../../deploy/helm/notification-system/templates)
- [README.md](../../README.md)

## Review triggers

- Reassess if there are multiple independent release lifecycles or if a different deployment platform becomes authoritative.
- Update this record if chart ownership, PVC retention, or deployment commands change.

## References

- [notification-system README](../../README.md)
- [System Design](../system-design.md)
- [ADR practices](https://adr.github.io/ad-practices/)
- [ADR template guidance](https://adr.github.io/adr-templates/)
