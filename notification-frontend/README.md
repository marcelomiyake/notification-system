# notification-frontend

Vue 3/TypeScript operator console. It shows request counts from API state, composes immediate or scheduled notifications, edits per-channel preferences, and displays recent status. The UI labels recording-adapter outcomes as simulated provider acceptance.

> Documentation: [project index](../docs/README.md) · [repository overview](../README.md)

## Contents

- [Run and test](#run-and-test)
- [OpenDesign prototype and design notes](#opendesign-prototype-and-design-notes)
- [Image](#image)
- [Browser agent support (WebMCP)](#browser-agent-support-webmcp)
- [Component ownership, prerequisites, and lifecycle](#component-ownership-prerequisites-and-lifecycle)
- [AI development disclaimer](#ai-development-disclaimer)

## Run and test

```sh
pnpm install --frozen-lockfile
pnpm dev
pnpm test:unit
pnpm test:e2e
pnpm build
```

Vite proxies `/api` to `http://127.0.0.1:8080`. For an end-to-end run against the Kind port-forward, set `E2E_BASE_URL=http://127.0.0.1:8080` before `pnpm test:e2e`; the real cluster E2E requires a running API and worker. The default API key is intentionally visible in the local browser bundle; this is only suitable for the isolated loopback demo.

The production-build browser and metadata audit is recorded in [Lighthouse verification](../docs/verification/lighthouse.md); it does not replace the API-backed Kind flow.

## OpenDesign prototype and design notes

The reviewed [OpenDesign prototype](http://127.0.0.1:17758/projects/93372b76-ddcc-44d0-aa3e-5bb6ddfed83f/conversations/f12440a0-b769-4a95-bd1e-cd2d835ad499/files/index.html) is available in the local OpenDesign project. Its visual direction, interaction states, and accessibility review are recorded in [`design-notes.md`](design-notes.md). OpenDesign is a design-time prototype tool only; the shipped app has no OpenDesign runtime dependency.

## Image

From repository root: `docker build -f notification-frontend/Dockerfile -t notification-frontend:local .`.

## Browser agent support (WebMCP)

WebMCP is not enabled for this operator console. Its workflows can dispatch notifications, expose recipient and preference data, and update persistent settings. A browser-agent surface would need an explicitly reviewed capability boundary and authorization model before any operations could be registered. Keep the normal operator API and UI as the supported workflow.

## Component ownership, prerequisites, and lifecycle

- **Owner:** `notification-system` / `notification-frontend`.
- **API, event, and data contract owners/producers/consumers:** see the [contract catalog](../docs/contracts/README.md) for each authoritative interface.
- **Parent architecture:** [System Design](../docs/system-design.md).

### Build prerequisites

Use this component’s pinned toolchain and lockfile/wrapper. The supported versions and complete local build environment are listed in the [root README](../README.md).

### Use prerequisites

This component is used as part of the parent system. Start its required local dependencies and use the supported local access path described in the [root README](../README.md).

### Build, verify, deploy, undeploy, and use

Build, run, and verification commands for this component are documented above. There is no independent release lifecycle for this component.
The parent Helm release owns deployment and removal; follow the [chart guide](../deploy/helm/notification-system/README.md) and [root deployment lifecycle](../README.md). Uninstall removes application workloads while retained PVCs keep their data; deleting the namespace or claims purges persistent data.

[Documentation index](../docs/README.md)


## AI development disclaimer

> **AI development disclaimer:** This project was built entirely with GPT-6 Luna at Max effort as a proof of concept exploring how low-cost AI plans can be useful when paired with disciplined harness and loop engineering. This is project-owner attribution; repository contents do not independently verify runtime model metadata. Review AI-generated design and code before relying on them.
