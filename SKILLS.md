# SKILLS (for AI Agents)

This file defines expected professional behavior for AI agents working in this repository.

## 1. Mission

Deliver small, correct, production-minded changes while preserving architecture and security standards.

## 2. Core skills expected

### Codebase comprehension
- Understand module boundaries before editing.
- Prefer existing patterns over introducing new ones.

### Safe implementation
- Make minimal, scoped changes.
- Avoid touching unrelated code.
- Keep compatibility with current APIs and migrations.

### Validation discipline
- Run existing checks before and after edits.
- Report failures with root-cause, not only symptoms.

### Documentation quality
- Keep docs synchronized with behavior.
- Document tradeoffs and assumptions clearly.

## 3. Repository-specific conventions

- Use `AppModule` structure for domains.
- Keep business logic in services, data access in repositories.
- Keep middleware generic and reusable.
- Use `ApiResponse` shape for consistent API output.
- Prefer typed errors through `AppError` (or module-specific mapped errors).

## 4. Editing checklist for AI agents

Before edits:
- read relevant files fully
- identify affected layers
- identify any migration/schema implications

During edits:
- keep changes cohesive
- preserve naming consistency (DB columns vs code fields)
- avoid placeholder logic unless explicitly requested

After edits:
- run `cargo check`
- run targeted tests for touched areas
- update docs if architecture/behavior changed

## 5. Communication style

- concise and direct
- explicitly state what changed and why
- list risks or unresolved items
- do not hide uncertainty

## 6. Security and compliance guardrails

- never commit secrets
- never weaken auth/security middleware without explicit request
- do not add unsafe dependencies unnecessarily
- avoid leaking internal details in user-facing errors

## 7. Preferred contribution quality bar

A change is considered high quality when:
- it compiles and passes relevant tests
- it fits existing architecture
- it is readable and maintainable
- it improves, not degrades, operational safety

