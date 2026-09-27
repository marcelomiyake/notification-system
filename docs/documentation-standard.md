# Markdown Documentation Standard

This standard defines how Markdown documentation is organized across this project. It keeps reader guidance, agent instructions, architecture, contracts, operating procedures, and evidence in documents with distinct owners and purposes.

> Project documentation index: [Documentation index](README.md)

## Contents

- [Core rules](#core-rules)
- [Architecture practice selection](#architecture-practice-selection)
- [Document responsibilities](#document-responsibilities)
- [README requirements](#readme-requirements)
- [Agent guidance requirements](#agent-guidance-requirements)
- [System Design outline](#system-design-outline)
- [Design Report outline](#design-report-outline)
- [Architectural Decision Records](#architectural-decision-records)
- [API and data contract requirements](#api-and-data-contract-requirements)
- [Data model documentation](#data-model-documentation)
- [Kubernetes and resource-budget practice](#kubernetes-and-resource-budget-practice)
- [Screenshots and visual evidence](#screenshots-and-visual-evidence)
- [Frontend, Lighthouse, and WebMCP verification](#frontend-lighthouse-and-webmcp-verification)
- [Post-agent documentation hooks](#post-agent-documentation-hooks)
- [Shared attribution](#shared-attribution)
- [Documentation index and maintenance](#documentation-index-and-maintenance)

## Core rules

- Keep documentation in Markdown and preserve the language of the existing project content.
- Follow the [AGENTS.md guidance](https://agents.md/): keep READMEs human-facing, dedicate `AGENTS.md` to agent instructions, and use nested files for subproject-specific guidance.
- Give each document one purpose. Keep the project `README.md` as the human entry point and `AGENTS.md` focused on agent instructions; use the nearest nested `AGENTS.md` for component-specific guidance.
- Start with one descriptive H1 and a short explanation of the document's purpose. Use a consistent H2/H3 hierarchy without skipping heading levels.
- Add a linked `Contents` list when a document has five or more H2 sections. Place it after the title and introduction. Keep it synchronized with the headings.
- Link to the authoritative document or source file instead of copying large definitions. Use relative links inside this repository and canonical repository links for cross-repository consumers.
- State whether information is implemented, proposed, assumed, observed, or unresolved. Do not invent owners, consumers, commands, measurements, or verification results.
- Use Mermaid in system designs for architecture and primary lifecycle flows. Use diagrams in other document types when they explain a real relationship better than prose.

## Architecture practice selection

Choose architectural structure from the system's actual rules, change boundaries, workload, and operating budget. Treat patterns as tools with continuing implementation, testing, token/context, deployment, and maintenance costs.

- **Clean Architecture:** keep important policy independent of delivery and storage details when those rules change or need isolated tests. Make the dependency direction clear between domain rules, use cases, and adapters. Do not create a layer, interface, or wrapper for every type or CRUD operation.
- **Domain-Driven Design:** use shared domain language and bounded contexts where rules, data authority, lifecycle, or team ownership differ. A folder, process, pod, or microservice is not automatically a bounded context. Add aggregates and value objects when they enforce named invariants; otherwise use simple modules and data types.
- **CQRS:** separate command and query models only when the read and write needs materially differ and the resulting freshness, consistency, and recovery costs are acceptable. A small query projection or immutable snapshot may be enough. Do not add event sourcing, a broker, duplicate stores, or independent deployment units just to claim CQRS.
- **YAGNI and KISS:** implement current requirements with the smallest understandable design. Keep boundaries discoverable and tasks narrow; defer hypothetical scale, providers, or flexibility until evidence justifies them.
- **DRY:** keep one authoritative source for contracts and rules, and share code when behavior and ownership are genuinely the same. Avoid coupling independent contexts through a generic abstraction created only to remove a few similar lines.
- **AI-assisted change cost:** make ownership, dependencies, contracts, and verification steps easy to find. Prefer a small task with a focused file/test cone over broad rewrites. Count prompt/context size, setup time, compile/test feedback, operational resources, and migration work when comparing designs.

Record the problem a selected pattern solves, its simpler alternative, its consistency and operational trade-offs, and a condition for revisiting it. These are decision lenses, not universal performance claims; see the [AI architecture guide](https://super-productivity.com/blog/ai-software-architecture-guide/), [Navigation Paradox preprint](https://arxiv.org/html/2602.20048v1), and [agentic coding practitioner analysis](https://addyo.substack.com/p/the-80-problem-in-agentic-coding).

## Document responsibilities

| Document | Primary audience and responsibility |
| --- | --- |
| Root `README.md` | People evaluating, building, deploying, or using the project; quick path and links to canonical detail. |
| Component `README.md` | People building or using one service, client, chart, database asset, or verification area. |
| `AGENTS.md` | Coding agents; repository sources of truth, boundaries, commands, conventions, security constraints, and safe workflow. Avoid duplicating full user runbooks. |
| `CLAUDE.md` | Claude Code compatibility import; place beside every `AGENTS.md` and use only `@AGENTS.md` as its content. |
| System Design | Architecture, requirements, component/data ownership, interfaces, failure behavior, operations, and decisions. |
| Design Report | Evidence-led findings, implications, recommendations, and sources for a review or change. |
| ADR | One significant architectural decision, its context, options, consequences, implementation evidence, and review triggers. |
| API/contract record | Interface owner, producer, consumers, authoritative definition, and compatibility guarantees. |
| Operations guide | Build/deploy/use/undeploy procedures, prerequisites, expected results, and recovery or data-retention effects. |
| Verification record | Exact revision, environment, commands, actual outcomes, evidence, and remaining gaps; web frontends also record Lighthouse and SEO metadata checks. |
| Design notes | Visual language, interaction states, accessibility, responsive behavior, and review evidence. |
| Other Markdown | Use the smallest structure appropriate to its purpose; link it from the documentation index and related source-of-truth docs. |

## README requirements

Root software READMEs include, when applicable:

1. Project summary, current status, capabilities, scope, and key boundaries.
2. Documentation index and links to the System Design, API contracts, component guides, operations, and verification evidence.
3. **Build prerequisites** and **use/runtime prerequisites** as separate lists.
4. Reproducible build and relevant test/verification commands.
5. Deployment instructions for the supported target, then explicit undeployment instructions.
6. A user-oriented use/quickstart path and configuration notes.
7. Real screenshots of user-facing UI, with useful alt text and a relative path to the checked-in asset.
8. The common AI development disclaimer below.

Component READMEs explain the component's responsibility, owner, consumers, source-of-truth contract, build/run/test steps, and links to project operations. Describe deploy/undeploy only when the component has an independent deployment lifecycle. For non-software material, explain how to read or contribute to it and state that build/deploy steps do not apply.

Undeployment instructions must distinguish removing the application from deleting persistent data. Name the exact command that removes PVCs or named volumes, and never describe a data-purging command as routine cleanup.

## Agent guidance requirements

Keep `AGENTS.md` actionable and concise: identify the project/component, point to human documentation and authoritative code/contracts, describe boundaries and conventions, list the narrow build/test checks, and record security or data-safety constraints. Nested files add only local instructions and inherit the repository guidance. Do not copy full README or System Design sections into agent files.

## System Design outline

Use this shared section order for system-design documents. Keep a section short or mark it not applicable only when the system genuinely has no relevant concern.

1. **Abstract** — problem, intended outcome, boundary, workloads, and constraints.
2. **Goals and Non-Goals** — explicit in-scope outcomes and exclusions.
3. **Background and Problem Statement** — current state, evidence, assumptions, and requirements.
4. **Proposed Architecture** — Mermaid component/deployment diagram, responsibilities, data stores, trust boundaries, and ownership.
5. **Request Lifecycle** — Mermaid sequence/flow for the main request, event, or job and its failure path.
6. **API and Data Contracts** — interfaces, owner, producers, consumers by repository/component, schemas, and source-of-truth links.
7. **Consistency, Idempotency, and Replay** — persistence boundary, retries, ordering, duplicates, and recovery.
8. **Security and Privacy Considerations** — authentication, authorization, secrets, sensitive data, logging, retention, and safe defaults.
9. **Operational Readiness** — prerequisites, build/deploy/undeploy/use links, probes, failure handling, and CPU/memory requests and limits for every Kubernetes container.
10. **Alternatives Considered** — meaningful options and consequences.
11. **Open Questions** — unresolved decisions with an owner or next step.
12. **Decision and Next Steps** — selected direction, implementation sequence, and acceptance criteria.
13. **References and Traceability** — source paths, external references, and verification records.

A Mermaid diagram represents the implemented state only when checked against source/configuration. Label proposed elements as proposed.

## Design Report outline

Use the retained Design Report structure in Markdown: **Executive Summary**, **At a Glance**, **Introduction**, **Key Findings** (including context/conditions, evidence patterns, and implications), **Recommendations**, **Conclusion**, and **Appendix** (notes and sources). Separate observed evidence from interpretation and recommendation. Replace or remove empty placeholders.

## Architectural Decision Records

- Keep one decision per record under `docs/adr/`, use zero-padded sequential filenames such as `0001-use-postgresql-frontier.md`, and link every record from `docs/adr/README.md` and this project's `docs/README.md`.
- Use a consistent lean MADR/Nygard structure: title, status, record date, decision owner, context/problem, decision drivers, at least two considered options with their trade-offs, decision outcome, consequences, implementation evidence, review triggers, and references.
- Treat ADRs as durable history. Do not rewrite a past outcome to fit a later design; create a new ADR that supersedes it and link the records.
- Distinguish observed implementation facts from reconstructed rationale. For retrospective ADRs, state that the current implementation is being recorded and mark the original decision date or approval as unknown when repository evidence does not establish it. Do not claim stakeholder agreement that was not recorded.
- Keep the ADR concise and decision-specific. Use `Proposed` until a decision owner confirms a new decision; for existing code, use a status such as `Implemented (retrospective)` when that is the evidence-supported state.
- Include an evidence and review plan: link the authoritative source/configuration, name conditions that should trigger reconsideration, and state the next verification or owner where known.

This practice follows [AD Practices](https://adr.github.io/ad-practices/) and the [ADR Templates guidance](https://adr.github.io/adr-templates/), which discuss decision context, options and pros/cons, metadata, and review.

## API and data contract requirements

For every HTTP API, event, queue message, shared schema, or cross-component boundary, identify:

| Field | Required content |
| --- | --- |
| Contract and kind | Stable name plus REST, event, queue, schema, or other interface type. |
| Owner | Repository/project and component that defines and changes the contract. |
| Producer | Repository/component that emits requests, events, or writes. |
| Consumers | Every known consumer as repository plus component; say `None identified in the inspected repositories` only after checking, otherwise say `Unknown`. |
| Authority | Relative or canonical link to OpenAPI/schema/source code that is authoritative. |
| Compatibility | Versioning, idempotency, ordering, error/retry, authentication, and data guarantees relevant to this interface. |

List external consumers in their actual repository, not only by service name. Cross-repository links must point to the consumer's README, System Design, or contract record. A repository scan that finds no consumer is evidence of “none identified,” not proof that no external consumer exists.

## Data model documentation

For each repository-owned relational or key-value schema, maintain a Markdown database model linked from the System Design and documentation index. Include the owning project/component, authoritative migration or schema path, known producer and consumers, and a table for every relation with each column's type, constraints/nullability, and plain-language meaning. Include key relationships, generated fields, important checks/indexes, retention/cascade behavior, and seed-data caveats. Keep the source schema authoritative; do not copy the model back into migration files. Mark cross-repository consumers `Unknown` until inspected. For vendor-managed schemas without repo migrations, document the ownership boundary and avoid treating private internal tables as a supported contract.

## Kubernetes and resource-budget practice

Every pod template in this repository's Kubernetes artifacts must set both `resources.requests` and `resources.limits` for **CPU and memory on every container**, including init containers and sidecars. PVC storage requests are separate and do not replace CPU/memory settings. Keep values in chart values where practical and make sure templates consume them.

Documentation for a Kubernetes system includes a workload table with the source chart path and each pod/container's CPU request, memory request, CPU limit, and memory limit. Report chart-configured values, not observed consumption or production sizing claims. Tune them from measured workload evidence; explicitly identify local/demo budgets as local defaults.

## Screenshots and visual evidence

Capture genuine screenshots from the running project UI. Store them under `docs/assets/screenshots/`, use descriptive filenames, and include informative Markdown alt text. State whether a screenshot is from a local/demo deployment when that affects interpretation. Do not use mockups or generated images as system screenshots. Headless services and static documents do not need screenshots; link component guides to the root project screenshot when relevant.

## Frontend, Lighthouse, and WebMCP verification

For each owned web frontend, assess whether WebMCP offers a useful browser-agent capability. Enable only bounded tools that reuse an authoritative application contract and have a clear owner and consumer. Prefer read-only operations; do not expose sensitive data or consequential writes without a separate security and authorization design. Validate inputs server-side, mark untrusted content and consequential behavior accurately, honor cancellation, unregister on teardown, and keep the UI usable when WebMCP is unavailable. Record enabled, unsuitable, or not-applicable status and the reason in the component README and contract catalog. Native clients and vendor-owned frontends are outside WebMCP scope unless they implement a web document surface.

Run Lighthouse against a production build on the target routes and record tool/browser versions, audit mode/device, URL, category scores, findings, and any unavailable dependencies. Lighthouse is a browser quality signal; keep it separate from SonarQube's static analysis and JEV's advisory readiness judgment. Do not imply one replaces another.

When WebMCP is in scope, also run Lighthouse's experimental Agentic Browsing category on Chrome 150 or newer when practical. The WebMCP registration audits require the current origin trial; record trial-dependent checks as unavailable or not applicable unless the page was actually audited with the required browser setup. The category's fractional score is informational, not a 0–100 quality score.

Check browser-facing metadata against the published SEO META in 1 CLICK checklist: title and length, description and length, canonical URL, robots directives, heading sequence, image alt text, links, Open Graph/Twitter tags, `robots.txt`, and sitemap. If the extension is not installed, label this as a manual checklist audit rather than claiming the extension ran. For local/private demos, document intentional omissions such as public canonical/social URLs and retain a suitable `noindex` policy; do not add localhost URLs as public metadata. Robots directives are indexing hints, not access control; private systems still need network and authorization boundaries.

## Shared attribution

Include this disclaimer in every `README.md` (translate faithfully only when the document language is not English):

> **AI development disclaimer:** This project was built entirely with GPT-6 Luna at Max effort as a proof of concept exploring how low-cost AI plans can be useful when paired with disciplined harness and loop engineering. This is project-owner attribution; repository contents do not independently verify runtime model metadata. Review AI-generated design and code before relying on them.

## Post-agent documentation hooks

Recommend a repository-local Codex `SubagentStop` hook to inspect source changes after agent jobs and return targeted Markdown follow-up context to the parent agent. An optional `Stop` hook can make the main turn perform a final documentation pass. Map API/OpenAPI, event, database, client/consumer, deployment, resource-budget, and UI changes to the contracts, database model, System Design, operations, and README documents that may need updating. The hook should inspect the worktree, skip docs-only changes to prevent loops, surface unknown ownership/consumers for research, and ask the agent to verify Markdown against source. It should not parse session transcripts as a stable API or rewrite authoritative docs blindly. Review and trust the exact hook definition before enabling it. See [Post-agent documentation synchronization hooks](agent-documentation-hooks.md) and the [official Codex Hooks reference](https://learn.chatgpt.com/docs/hooks).

## Documentation index and maintenance

`docs/README.md` is the repository documentation index. It links every tracked Markdown document by role and links the main implementation/configuration contracts. Root READMEs link to this index; component guides link to their project README, applicable System Design/contract, and their nearest `AGENTS.md`.

When a change affects behavior, ownership, interface, commands, resource budgets, deployment, or verification, update the relevant source-of-truth document in the same change. Keep the copies of this standard and the templates identical across the participating repositories.
