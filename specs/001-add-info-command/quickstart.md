# Quickstart: locrawl CLI

**Date**: 2026-02-06
**Feature**: [specs/001-add-info-command/spec.md](specs/001-add-info-command/spec.md)

## Prerequisites

- Rust 1.70+ installed
- Cargo package manager

## Installation

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd locrawl
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. The binary will be available at `target/release/locrawl`

## Usage

### Info Command
Display basic information about the tool:

```bash
./target/release/locrawl info
```

Expected output:
```
locrawl v1.0.0
A CLI tool for retrieving railway model data from manufacturer websites and webshops.
```

### Help
Get help for any command:

```bash
./target/release/locrawl --help
./target/release/locrawl info --help
```

## Development

### Run tests
```bash
cargo test
```

### Run lints
```bash
cargo clippy
```

### Format code
```bash
cargo fmt
```

### Development build
```bash
cargo build
```

## Troubleshooting

- If colors don't display on Windows, ensure terminal supports ANSI colors or virtual terminal processing is enabled
- For build issues, ensure Rust toolchain is up to date: `rustup update`</content>
<parameter name="filePath">/home/carlo/Projects/locrawl/specs/001-add-info-command/quickstart.md