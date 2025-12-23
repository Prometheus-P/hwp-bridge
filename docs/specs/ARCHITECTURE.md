> Planned web assets/specs are moved to `../../future/` (see `future/README.md`).

# ARCHITECTURE.md - HwpBridge System Design

> **Version:** 1.0.0
> **Author:** @Architect
> **Status:** Approved
> **Last Updated:** 2025-12-09

---

## 1. System Overview

```
┌──────────────────────────────────────────────────────────────────────────┐
│                           HwpBridge System (Option A)                    │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐                  │
│  │   hwp-cli   │    │   hwp-wasm  │    │   hwp-mcp   │  ← Applications  │
│  │  (Binary)   │    │  (WASM)     │    │  (Stdio)    │                  │
│  └──────┬──────┘    └──────┬──────┘    └──────┬──────┘                  │
│         │                  │                  │                          │
│         └──────────────────┼──────────────────┘                          │
│                            │                                             │
│                    ┌───────▼───────┐                                     │
│                    │   hwp-core    │  ← Core Library (Parser/Converter)  │
│                    └───────┬───────┘                                     │
│                            │                                             │
│                    ┌───────▼───────┐                                     │
│                    │  hwp-types    │  ← Types / Tags / Structured model  │
│                    └───────────────┘                                     │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Crate Architecture

### 2.1 Dependency Graph

```
hwp-types (v0.1.0)
    │
    │  ← HwpError, FileHeader, StructuredDocument, tags, styles, etc.
    │
hwp-core (v0.1.0)
    │
    │  ← OLE2 + parser + structured converter
    │
    ├───────────────┬───────────────┬───────────────┐
    │               │               │               │
hwp-cli          hwp-wasm         hwp-mcp
(v0.1.0)         (v0.1.0)         (v0.1.0)
```

### 2.2 Module Structure

```
hwp-bridge/
├── Cargo.toml
├── Dockerfile
├── docker-compose.yml
├── crates/
│   ├── hwp-types/       # shared types / tags / structured model
│   │   └── src/
│   ├── hwp-core/        # parser + converter
│   │   └── src/
│   ├── hwp-cli/         # CLI entry
│   │   └── src/main.rs
│   ├── hwp-wasm/        # WASM bindings (browser/JS)
│   │   └── src/lib.rs
│   └── hwp-mcp/         # MCP stdio server
│       └── src/main.rs
├── docs/
│   ├── specs/
│   ├── operations/
│   └── guides/
```

---

## 3. Core Components

### 3.1 hwp-core: Parser Pipeline

```
┌─────────────────────────────────────────────────────────────────┐
│                      HWP Parsing Pipeline                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐  │
│  │  File    │───▶│   OLE    │───▶│  Header  │───▶│ Validate │  │
│  │  Input   │    │  Open    │    │  Parse   │    │ Fail-Fast│  │
│  └──────────┘    └──────────┘    └──────────┘    └──────────┘  │
│                                                       │         │
│                                                       ▼         │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐  │
│  │  Output  │◀───│ Convert  │◀───│  Parse   │◀───│  Decomp  │  │
│  │ HTML/MD  │    │ to HTML  │    │ Records  │    │  zlib    │  │
│  └──────────┘    └──────────┘    └──────────┘    └──────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 HwpOleFile API

```rust
pub struct HwpOleFile<F: Read + Seek> {
    cfb: CompoundFile<F>,
    header: FileHeader,
}

impl<F: Read + Seek> HwpOleFile<F> {
    /// Open and validate HWP file (Fail-Fast)
    pub fn open(inner: F) -> Result<Self, HwpError>;

    /// Get parsed FileHeader
    pub fn header(&self) -> &FileHeader;

    /// Read raw stream data
    pub fn read(&mut self, path: &str) -> Result<Vec<u8>, HwpError>;

    /// Read DocInfo stream
    pub fn read_doc_info(&mut self) -> Result<Vec<u8>, HwpError>;

    /// List BodyText sections
    pub fn list_sections(&self) -> Vec<String>;

    /// Read specific section
    pub fn read_section(&mut self, index: usize) -> Result<Vec<u8>, HwpError>;
}
```

### 3.3 Record Parsing (TODO)

