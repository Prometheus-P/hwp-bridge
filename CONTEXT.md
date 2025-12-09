# CONTEXT.md - HWP Bridge

> **Version:** 0.1.0
> **Last Updated:** 2025-12-09
> **Status:** Initial Development

---

## 1. Project Overview

### 1.1 What is HWP Bridge?

HWP Bridge는 한글(HWP) 문서를 파싱하고 변환하는 Rust 기반 도구입니다. CLI, Web API, MCP(Model Context Protocol) 서버 등 다양한 인터페이스를 통해 HWP 문서를 처리할 수 있습니다.

### 1.2 Problem Statement

- 한글(HWP)은 한국에서 널리 사용되는 문서 형식이나, 크로스플랫폼 지원이 제한적
- 기존 HWP 파싱 도구들은 대부분 Python/Java 기반으로 성능 제약 존재
- AI/LLM 워크플로우에서 HWP 문서 접근이 어려움

### 1.3 Solution

- Rust 기반 고성능 HWP 파서
- 다중 인터페이스 지원 (CLI, Web, MCP)
- LLM 도구 호출을 위한 MCP 서버 제공

---

## 2. Architecture

### 2.1 Workspace Structure

```
hwp-bridge/
├── Cargo.toml              # Workspace manifest
└── crates/
    ├── hwp-types/          # 공용 타입, 에러 정의
    ├── hwp-core/           # 핵심 파싱 로직
    ├── hwp-cli/            # CLI 인터페이스
    ├── hwp-web/            # Web API 서버 (Axum)
    └── hwp-mcp/            # MCP 서버
```

### 2.2 Crate Dependency Graph

```
hwp-types (base)
    ↑
hwp-core (parser)
    ↑
┌───┼───┐
│   │   │
cli web mcp
```

### 2.3 Crate Responsibilities

| Crate | Responsibility | Key Dependencies |
|-------|----------------|------------------|
| `hwp-types` | 공용 타입, 에러, 데이터 구조 | serde, thiserror |
| `hwp-core` | OLE 파싱, 스트림 추출, 문서 구조 해석 | cfb, nom, encoding_rs, flate2 |
| `hwp-cli` | 커맨드라인 인터페이스 | tokio, tracing |
| `hwp-web` | REST API 서버 | axum, tower-http |
| `hwp-mcp` | MCP 프로토콜 서버 | serde_json, tokio |

---

## 3. Technical Decisions

### 3.1 Why Rust?

- **성능:** 바이너리 파싱에서 C/C++ 수준의 성능
- **안전성:** 메모리 안전성 보장
- **크로스 플랫폼:** 단일 바이너리 배포 가능
- **WASM 지원:** 브라우저 환경 지원 가능성

### 3.2 HWP File Format

HWP 5.x는 OLE2 Compound Document Format 기반:

```
HWP File (OLE Container)
├── FileHeader          # 버전, 암호화 플래그
├── DocInfo             # 문서 설정, 스타일
├── BodyText/
│   ├── Section0        # 본문 섹션 (zlib 압축)
│   ├── Section1
│   └── ...
├── BinData/            # 임베디드 이미지/OLE
├── PrvText             # 미리보기 텍스트
└── PrvImage            # 썸네일 이미지
```

### 3.3 Key Parsing Challenges

| Challenge | Solution |
|-----------|----------|
| OLE2 Container | `cfb` crate 사용 |
| zlib 압축 | `flate2` crate 사용 |
| EUC-KR/UTF-16 인코딩 | `encoding_rs` crate 사용 |
| 바이너리 레코드 파싱 | `nom` parser combinator |
| 암호화 문서 | 지원 불가 (에러 반환) |

---

## 4. Data Models

### 4.1 Core Types (hwp-types)

```rust
// 에러 타입
pub enum HwpError {
    Io(std::io::Error),
    OleError(String),
    UnsupportedVersion(String),
    Encrypted,
    DistributionOnly,
    ParseError(String),
    GoogleDriveError(String),
}

// 문서 구조
pub struct HwpDocument {
    pub metadata: DocumentMetadata,
    pub content: String,  // TODO: Vec<Section>으로 구조화
}

pub struct DocumentMetadata {
    pub title: String,
    pub author: String,
    pub created_at: String,
    pub is_encrypted: bool,
    pub is_distribution: bool,
}

// 변환 옵션
pub struct ConvertOptions {
    pub extract_images: bool,
    pub include_comments: bool,
}
```

