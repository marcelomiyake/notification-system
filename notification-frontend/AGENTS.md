# Notification frontend agent guidance

> Human guide: [README.md](../README.md) · [Documentation index](../docs/README.md)

- Keep the operator UI a client of the Notification Management API; never duplicate authoritative notification or preference policy in Vue state.
- Use local synthetic recipients only. The API key embedded for Kind is a local fixture, not a production credential. Do not add real provider credentials or PII.
- Preserve accessible names, labels, keyboard focus, loading/empty/error/success/disabled states, and reduced-motion behavior.
- Use `api.ts` for HTTP calls and shared request/response types from `types.ts`. Keep API versioning and channel values aligned with `docs/openapi.yaml`.
- Unit tests should cover rendering/validation/state. Playwright tests must include the real Kind flow when `E2E_BASE_URL` is set; mocked browser tests are not proof of backend integration.
- See `design-notes.md` for the prototype's visual language and accessibility decisions. Keep the OpenDesign tool out of runtime dependencies.
- Do not register WebMCP tools for notification dispatch or recipient/preference data; these actions and records require an explicit security and authorization design first.
