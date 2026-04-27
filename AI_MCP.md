# AI MCP Guide

Guidance for AI tools and MCP-enabled agents interacting with this repository.

## 1. Purpose

Define a reliable, auditable workflow for AI-assisted development using MCP/tooling interfaces.

## 2. MCP usage principles

- **Read first, then write**: inspect all relevant files before changing anything.
- **Small batches**: prefer incremental commits and progress updates.
- **Tool-grounded claims**: do not claim success without check output.
- **No speculative refactors** without explicit requirement.

## 3. Required execution pattern

1. Discover context (`view`, `glob`, `rg`)
2. Validate baseline (`cargo check`, targeted tests)
3. Plan minimal changes
4. Implement with tight scope
5. Re-validate (`cargo check`, targeted tests)
6. Summarize outcomes + risks

## 4. CI and workflow policy

When CI/build/test failures are reported:
- inspect workflow runs and failing jobs
- use logs to identify root cause
- patch only related failures first
- re-run local validations

## 5. Documentation policy for AI edits

If architecture, behavior, or operations changed, update:
- `README.md` (user-facing summary)
- `ARCHITECTURE.md` (technical model)
- relevant docs under `/docs`

For agent-process changes, update:
- `SKILLS.md`
- `AI_MCP.md`

## 6. Safety constraints

- Never exfiltrate repository data.
- Never introduce credentials into source/docs.
- Avoid destructive commands unless explicitly required.
- Preserve existing branch/PR workflow.

## 7. Definition of done for AI contributions

- Change request satisfied
- Local validation performed and reported
- Documentation aligned
- Remaining warnings/risks explicitly disclosed

