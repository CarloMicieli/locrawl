# Research: Add Info Command to locrawl CLI

**Date**: 2026-02-06
**Feature**: [specs/001-add-info-command/spec.md](specs/001-add-info-command/spec.md)

## Research Tasks

### 1. Best practices for Rust CLI with clap subcommands
**Task**: Research clap best practices for implementing subcommands in Rust CLI applications.

**Findings**:
- Use clap 4.x with derive API for type-safe command definitions
- Organize commands in separate modules (commands/info.rs)
- Use Subcommand enum for main command routing
- Implement Display trait for consistent output formatting
- Handle --help automatically via clap

**Decision**: Use clap 4.x derive API with Subcommand enum and modular command structure.

**Rationale**: Provides compile-time safety, automatic help generation, and clean separation of concerns.

**Alternatives considered**:
- Clap builder API: More verbose, less type-safe
- Structopt (deprecated): No longer maintained, clap derive is preferred

### 2. Cross-platform colored terminal output
**Task**: Research crates for colored terminal output that work reliably on Linux and Windows.

**Findings**:
- `colored` crate: Simple API, supports Windows via winapi, good ANSI fallback
- `termcolor` crate: More control but complex API, also cross-platform
- `ansi_term` crate: Good but less maintained than colored
- Windows support requires enabling virtual terminal processing or winapi calls

**Decision**: Use `colored` crate for simplicity and cross-platform support.

**Rationale**: Simple API, reliable Windows support, good performance, active maintenance.

**Alternatives considered**:
- `termcolor`: Overkill for simple colored output needs
- `ansi_term`: Less active maintenance, colored has better Windows support

### 3. Version handling in Rust CLI binaries
**Task**: Research best practices for displaying version information in Rust CLI tools.

**Findings**:
- Use `env!("CARGO_PKG_VERSION")` for compile-time version from Cargo.toml
- Clap provides automatic --version flag when version is set
- Can combine with build info for git commit hashes
- Display format: "locrawl v1.0.0"

**Decision**: Use `env!("CARGO_PKG_VERSION")` with clap's version feature.

**Rationale**: Automatic, reliable, integrates well with clap's help system.

**Alternatives considered**:
- Manual version constants: Error-prone, requires updates
- Build scripts: Unnecessary complexity for simple version display

## Summary

Technical approach confirmed: Rust + clap derive + colored crate + built-in version handling. All choices prioritize simplicity, cross-platform compatibility, and maintainability.</content>
<parameter name="filePath">/home/carlo/Projects/locrawl/specs/001-add-info-command/research.md