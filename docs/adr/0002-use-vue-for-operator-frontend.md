# ADR-0002: Use Vue for the Operator Frontend

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

The operator console is implemented as a Vue 3 and TypeScript single-page application built with Vite. It calls the notification API and does not own notification persistence or delivery. No original frontend selection record was found.

The scope of this decision is Vue 3 and TypeScript for the browser operator console. The source confirms the implementation; its historical selection rationale and original option set are not recorded.

## Decision drivers

- Keep the console as a small, independently built browser client.
- Support component-based forms, status views, and operator workflows with type checking and automated UI tests.
- Keep frontend state separate from the API’s durable business state.

## Options considered

### Vue 3 with TypeScript

- **Benefits:** The current component model, Vite build, type checks, and UI tests fit the implemented console.
- **Costs and risks:** The project depends on Vue-specific conventions and a separate JavaScript toolchain.

### React with TypeScript

- **Benefits:** A mature component ecosystem could support the same browser workflows.
- **Costs and risks:** Switching would rewrite the existing client and tests without evidence that Vue blocks required features.

### Server-rendered HTML

- **Benefits:** Could reduce client-side framework code for a mostly form-driven interface.
- **Costs and risks:** The current interactive console already has client-side state and API workflows; changing rendering model would add migration work.

## Decision outcome

Retain Vue 3 and TypeScript for the browser operator console. This records the current technology and a present-day rationale; it does not assert that these alternatives were compared when the project began.

## Consequences

### Positive

- The browser app has a clear UI boundary and remains a consumer of the notification API.
- Type checking and the existing frontend test/build scripts provide repeatable change feedback.

### Negative and risks

- Contributors need Vue, TypeScript, and Node.js tooling in addition to the Rust backend toolchain.
- The repository does not provide a measured comparison against another frontend framework.

## Evidence and realization

- [package.json](../../notification-frontend/package.json)
- [Dockerfile](../../notification-frontend/Dockerfile)
- [system-design.md](../system-design.md)

## Review triggers

- Reconsider if frontend maintenance becomes a measured delivery problem or if a required platform cannot be supported by the current client.
- Create a new ADR before replacing Vue or moving the UI to server rendering.

## References

- [notification-system README](../../README.md)
- [System Design](../system-design.md)
- [ADR practices](https://adr.github.io/ad-practices/)
- [ADR template guidance](https://adr.github.io/adr-templates/)
