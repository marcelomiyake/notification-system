# Post-agent documentation synchronization hooks

Recommended Codex hook design for keeping architecture, API contracts, consumer maps, database models, and operational Markdown aligned with code changes. This repository does not install or enable the hook; review and trust any hook configuration before it runs.

> Official reference: [Codex Hooks](https://learn.chatgpt.com/docs/hooks).

## Contents

- [Goal and scope](#goal-and-scope)
- [Lifecycle events](#lifecycle-events)
- [Recommended workflow](#recommended-workflow)
- [Example project configuration](#example-project-configuration)
- [Trust and safety](#trust-and-safety)

## Goal and scope

Use a `SubagentStop` hook to check the repository after an agent finishes and a `Stop` hook as an optional final pass after the main agent turn. Have a small repo-local script classify changed paths and provide concise documentation follow-up context to the agent. The script should guide a source-checked documentation update rather than guess or invent API owners, consumers, schema meaning, or runtime evidence.

Relevant source changes include:

- OpenAPI files, routes, request/response types, event schemas, queue payloads, and public SDK/client code.
- Browser WebMCP tool registrations, input schemas, output/annotation behavior, and browser-agent consumers.
- Database migrations, ORM models, persistence queries, and seed data that changes the documented model.
- Client or service integration changes that add, remove, or change API/queue consumers, including consumers in another repository.
- Helm values/templates, Dockerfiles, scripts, and build/deploy commands that change operations or CPU/memory budgets.
- User-facing flows that affect README feature descriptions or screenshots.

## Lifecycle events

- `SubagentStop` is useful after an individual agent job because its output can add context for the parent agent to finish the documentation sync.
- `Stop` can provide an optional final check at the end of the main turn; it can keep a turn open when the hook asks the agent to complete an identified follow-up.
- A command hook receives a JSON event on standard input. Use `cwd`, `hook_event_name`, and the current repository state; do not parse transcript files as a stable API.
- Keep the hook fast and make it inspect Git change metadata, not run application tests or deploy systems.

## Recommended workflow

1. Compare staged, unstaged, and untracked paths from the repository root. Ignore docs-only changes to avoid self-trigger loops.
2. Map changed sources to likely docs: API/schema changes to contract records and System Design; WebMCP tool changes to the tool contract, frontend README, security review, and focused tests; migrations/models to the database model; integration/client changes to the producer/consumer catalog and consumer's repository link; Helm/runtime changes to operations and Kubernetes resource records; UI changes to README feature guidance and screenshot review.
3. Return a short, actionable context listing the changed source paths and documents to review. Mark consumers as `Unknown` until repositories have been inspected; never infer that no external consumer exists from a local search alone.
4. Ask the agent to inspect the authoritative source, update Markdown in the same worktree, and verify relative links, TOC anchors, headings, Mermaid blocks, and contract owner/producer/consumer details.
5. Keep documentation changes in the same reviewable change as their implementation. If the hook finds no relevant source changes, return successfully without output.

## Example project configuration

Place a project-local `hooks.json` under `.codex/` after implementing a repository-local checker. This sketch wires the same checker to agent completion and the optional main-turn final pass; the checker must handle both event payloads and return only supported fields for the current event.

```json
{
  "description": "Suggest Markdown updates after source changes",
  "hooks": {
    "SubagentStop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "python3 \"$(git rev-parse --show-toplevel)/.codex/hooks/check_doc_sync.py\"",
            "timeout": 30
          }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "python3 \"$(git rev-parse --show-toplevel)/.codex/hooks/check_doc_sync.py\"",
            "timeout": 30
          }
        ]
      }
    ]
  }
}
```

An implementation can return `hookSpecificOutput.additionalContext` for `SubagentStop`, and an event-supported continuation message for `Stop`. Keep output short: changed sources, likely docs, and unresolved ownership/consumer checks. Do not have the hook itself call an LLM, edit authoritative Markdown blindly, or trigger another agent.

## Trust and safety

Project-local hooks run only when the project `.codex/` layer is trusted. Codex requires review and trust of the exact non-managed hook definition; a changed definition needs review again. Inspect the script and configuration in `/hooks` before enabling it. Keep it read-only unless deterministic generation is intentionally chosen, make it idempotent, skip documentation-only changes, and ensure the hook command cannot recursively launch the agent or mutate deployment state.
