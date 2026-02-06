# Data Model: Add Info Command to locrawl CLI

**Date**: 2026-02-06
**Feature**: [specs/001-add-info-command/spec.md](specs/001-add-info-command/spec.md)

## Overview

No persistent data model is required for this feature. The info command is a stateless CLI operation that displays static information about the tool.

## Entities

None - this is a simple command-line interface feature with no data persistence or complex state management.

## Notes

- Version information comes from Cargo.toml at compile time
- Command name and summary are hardcoded strings
- No database or file storage needed</content>
<parameter name="filePath">/home/carlo/Projects/locrawl/specs/001-add-info-command/data-model.md