---

## 5. Interfaces

### 5.1 CLI (hwp-cli)

```bash
# 계획된 인터페이스
hwp-cli convert input.hwp -o output.txt
hwp-cli info input.hwp
hwp-cli extract-images input.hwp -d ./images
```

### 5.2 Web API (hwp-web)

```
POST /api/convert
  - multipart/form-data: file
  - Response: { content, metadata }

GET /api/info
  - Query: url (Google Drive URL)
  - Response: { metadata }
```

### 5.3 MCP Server (hwp-mcp)

```json
// Tools
{
  "name": "hwp_read",
  "description": "Read and parse HWP document",
  "parameters": {
    "path": "string (file path or URL)"
  }
}
```

---

## 6. Development Guidelines

### 6.1 Prerequisites

- Rust 1.85+ (2024 Edition)
- cargo

### 6.2 Build & Test

```bash
# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Run specific binary
cargo run -p hwp-cli
cargo run -p hwp-web
cargo run -p hwp-mcp
```

### 6.3 Code Style

- `cargo fmt` 적용
- `cargo clippy` 경고 해결
- 핵심 로직 단위 테스트 필수

---

## 7. Development Roadmap

### Phase 1: Core 엔진 기초 (Week 1)
- [x] 프로젝트 워크스페이스 구축
- [x] hwp-types: 기본 타입/에러 정의
- [ ] **Fail Fast 구현:** FileHeader 파싱, 암호화/배포용 문서 필터링
- [ ] **텍스트 추출:** OLE → zlib 압축 해제 → 텍스트 레코드 파싱
- [ ] hwp-cli 텍스트 추출 테스트

### Phase 2: 변환 로직 고도화 (Week 2-3)
- [ ] HTML 변환기: 문단/글자 스타일 → CSS 매핑
- [ ] 이미지 처리: BinData 추출, Base64 인코딩
- [ ] 표(Table) 변환: HWPTAG_TABLE → HTML table
- [ ] Markdown 변환기: AI 분석용 경량 변환

### Phase 3: 인터페이스 구현 (Week 4-5)
- [ ] hwp-web: Axum 기반 REST API, Google Drive 연동
- [ ] hwp-mcp: MCP 프로토콜, Tool 핸들러
- [ ] hwp-cli: 완전한 CLI 인터페이스

### Phase 4: 안정화 및 배포 (Week 6)
- [ ] 대용량 파일 스트리밍 최적화
- [ ] Docker 이미지 빌드
- [ ] GitHub Release 및 문서화

---

## 8. Risk Management

| Risk | Impact | Mitigation |
|------|--------|------------|
| 배포용 문서 | 콘텐츠 암호화로 복호화 불가 | FileHeader Bit 2 체크 → `HwpError::DistributionOnly` 반환 |
| 암호화 문서 | 전체 파일 복호화 불가 | FileHeader Bit 0 체크 → `HwpError::Encrypted` 반환 |
| 수식 호환성 | HWP 독자 스크립트 | 텍스트로 노출 또는 `[수식]` 태그 대체 |
| Google API Quota | 과도한 API 호출 제한 | 사용자 개인 OAuth 토큰으로 Quota 분산 |

---

## 8. References

### 8.1 HWP Format Specifications

- [한글과컴퓨터 HWP 포맷 문서](https://www.hancom.com/etc/hwpDownload.do)
- [HWP 5.0 파일 구조 분석](https://github.com/nicholasng99/hwp5)

### 8.2 Related Projects

- [pyhwp](https://github.com/mete0r/pyhwp) - Python HWP parser
- [hwp.js](https://github.com/nicholasng99/hwp.js) - JavaScript HWP parser

---

## 9. Glossary

| Term | Definition |
|------|------------|
| HWP | Hangul Word Processor, 한글과컴퓨터의 문서 포맷 |
| OLE | Object Linking and Embedding, MS Compound Document Format |
| CFB | Compound File Binary Format |
| MCP | Model Context Protocol, AI 도구 호출 프로토콜 |
| HWPX | HWP의 XML 기반 버전 (OOXML 유사) |

---

**Last Modified:** 2025-12-09
**Maintainer:** AI Software Factory
