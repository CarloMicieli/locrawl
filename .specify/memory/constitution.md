<!--
Sync Impact Report
- Version change: template → 1.0.0
- Modified principles: N/A (initial constitution)
- Added sections: Engineering Standards, Development Workflow
- Removed sections: None
- Templates requiring updates:
  - ✅ updated: .specify/templates/plan-template.md
  - ✅ updated: .specify/templates/spec-template.md
  - ✅ updated: .specify/templates/tasks-template.md
  - ⚠️ pending: .specify/templates/commands/*.md (directory not found)
- Follow-up TODOs: TODO(RATIFICATION_DATE) pending historical adoption date
-->
# locrawl Constitution

## Core Principles

### I. Code Quality & Maintainability
All production code MUST be readable, consistent, and easy to change.
Code MUST follow repository lint/format rules, use clear naming, and avoid
unnecessary complexity. Functions and modules MUST be kept small and focused;
introduce abstraction only when it reduces duplication or improves clarity.
Public interfaces MUST be documented where usage is non-obvious.

### II. Testing Standards (NON-NEGOTIABLE)
All new or changed behavior MUST be covered by automated tests. Bug fixes MUST
include a regression test. Unit tests are required for core logic; integration
tests are required for cross-component behavior and critical user journeys.
Tests MUST be deterministic, run in CI, and not be skipped without a tracked
issue and explicit approval.

### III. UX Consistency
User experience MUST be consistent across screens, flows, and states.
UI components MUST use shared patterns, tokens, and behaviors to avoid drift.
Error, loading, and empty states MUST follow the same interaction and copy
guidelines. Any UX change MUST be validated against the feature spec's
acceptance scenarios.

### IV. Performance Requirements
Performance budgets MUST be defined in the feature spec or plan before
implementation. Changes MUST meet those budgets and MUST NOT regress key
metrics (latency, throughput, memory, or frame rate where applicable). If a
budget cannot be met, the change MUST include a documented mitigation plan and
explicit approval.

### V. Quality Gates & Review Discipline
No change may merge without passing quality gates. At minimum this includes
lint/format checks, required automated tests, and any defined UX or performance
checks. Every change MUST be reviewed for compliance with these principles; a
review that does not address them is incomplete.

## Engineering Standards

- CI MUST run linting, formatting, and all required tests on every change.
- Accessibility checks MUST be executed for UI features when applicable.
- Performance checks/profiling MUST be executed when budgets are defined.
- Any waiver of a gate MUST be documented with scope, risk, and follow-up work.

## Development Workflow

- Feature work MUST include an updated spec and plan when scope changes.
- Pull requests MUST link to the relevant spec/plan and state which budgets
	and tests were executed.
- Releases MUST include notes for user-facing changes and performance impacts.

## Governance
<!-- Example: Constitution supersedes all other practices; Amendments require documentation, approval, migration plan -->

Amendments require a documented proposal, rationale, and impact analysis in a
pull request. Approval requires consensus from maintainers or the designated
project owner. Versioning follows semantic versioning: MAJOR for removals or
backward-incompatible governance changes, MINOR for new or expanded principles,
PATCH for clarifications or wording-only updates. Every review MUST explicitly
verify compliance with Core Principles, Engineering Standards, and Development
Workflow. The constitution supersedes all other guidance.

**Version**: 1.0.0 | **Ratified**: TODO(RATIFICATION_DATE): historical adoption date unknown | **Last Amended**: 2026-02-06
