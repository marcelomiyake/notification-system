# {{ Component name }}

{{ Purpose, runtime, and role in the parent project. }}

## Contents

- [Ownership and contracts](#ownership-and-contracts)
- [Build prerequisites](#build-prerequisites)
- [Build and run](#build-and-run)
- [Verify](#verify)
- [Browser agent support (WebMCP)](#browser-agent-support-webmcp)
- [Use and deployment](#use-and-deployment)
- [Related documentation](#related-documentation)

## Ownership and contracts

- **Owner:** {{ repository/project and component }}
- **Producers:** {{ callers or event producers }}
- **Consumers:** {{ repository/component; explicitly identify known external consumers or say unknown }}
- **Authoritative contract:** [{{ OpenAPI/schema/source }}]({{ path }})
- **Parent design:** [{{ System Design }}]({{ path }})

## Build prerequisites

- {{ Toolchain, version, dependencies }}

## Build and run

```sh
{{ Exact commands from this component directory or repository root }}
```

## Verify

```sh
{{ Narrow checks, required services/test fixtures, and expected output }}
```

For web frontends, link the Lighthouse and SEO metadata audit record. Keep Lighthouse distinct from SonarQube static analysis and advisory JEV review.

## Browser agent support (WebMCP)

{{ State enabled, unsuitable, or not applicable. If enabled, document each bounded tool, owner, authoritative contract, consumers, side effects/annotations, input validation, untrusted output, cancellation/cleanup, browser requirements, and unsupported-browser fallback. If unsuitable, state the concrete risk or missing capability. }}

## Use and deployment

{{ Access path, supported environment, deploy owner, and relevant lifecycle links. }}

## Related documentation

- [Project README]({{ path }})
- [Documentation index]({{ path }})
- [Agent guidance]({{ path }})
