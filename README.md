# locrawl

Command-line tooling for importing railway model datasets into a unified manifest.

`locrawl` validates source JSON payloads against per-domain schemas, merges them into a manifest, and validates the final manifest output before writing it.

## Features

- Schema-validated imports for collection, digital roster, track, and wishlist datasets.
- Safe merge behavior with explicit conflict handling.
- Optional overwrite mode for conflicts with `--force`.
- Atomic writes for manifest output files.

## Installation

### From source

Ensure you have the [Rust toolchain](https://rustup.rs/) installed:

```bash
git clone https://github.com/CarloMicieli/locrawl.git
cd locrawl
cargo install --path .
```

## Usage

Show available commands:

```bash
locrawl --help
```

Display tool information:

```bash
locrawl info
```

Import a collection payload into a manifest:

```bash
locrawl import-collection \
	--source ./samples/collection.json \
	--output ./manifest.json
```

Import digital roster assignments into a manifest:

```bash
locrawl import-digital-roster \
	--source ./samples/digital_roster.json \
	--output ./manifest.json
```

Import track products and inventories into a manifest:

```bash
locrawl import-track \
	--source ./samples/track_import.json \
	--output ./manifest.json
```

Import a wishlist into a manifest:

```bash
locrawl import-wishlist \
	--source ./samples/wishlist.json \
	--output ./manifest.json
```

Overwrite conflicting entries during merge:

```bash
locrawl import-track \
	--source ./samples/track_import.json \
	--output ./manifest.json \
	--force
```

## Command reference

| Command | Description |
| :--- | :--- |
| `info` | Display basic tool information. |
| `import-collection` | Import collection data into a manifest file. |
| `import-digital-roster` | Import digital roster data into a manifest file. |
| `import-track` | Import track products and inventories into a manifest file. |
| `import-wishlist` | Import wishlist data into a manifest file. |

## Import options

All import commands support:

- `-s, --source <PATH>`: path to source JSON.
- `-o, --output <PATH>`: path to manifest JSON to create or update.
- `-f, --force`: overwrite conflicting existing entries.

## Validation workflow

For each import command:

1. Input JSON is validated against its domain schema in `schema/`.
2. Data is merged into an existing manifest or a new empty manifest.
3. Output is validated against `schema/manifest_schema.json`.
4. The manifest is written atomically to the output path.

## Contributing

1. Fork the project
2. Create your branch (`git checkout -b feature/my-change`)
3. Commit changes (`git commit -m "Describe change"`)
4. Push branch (`git push origin feature/my-change`)
5. Open a pull request