```rust
/// HWP 레코드 헤더 (4 bytes)
pub struct RecordHeader {
    pub tag_id: u16,      // 10 bits
    pub level: u8,        // 10 bits
    pub size: u32,        // 12 bits (or extended)
}

/// 레코드 태그 종류
pub enum HwpTag {
    // DocInfo tags (HWPTAG_BEGIN = 0x10)
    DocumentProperties = 0x10,
    IdMappings = 0x11,
    BinData = 0x12,
    FaceName = 0x13,
    CharShape = 0x15,
    ParaShape = 0x19,

    // BodyText tags (0x40 ~ 0x7F)
    ParaHeader = 0x42,
    ParaText = 0x43,
    ParaCharShape = 0x44,
    ParaLineSeg = 0x45,

    // Table tags
    Table = 0x4D,
}
```

---

## 4. Web Service Architecture (planned / disabled)

> 웹 서버 아키텍처 섹션은 아래로 이동했습니다:

## 5. MCP Server Architecture

### 5.1 Protocol Flow

```
┌────────────────┐          ┌────────────────┐
│  Claude/AI     │          │    hwp-mcp     │
│  (Client)      │          │    (Server)    │
└───────┬────────┘          └───────┬────────┘
        │                           │
        │  initialize               │
        │──────────────────────────▶│
        │                           │
        │  { capabilities, tools }  │
        │◀──────────────────────────│
        │                           │
        │  tools/call               │
        │  { read_hwp_content }     │
        │──────────────────────────▶│
        │                           │
        │  { content: "..." }       │
        │◀──────────────────────────│
        │                           │
```

### 5.2 Tool Definitions

```json
{
  "tools": [
    {
      "name": "read_hwp_summary",
      "description": "HWP 문서의 메타데이터(제목, 작성자, 생성일)를 반환합니다.",
      "inputSchema": {
        "type": "object",
        "properties": {
          "file": {
            "type": "object",
            "description": "HWP payload (base64) or local path (when enabled).",
            "properties": {
              "name": { "type": "string" },
              "content": { "type": "string", "contentEncoding": "base64" },
              "path": { "type": "string" }
            },
            "oneOf": [
              { "required": ["name", "content"] },
              { "required": ["path"] }
            ]
          }
        },
        "required": ["file"]
      }
    },
    {
      "name": "read_hwp_content",
      "description": "HWP 문서의 본문을 Markdown 형식으로 반환합니다.",
      "inputSchema": {
        "type": "object",
        "properties": {
          "file": {
            "type": "object",
            "description": "HWP payload (base64) or local path (when enabled).",
            "properties": {
              "name": { "type": "string" },
              "content": { "type": "string", "contentEncoding": "base64" },
              "path": { "type": "string" }
            },
            "oneOf": [
              { "required": ["name", "content"] },
              { "required": ["path"] }
            ]
          },
          "format": {
            "type": "string",
            "enum": ["semantic-markdown", "plain"],
            "default": "semantic-markdown"
          }
        },
        "required": ["file"]
      }
    },
    {
      "name": "convert_to_gdocs",
      "description": "HWP 파일을 Google Docs로 변환하고 편집 링크를 반환합니다.",
      "inputSchema": {
        "type": "object",
        "properties": {
          "file": {
            "type": "object",
            "description": "HWP payload (base64) or local path (when enabled).",
            "properties": {
              "name": { "type": "string" },
              "content": { "type": "string", "contentEncoding": "base64" },
              "path": { "type": "string" }
            },
            "oneOf": [
              { "required": ["name", "content"] },
              { "required": ["path"] }
            ]
          }
        },
        "required": ["file"]
      }
    }
  ]
}
```

---

## 6. Data Flow

### 6.1 HWP File Processing

```
┌─────────────────────────────────────────────────────────────────┐
│                        HWP File Structure                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  HWP File (OLE Container)                                       │
│  ├── /FileHeader ────────────▶ FileHeader struct               │
│  │      (256 bytes)                version, properties          │
│  │                                                              │
│  ├── /DocInfo ───────────────▶ DocInfo records                 │
│  │      (zlib compressed)          styles, fonts, shapes        │
│  │                                                              │
│  ├── /BodyText/                                                 │
│  │   ├── Section0 ───────────▶ Section records                 │
│  │   ├── Section1                  paragraphs, text, tables     │
│  │   └── ...                                                    │
│  │                                                              │
│  ├── /BinData/ ──────────────▶ Binary resources                │
│  │   ├── BIN0001.png              images, OLE objects           │
│  │   └── ...                                                    │
│  │                                                              │
│  └── /PrvText ───────────────▶ Preview text                    │
│                                    (optional, uncompressed)     │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 6.2 Conversion Pipeline

```
HWP Binary
    │
    ▼
