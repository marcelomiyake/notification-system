# Notification API agent guidance

> Human guide: [README.md](../README.md) · [Documentation index](../docs/README.md)

- This service owns the Notification Management bounded context. Do not implement provider calls or delivery retries here.
- PostgreSQL is authoritative. Accept a notification only after its notification, caller-scoped idempotency key, and outbox row commit in one transaction.
- Authentication, rate limits, validation, opt-out checks, recipient/device operations, template versioning, and request status routes belong here.
- Recheck preferences in the worker immediately before adapter invocation; an API-time check is not sufficient.
- Never return or log API keys, full device tokens, full phone/email destinations, or message bodies in status/list endpoints.
- Keep behavior covered by unit tests and disposable database/API integration tests. See this service README for commands and fixtures.
- Preserve the local-only boundary: synthetic recipients and no public ingress.
