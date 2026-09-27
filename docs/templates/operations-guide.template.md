# {{ System/component }} operations guide

{{ Target environment and operational scope. }}

> Project documentation index: [Documentation index](../README.md)

## Contents

- [Prerequisites for build](#prerequisites-for-build)
- [Prerequisites for use](#prerequisites-for-use)
- [Build](#build)
- [Deploy](#deploy)
- [Use and verify](#use-and-verify)
- [Undeploy and data retention](#undeploy-and-data-retention)
- [Recovery and limitations](#recovery-and-limitations)

## Prerequisites for build

- {{ tool, version, cluster/context, permissions }}

## Prerequisites for use

- {{ runtime dependencies, configuration, test data, access path }}

## Build

```sh
{{ Exact build commands }}
```

## Deploy

```sh
{{ Exact deploy commands with explicit context/release/namespace }}
```

### Resource budgets

| Pod/container | CPU request | Memory request | CPU limit | Memory limit | Workload source |
| --- | ---: | ---: | ---: | ---: | --- |
| {{ name }} | {{ value }} | {{ value }} | {{ value }} | {{ value }} | [{{ chart }}]({{ path }}) |

## Use and verify

{{ Access path, health checks, and expected observable behavior. }}

## Undeploy and data retention

```sh
{{ Remove the application while preserving persistent data }}
```

{{ Explain volume/PVC retention and the distinct command that permanently removes data. }}

## Recovery and limitations

{{ Failure modes, recovery steps, backup/restore status, and evidence links. }}
