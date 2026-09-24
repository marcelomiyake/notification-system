# {{ Contract name }}

{{ Brief purpose, current status, and authoritative source. }}

> Project documentation index: [Documentation index](../README.md)

## Contents

- [Ownership and consumers](#ownership-and-consumers)
- [Authoritative definition](#authoritative-definition)
- [Contract behavior](#contract-behavior)
- [Security and compatibility](#security-and-compatibility)
- [Related documentation](#related-documentation)

## Ownership and consumers

| Role | Repository | Component | Responsibility |
| --- | --- | --- | --- |
| Owner | {{ repo }} | {{ component }} | Defines and changes this contract. |
| Producer | {{ repo }} | {{ component }} | Emits requests, events, or writes. |
| Consumer | {{ repo }} | {{ component }} | Reads, calls, or processes the contract. |

Record each known consumer separately. If inspection did not find a consumer, state `None identified in the inspected repositories ({{ date/revision }})`; if unverified, state `Unknown`.

## Authoritative definition

- **Source:** [{{ OpenAPI/schema/code }}]({{ relative or canonical link }})
- **Version/protocol:** {{ version, transport, content type }}

## Contract behavior

| Operation/message | Request/event | Success | Errors/retry | Idempotency/order |
| --- | --- | --- | --- | --- |
| {{ operation }} | {{ shape or schema link }} | {{ result }} | {{ behavior }} | {{ guarantee }} |

## Security and compatibility

{{ Authn/authz, sensitive data, version evolution, deprecation, and backward-compatibility rules. }}

## Related documentation

- [System Design]({{ path }})
- [Consumer documentation]({{ path }})
- [Verification evidence]({{ path }})
