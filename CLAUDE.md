# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Quick Start

**Build:** `cargo build`
**Run:** `cargo run -- <command>` or `./target/debug/locrawl <command>`
**Test:** `cargo test`
**Lint:** `cargo clippy`
**Format:** `cargo fmt`
**Single test:** `cargo test <test_name> -- --nocapture`

## Project Overview

**locrawl** is a Rust CLI tool for retrieving railway model data from manufacturer websites and webshops. It's built with:
- **clap** for CLI argument parsing with subcommands
- **colored** for cross-platform colored terminal output
- **serde**/**serde_json** for data serialization (future web scraping)
- Standard Rust ecosystem tools (anyhow for error handling, log/env_logger for logging)

## Architecture

### CLI Structure

The CLI is organized around the concept of commands, following a modular pattern:

- **`src/cli.rs`**: Defines the `Cli` struct (clap Parser) and the `Commands` enum that represents all available subcommands. This is where you define command-line interface structure.
- **`src/commands/mod.rs`**: Module declaration and public interface for commands.
- **`src/commands/<command>.rs`**: Implementation of individual commands (e.g., `info.rs`).
- **`src/main.rs`**: Entry point that parses CLI args and dispatches to the appropriate command handler.

### Adding a New Command

1. Create a new file in `src/commands/` (e.g., `src/commands/scrape.rs`)
2. Implement a public `run()` function that returns `Result<(), Box<dyn std::error::Error>>`
3. Add `pub mod scrape;` to `src/commands/mod.rs`
4. Add a variant to the `Commands` enum in `src/cli.rs` with a description
5. Add a match arm in `src/main.rs` to call `commands::<command>::run()`

**Example command signature:**
```rust
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    // implementation
    Ok(())
}
```

## Development Notes

- **Edition:** Rust 2024 (not 2021—see Cargo.toml)
- **MSRV:** Latest stable Rust (currently 1.93+)
- **Code Quality:** Use `cargo clippy` for lints; fix warnings before committing
- **Formatting:** Run `cargo fmt` before committing; CI may enforce this

## Testing

Integration tests (if added) should go in `tests/` directory and use the `assert_cmd` crate for testing CLI invocations. Example:
```rust
#[test]
fn test_info_command() {
    let mut cmd = Command::cargo_bin("locrawl").unwrap();
    cmd.arg("info");
    cmd.assert().success();
}
```

## Future Scope

This is the foundation for web scraping railway model data. Future work likely involves:
- Adding web scraping commands (scrape, parse, store data)
- Building data models for railway products
- Integrating with manufacturer websites and webshops
