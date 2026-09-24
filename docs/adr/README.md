# Architectural Decision Records

This index records the significant architectural choices for Notification System. Each ADR covers one decision and links to its implementation evidence.

## Records

- [ADR-0001: Use Rust for Backend Services](0001-use-rust-for-backend-services.md)
- [ADR-0002: Use Vue for the Operator Frontend](0002-use-vue-for-operator-frontend.md)
- [ADR-0003: Use PostgreSQL for Notification State](0003-use-postgresql-for-notification-state.md)
- [ADR-0004: Use RabbitMQ for Channel Delivery](0004-use-rabbitmq-for-channel-delivery.md)
- [ADR-0005: Run Notification Services on Kubernetes](0005-run-notification-services-on-kubernetes.md)
- [ADR-0006: Package the Notification Deployment with Helm](0006-package-notification-deployment-with-helm.md)

## Maintaining this log

New records use the [ADR template](../templates/adr.template.md). Keep one decision per file, number records sequentially, and add a new ADR when an accepted or implemented decision changes. Do not rewrite historical outcomes. Retrospective records distinguish known implementation evidence from unknown original decision dates or approval history.

This Markdown index has no separate software build, deploy, use, or undeploy lifecycle; see the [project README](../../README.md) for application operations.

## AI development disclaimer

> **AI development disclaimer:** This project was built entirely with GPT-6 Luna at Max effort as a proof of concept exploring how low-cost AI plans can be useful when paired with disciplined harness and loop engineering. This is project-owner attribution; repository contents do not independently verify runtime model metadata. Review AI-generated design and code before relying on them.
