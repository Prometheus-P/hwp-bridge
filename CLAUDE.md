# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

HWP Bridge is a Rust-based toolkit for parsing and converting HWP (한글) documents. It provides:
- **hwp-cli**: Command-line interface for local conversion & inspection
- **hwp-mcp**: MCP (Model Context Protocol) server for AI clients (Claude Desktop, etc.)
- **hwp-wasm**: WebAssembly bindings
- **hwp-gui**: Tauri-based desktop GUI

> Note: `hwp-web` is disabled and parked in `/future/`.

## Build & Development Commands

```bash
# Build
cargo build --workspace              # Debug build
cargo build --workspace --release    # Release build

# Test
cargo test --workspace               # Run all tests
cargo test -p hwp-core               # Test specific crate
cargo test test_name                 # Run single test by name

# Lint & Format (required before commit)
cargo fmt --all                      # Format code
cargo clippy --workspace -- -D warnings  # Lint with warnings as errors

# Run binaries
cargo run -p hwp-cli -- --help                    # CLI help
cargo run -p hwp-cli -- info ./sample.hwp         # Inspect HWP file
cargo run -p hwp-cli -- extract ./sample.hwp -o out.txt  # Extract text
cargo run -p hwp-mcp                              # MCP server (stdio)
HWP_MCP_TRANSPORT=http PORT=8081 cargo run -p hwp-mcp  # MCP server (HTTP)
```

## Crate Architecture

```
hwp-types (base)  ──  Types, errors, data structures
    ↓
hwp-core (parser) ──  OLE parsing, decompression, document structure
    ↓
┌───┴───┬───────┐
cli    mcp    wasm  ──  Thin wrappers exposing hwp-core
```

**Key constraints:**
- All parsing/conversion logic lives in `hwp-core` — application crates are thin wrappers only
- No circular dependencies between crates
- `hwp-types` contains only type definitions and error types (no logic)

### hwp-core Internal Structure

```
hwp-core/src/
├── parser/
│   ├── ole.rs         # OLE/CFB container handling
│   ├── header.rs      # FileHeader parsing (version, encryption flags)
│   ├── section.rs     # Section stream parsing
│   ├── record.rs      # Record-based binary format parsing
│   ├── docinfo/       # Document info stream parsers
│   └── bodytext/      # Body text and table parsing
└── converter/         # Output format converters (markdown, structured JSON)
```

## HWP File Format Notes

HWP 5.x uses OLE2 Compound Document Format:
- **FileHeader**: Version info, encryption/distribution flags (Fail-fast: check first!)
- **DocInfo**: Document settings, styles (zlib compressed)
- **BodyText/SectionN**: Main content sections (zlib compressed)
- **BinData/**: Embedded images and OLE objects
- **PrvText/PrvImage**: Preview text and thumbnail

Critical error handling:
- Encrypted documents → `HwpError::Encrypted` (immediate return)
- Distribution-only documents → `HwpError::DistributionOnly` (immediate return)
- HWPX (.hwpx) is **not supported** — only HWP v5 (.hwp, OLE/CFB format)

## Git Workflow

- **Never push directly to main** — use feature branches
- Branch naming: `feature/`, `fix/`, `refactor/`, `docs/`, `test/`, `chore/`

### Pre-commit checklist

```bash
cargo test --workspace
cargo fmt --all
cargo clippy --workspace -- -D warnings
```

### Commit message format

```
<type>(<scope>): <subject>
```

Types: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`
Scopes: `types`, `core`, `cli`, `mcp`, `wasm`

**커밋 메시지에 AI/Claude 언급 금지**

## TDD Requirements

This project follows strict TDD:

1. **RED**: Write failing test first
2. **GREEN**: Minimum implementation to pass
3. **REFACTOR**: Clean up while keeping tests green
4. **COMMIT**: Only after all tests pass

```rust
#[test]
fn test_should_{expected}_when_{condition}() {
    // Arrange
    // Act
    // Assert
}
```

**Rules:**
- `hwp-core` test coverage must be ≥80%
- No `.unwrap()` / `.expect()` in library code — use proper error handling
- Separate "tidy" commits (refactoring) from "behavior" commits (features)

## MCP Tools

The MCP server exposes these tools:
- `hwp.inspect` — Parse and return structured summary + metadata
- `hwp.to_markdown` — Convert to semantic markdown
- `hwp.extract` — Fast path for plain text extraction
- `hwp.to_json` — Structured JSON output

## Quality Gate

The project uses a private corpus for testing. Put HWP files in `corpus/local/` (gitignored):

```bash
cargo build --release -p hwp-cli
python3 scripts/corpus_scan.py
python3 scripts/v1_gate.py --ci --min-corpus-size 100
```

## Key Dependencies

| Crate | Purpose |
|-------|---------|
| `cfb` | OLE2/CFB container parsing |
| `flate2` | zlib decompression |
| `encoding_rs` | EUC-KR/UTF-16 encoding conversion |
| `nom` | Binary record parsing (parser combinators) |
| `mcp-sdk-rs` | MCP protocol implementation |
| `axum` | HTTP server for MCP HTTP transport |
