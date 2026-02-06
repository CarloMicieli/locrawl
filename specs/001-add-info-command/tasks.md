# Tasks: Add Info Command to locrawl CLI

**Input**: Design documents from `/specs/001-add-info-command/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: The examples below include test tasks. Tests are REQUIRED for all new or changed behavior.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/` at repository root
- **Web app**: `backend/src/`, `frontend/src/`
- **Mobile**: `api/src/`, `ios/src/` or `android/src/`
- Paths shown below assume single project - adjust based on plan.md structure

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [x] T001 Create Cargo.toml with clap and colored dependencies
- [x] T002 Initialize src/main.rs with basic application structure
- [x] T003 Create tests/ directory structure (unit/, integration/)
- [x] T004 Verify GitHub Actions workflows are configured (.github/workflows/ci.yml, update_dep.yml)

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core CLI infrastructure that MUST be complete before user story implementation

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T005 Implement CLI argument parsing structure in src/cli.rs using clap derive
- [x] T006 Setup command routing in src/main.rs
- [x] T007 Create src/commands/ directory for command modules
- [x] T008 Add error handling infrastructure for CLI operations

**Checkpoint**: CLI foundation ready - user story implementation can now begin

## Phase 3: User Story 1 - Display Basic Tool Information (Priority: P1) 🎯 MVP

**Goal**: Implement the info command that displays command name, version, and summary

**Independent Test**: Can be fully tested by running "locrawl info" and verifying output contains name, version, and summary

### Tests for User Story 1 (REQUIRED) ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T009 [P] [US1] Unit test for info command output formatting in tests/unit/commands/test_info.rs
- [x] T010 [P] [US1] Integration test for end-to-end "locrawl info" execution in tests/integration/test_cli_info.rs

### Implementation for User Story 1

- [x] T011 [P] [US1] Create info command module in src/commands/info.rs
- [x] T012 [US1] Implement info command logic with version and summary display (depends on T011)
- [x] T013 [US1] Integrate info command into main CLI routing (depends on T012)
- [x] T014 [US1] Add colored output support for command name display
- [x] T015 [US1] Add help text for info command

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational phase completion

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories

### Within Each User Story

- Tests (T009, T010) MUST be written and FAIL before implementation
- Command module (T011) before command logic (T012)
- Logic before integration (T013)
- Integration before polish (T014, T015)
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks (T001-T004) can run in parallel
- All Foundational tasks (T005-T008) can run in parallel (within Phase 2)
- Tests for User Story 1 (T009, T010) can run in parallel
- Command module creation (T011) can run in parallel with test writing

### Within User Story 1

```bash
# Launch tests together:
Task: "Unit test for info command output formatting in tests/unit/commands/test_info.rs"
Task: "Integration test for end-to-end "locrawl info" execution in tests/integration/test_cli_info.rs"

# Launch foundational setup together:
Task: "Implement CLI argument parsing structure in src/cli.rs using clap derive"
Task: "Setup command routing in src/main.rs"
Task: "Create src/commands/ directory for command modules"
Task: "Add error handling infrastructure for CLI operations"
```

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks user story)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Test User Story 1 independently
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP!)

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 implementation
   - Developer B: User Story 1 tests
3. Story complete and integrate independently

## Notes

- [P] tasks = different files, no dependencies
- [US1] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence</content>
<parameter name="filePath">/home/carlo/Projects/locrawl/specs/001-add-info-command/tasks.md