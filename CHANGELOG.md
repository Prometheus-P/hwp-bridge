# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-12-31

### Added
- **hwp-types**: Core type definitions and error types for HWP parsing
- **hwp-core**: HWP v5 parser with OLE/CFB container support
  - FileHeader parsing with version and encryption detection
  - DocInfo stream parsing (styles, fonts, paragraph shapes)
  - BodyText section parsing with zlib decompression
  - Table structure parsing
  - Text extraction to plain text and markdown
  - Structured JSON output
- **hwp-cli**: Command-line interface
  - `info` command for document inspection
  - `extract` command for text extraction
- **hwp-mcp**: MCP (Model Context Protocol) server
  - `hwp.inspect` - Parse and return structured summary
  - `hwp.to_markdown` - Convert to semantic markdown
  - `hwp.extract` - Fast path for plain text extraction
  - `hwp.to_json` - Structured JSON output
  - HTTP transport support for Smithery/container deployment
- **hwp-wasm**: WebAssembly bindings for browser usage

### Known Limitations
- HWPX format (.hwpx) is not supported
- Encrypted documents return `HwpError::Encrypted`
- Distribution-only documents return `HwpError::DistributionOnly`

[0.1.0]: https://github.com/x-ordo/hwp-bridge/releases/tag/v0.1.0
