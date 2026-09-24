# {{ Project or component }} agent guidance

## Sources of truth and boundaries

> Project documentation index: [Documentation index](../README.md)

- Human entry point: [README]({{ path }}).
- Architecture and decisions: [System Design]({{ path }}).
- API/data contract: [{{ contract }}]({{ path }}).
- This component owns {{ responsibilities }}; it does not own {{ explicit boundaries }}.

## Build and verification

- Prerequisites: {{ supported versions/configuration }}.
- Build: `{{ command }}`
- Focused checks: `{{ command }}`
- State required disposable/local dependencies and where results are recorded.

## Conventions and safety

- {{ Local style/domain/security/data constraints }}
- {{ Deployment context, namespace, and persistence caution where applicable }}
- Do not claim commands or checks passed unless actually run.
