# Feature Specification: Add Info Command to locrawl CLI

**Feature Branch**: `001-add-info-command`  
**Created**: 2026-02-06  
**Status**: Draft  
**Input**: User description: "locrawl is CLI tool which allows users to retrieve railway model data from manufacturers websites or webshops.

The first command we will like to offer is the "info" command which returns

* command name ("locrawl") , the current version
* a short summary of the command"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Display Basic Tool Information (Priority: P1)

As a user of the locrawl CLI tool, I want to run an "info" command to quickly see the tool's name, current version, and a brief description of what it does, so I can confirm I'm using the right tool and understand its purpose.

**Why this priority**: This is the first command and provides essential discovery and verification functionality for users.

**Independent Test**: Can be fully tested by running "locrawl info" and verifying the output contains the expected name, version, and summary. This delivers immediate value as a basic CLI tool introduction.

**Acceptance Scenarios**:

1. **Given** locrawl CLI is installed and accessible, **When** user runs `locrawl info`, **Then** displays "locrawl" as the command name
2. **Given** locrawl CLI is installed and accessible, **When** user runs `locrawl info`, **Then** displays the current version (e.g., "v1.0.0")
3. **Given** locrawl CLI is installed and accessible, **When** user runs `locrawl info`, **Then** displays a short summary describing the tool's purpose
4. **Given** locrawl CLI is installed and accessible, **When** user runs `locrawl info --help`, **Then** shows help text explaining the info command

---

### Edge Cases

- What happens when the version is not set or available?
- How does the command handle invalid arguments or flags?
- What if the tool is run in an environment where output formatting is limited?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide an "info" subcommand accessible via `locrawl info`
- **FR-002**: System MUST display "locrawl" as the command name when info command is executed
- **FR-003**: System MUST display the current version of the tool when info command is executed
- **FR-004**: System MUST display a short summary describing the tool's purpose (retrieving railway model data from manufacturer websites/webshops)
- **FR-005**: System MUST support `--help` flag for the info command to show usage information

### Non-Functional Requirements

- **NFR-001**: Performance budgets MUST be defined for critical paths (e.g., p95 latency under 100ms for info command execution)
- **NFR-002**: UX consistency MUST follow shared patterns for CLI output formatting and error messages
- **NFR-003**: Accessibility expectations MUST be specified for command-line interface (clear output, proper exit codes)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can run `locrawl info` and see all required information (name, version, summary) within 2 seconds
- **SC-002**: Info command executes with 100% success rate in standard environments
- **SC-003**: Command output is readable and properly formatted for terminal display
- **SC-004**: Help functionality works and provides clear usage guidance

## Key Entities *(include if feature involves data)*

No data entities required for this basic info command feature.
