# CLI Contract: locrawl Info Command

**Version**: 1.0.0
**Date**: 2026-02-06
**Feature**: [specs/001-add-info-command/spec.md](specs/001-add-info-command/spec.md)

## Command Interface

### Command Structure
```
locrawl info [OPTIONS]
```

### Options
- `--help, -h`: Display help information
- `--version, -V`: Display version information (inherited from clap)

### Output Format
```
locrawl v1.0.0
A CLI tool for retrieving railway model data from manufacturer websites and webshops.
```

### Exit Codes
- `0`: Success
- `1`: Error (invalid arguments, etc.)

### Colored Output
When terminal supports colors:
- Command name: colored (if configured)
- Version: standard color
- Summary: standard color

### Examples
```bash
# Basic usage
locrawl info

# Help
locrawl info --help

# Version (via clap)
locrawl --version
```

## Implementation Notes
- Uses clap for argument parsing
- Colored output via `colored` crate
- Version from `env!("CARGO_PKG_VERSION")`
- Cross-platform compatible (Linux/Windows)</content>
<parameter name="filePath">/home/carlo/Projects/locrawl/specs/001-add-info-command/contracts/cli-contract.md