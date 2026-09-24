# ADR-0001: Use Rust for Backend Services

- **Status:** Implemented (retrospective)
- **Recorded:** 2026-09-25
- **Original decision date:** Unknown from repository evidence
- **Decision owner:** Project owner
- **Confirmation:** Current implementation is documented at the project owner's request; historical team approval is not recorded.

> This record captures the Rust backend already present in the repository. The options and rationale below are a retrospective comparison, not a claim that the original project formally evaluated them.

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

The system has a Rust HTTP API that accepts notification requests and a Rust worker that dispatches deliveries, while the operator console is a Vue application. The API and worker perform concurrent I/O and persist delivery state.

The project owner's rationale is that Rust is safer, fast, and can use less CPU and memory. The owner also recognizes Rust's deeper learning curve and considers it feasible with AI-assisted code authoring. These are design expectations and the owner's experience, not results from a comparative benchmark in this repository.

## Decision drivers

- Catch memory-safety and concurrency errors through Rust's type and ownership checks.
- Keep runtime overhead and expected CPU/memory needs suitable for the local Kubernetes resource budgets.
- Support responsive request handling and independent asynchronous delivery workers.
- Make Rust development manageable for this proof of concept with AI assistance, compiler feedback, focused tests, and human review.

## Options considered

### Rust for the API and worker

- **Benefits:** Compiled native services, strong compile-time ownership and type checks, and a small runtime footprint are a good fit for concurrent network services.
- **Costs and risks:** Ownership, lifetimes, async types, and compiler diagnostics have a steeper learning curve; builds can take longer than a small scripting service.

### Go for the API and worker

- **Benefits:** A comparatively approachable service language, static types, built-in concurrency primitives, and a straightforward deployment model.
- **Costs and risks:** Garbage collection and a different type/concurrency model; switching would add a second language without evidence that Rust is a project bottleneck.

### TypeScript/Node.js for the API and worker

- **Benefits:** Could reuse the frontend language and reduce language switching for contributors.
- **Costs and risks:** The backend's concurrent delivery and persistence code would use a managed runtime and would not receive Rust's ownership checks; resource use would need measurement rather than assumption.

No cross-language performance benchmark is available, so the trade-offs are qualitative.

## Decision outcome

Use Rust for the notification API and delivery worker. AI-assisted authoring makes the learning curve acceptable for this proof of concept, paired with compiler checks, automated tests, documented contracts, and code review. AI-generated code is not treated as verified solely because it compiles or was produced by a capable model.

## Consequences

### Positive

- The compiler and ownership model catch many memory-safety and data-race classes before deployment.
- Native binaries and low runtime overhead are expected to fit the configured local budgets; these benefits still require measurement under representative load.
- AI assistance can help contributors navigate Rust syntax and patterns while existing tests and review preserve accountability.

### Negative and risks

- Contributors need time to learn the borrow checker, async Rust, and the codebase's domain boundaries.
- The API and worker require Rust toolchains in local builds and CI.
- Generated code can encode incorrect domain behavior; reviewers must check invariants, retries, idempotency, and privacy behavior.

## Evidence and realization

- The [notification API manifest](../../notification-api/Cargo.toml) and [worker manifest](../../notification-worker/Cargo.toml) define the Rust services.
- The [System Design](../system-design.md) describes the API, transactional outbox, RabbitMQ delivery, and worker responsibilities.
- The [resource budget record](../kubernetes-resources.md) gives configured local CPU/memory requests and limits. It does not compare Rust with another language or report production consumption.

## Review triggers

- Reconsider if representative profiling shows the Rust services miss a measured performance or resource target and a controlled alternative-language comparison indicates a better fit.
- Reconsider if Rust's learning or maintenance cost remains a material delivery risk despite AI assistance, compiler tooling, tests, and review.
- Create a new ADR before migrating a backend component to another language or runtime.

## References

- [Notification System README](../../README.md)
- [System Design](../system-design.md)
- [ADR practices](https://adr.github.io/ad-practices/)
- [ADR template guidance](https://adr.github.io/adr-templates/)