┌─────────────┐
│ FileHeader  │──▶ Version check, Fail-Fast
└─────────────┘
    │
    ▼
┌─────────────┐
│   DocInfo   │──▶ Style definitions (CharShape, ParaShape)
└─────────────┘
    │
    ▼
┌─────────────┐
│  BodyText   │──▶ Paragraph records with text & style refs
└─────────────┘
    │
    ▼
┌─────────────┐
│   BinData   │──▶ Embedded images (Base64)
└─────────────┘
    │
    ▼
┌─────────────┐
│ HwpDocument │──▶ Intermediate representation
└─────────────┘
    │
    ├──▶ HTML (with inline CSS)
    ├──▶ Markdown (for AI)
    └──▶ Google Docs (via API)
```

---

## 7. Error Handling Strategy

### 7.1 Error Hierarchy

```rust
pub enum HwpError {
    // I/O Layer
    Io(std::io::Error),

    // OLE Layer
    OleError(String),

    // Validation Layer (Fail-Fast)
    InvalidSignature,
    UnsupportedVersion(String),
    Encrypted,
    DistributionOnly,

    // Parse Layer
    ParseError(String),

    // External Services
    GoogleDriveError(String),
}
```

### 7.2 Error Response (Web)

```json
{
  "error": {
    "code": "ENCRYPTED_DOCUMENT",
    "message": "이 문서는 암호화되어 있어 변환할 수 없습니다.",
    "details": null
  }
}
```

---

## 8. Security Considerations

> Web 서비스(hwp-web) 관련 보안 항목은 `future/`로 이동했습니다:
> - ../../future/hwp-web/ARCHITECTURE_WEB.md


### 8.1 Input Validation

| Check | Location | Action |
|-------|----------|--------|
| File signature | hwp-core | Reject non-HWP |
| Path traversal | hwp-mcp | Sanitize paths |

### 8.2 Resource Limits

| Resource | Limit | Enforcement |
|----------|-------|-------------|
| Upload size | 10MB | Axum middleware |
| Parse timeout | 30s | Tokio timeout |
| Memory per request | 100MB | Process monitoring |
| Concurrent requests | 100 | Rate limiter |

---

## 9. ADR (Architecture Decision Records)

### ADR-001: Rust for Core Parser

**Status:** Accepted

**Context:** HWP 파싱은 바이너리 처리가 많고 성능이 중요함.

**Decision:** Rust를 선택하여 메모리 안전성과 C++ 수준의 성능 확보.

**Consequences:**
- (+) 메모리 안전성 보장
- (+) 크로스 플랫폼 단일 바이너리
- (-) 러닝 커브
- (-) 생태계 규모 (Python 대비)

---

### ADR-002: Axum for Web Framework

**Status:** Accepted

**Context:** Rust 웹 프레임워크 선택 필요.

**Decision:** Axum (tokio 기반, tower 미들웨어)

**Alternatives:** Actix-web, Warp, Rocket

**Consequences:**
- (+) tokio 생태계 통합
- (+) tower 미들웨어 재사용
- (+) 타입 안전한 extractors

---

### ADR-003: cfb Crate for OLE Parsing

**Status:** Accepted

**Context:** OLE2 Compound Document 파싱 필요.

**Decision:** `cfb` crate 사용

**Consequences:**
- (+) Pure Rust, no native deps
- (+) 안정적인 유지보수
- (-) 일부 엣지 케이스 미지원 가능

---

## 10. Future Considerations

### 10.1 Scalability

- Horizontal scaling via container orchestration
- Redis cache for parsed results
- CDN for static assets

### 10.2 Extensibility

- Plugin system for custom converters
- HWPX (OOXML) support
- Additional output formats (PDF, DOCX)

---

**Approved by:** @Architect
**Date:** 2025-12-09


## Distribution (Smithery)

- MCP server ships with `smithery.yaml` for Smithery registry deployment.
- See `docs/SMITHERY.md`.
