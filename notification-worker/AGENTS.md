# Notification worker agent guidance

> Human guide: [README.md](../README.md) · [Documentation index](../docs/README.md)

- This service owns the Delivery bounded context: due-time dispatch, transactional-outbox relay, Rabbit routing, attempts, retries, DLQs, and adapter outcomes.
- PostgreSQL is the durable source of truth; RabbitMQ is a transport buffer. Confirm publishes before marking outbox rows published.
- Keep four independent routing keys/queues and dead-letter queues. A blocked provider/channel must not stop unrelated channels.
- Recheck recipient preferences immediately before adapter invocation. Persist terminal state and attempt results so redelivery is idempotent.
- The built-in adapters only record synthetic, local provider acceptance. Do not add provider credentials or claim recipient delivery.
- Retry transient failures with bounded backoff; permanent failures are terminal. Test both branches and worker restart recovery.
- Use unit tests for routing/rendering/state/retry decisions and disposable PostgreSQL/RabbitMQ integration tests. See README for commands.
