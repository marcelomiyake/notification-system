# {{ Project name }}

{{ One-sentence purpose, status, and intended audience. }}

## Contents

- [Overview](#overview)
- [Build prerequisites](#build-prerequisites)
- [Use prerequisites](#use-prerequisites)
- [Build and verify](#build-and-verify)
- [Deploy](#deploy)
- [Undeploy](#undeploy)
- [Use](#use)
- [Screenshots](#screenshots)
- [Contracts and documentation](#contracts-and-documentation)
- [AI development disclaimer](#ai-development-disclaimer)

## Overview

{{ Capabilities, boundaries, local/demo limits, and data used. }}

## Build prerequisites

- {{ Tool and supported version }}

## Use prerequisites

- {{ Runtime/service/account/configuration requirement distinct from build tools }}

## Build and verify

```sh
{{ Reproducible build command }}
{{ Narrow verification command(s), with prerequisites stated }}
```

## Deploy

{{ Supported environment and exact deploy command. State context/namespace and required secrets/configuration. }}

## Undeploy

```sh
{{ Remove application resources while preserving data where possible }}
```

{{ Explain which command deletes persistent data and its effect. }}

## Use

{{ Quickstart, access URL/client, and first useful operation. }}

## Screenshots

![{{ Describe the actual UI state and surface }}](assets/screenshots/{{ screenshot-name }}.png)

{{ State local/demo provenance and any limits. Omit this section for headless or non-software projects. }}

## Contracts and documentation

| Interface | Owner | Consumers | Authority |
| --- | --- | --- | --- |
| {{ contract }} | {{ repo/component }} | {{ repo/component(s) }} | [{{ source }}]({{ relative path }}) |

- [Documentation index](docs/README.md)
- [System Design](docs/system-design.md)
- [API contract catalog](docs/contracts/README.md)
- [Agent guidance](AGENTS.md)
- [Verification](docs/verification/README.md)

## AI development disclaimer

> **AI development disclaimer:** This project was built entirely with GPT-6 Luna at Max effort as a proof of concept exploring how low-cost AI plans can be useful when paired with disciplined harness and loop engineering. This is project-owner attribution; repository contents do not independently verify runtime model metadata. Review AI-generated design and code before relying on them.
