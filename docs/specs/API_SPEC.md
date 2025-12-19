> ⚠️ Note (Option A): `hwp-web` is intentionally excluded/disabled in this repo snapshot.
> This document may still mention `hwp-web` as a *planned* component.

# API_SPEC.md - HwpBridge API Specification

> **Version:** 1.0.0
> **Author:** @Architect
> **Status:** Draft
> **Last Updated:** 2025-12-09

---

## 1. Overview

HwpBridge는 두 가지 인터페이스를 제공합니다:

1. **REST API** (hwp-web): HTTP 기반 웹 서비스
2. **MCP Protocol** (hwp-mcp): AI 에이전트용 Tool 인터페이스

---

## 2. Web REST API (hwp-web) — planned (disabled)

> Option A에서는 REST 웹 서버(`hwp-web`)를 포함하지 않습니다.
> 관련 문서는 아래로 이동했습니다:
> - ../../future/hwp-web/API_SPEC_REST.md

## 4. MCP Protocol (hwp-mcp)

### 4.1 Transport

- **Protocol:** JSON-RPC 2.0 over stdio
- **Encoding:** UTF-8
- **Message delimiter:** newline (`\n`)

### 4.2 Server Capabilities

```json
{
  "capabilities": {
    "tools": {}
  },
  "serverInfo": {
    "name": "hwp-mcp",
    "version": "0.1.0"
  }
}
```

---

## 5. MCP Tools

Option A의 MCP 서버(`crates/hwp-mcp`)는 **2개 도구**를 제공합니다.

> 입력은 **파일 경로가 아니라 base64 인코딩된 파일 바이트**입니다.  
> (호스트가 파일 접근 권한을 어떻게 주는지에 따라 달라지기 때문에, 서버는 경로를 신뢰하지 않습니다.)

### 5.1 `hwp.inspect`

HWP 파일의 메타데이터/통계 정보를 반환합니다.

**Input Schema:**

```json
{
  "type": "object",
  "properties": {
    "file": {
      "type": "object",
      "description": "HWP payload encoded as base64 (content) and logical name.",
      "properties": {
        "name": { "type": "string" },
        "content": {
          "type": "string",
          "description": "base64 encoded bytes",
          "contentEncoding": "base64"
        }
      },
      "required": ["name", "content"]
    }
  },
  "required": ["file"]
}
```

**Structured Output (`structured_content`) 예시:**

```json
{
  "title": "문서 제목",
  "author": "작성자",
  "created_at": "2025-01-01",
  "is_encrypted": false,
  "is_distributed": false,
  "sections": 3,
  "paragraphs": 120,
  "tables": 5
}
```

### 5.2 `hwp.to_markdown`

HWP 내용을 **(1) semantic-markdown** 또는 **(2) plain text**로 변환합니다.  
또한 `structured_content`로 `StructuredDocument`(문단/표 구조 포함)를 함께 반환합니다.

**Input Schema:**

```json
{
  "type": "object",
  "properties": {
    "file": {
      "type": "object",
      "properties": {
        "name": { "type": "string" },
        "content": { "type": "string", "contentEncoding": "base64" }
      },
      "required": ["name", "content"]
    },
    "format": {
      "type": "string",
      "enum": ["semantic-markdown", "plain"],
      "default": "semantic-markdown"
    }
  },
  "required": ["file"]
}
```

**Output:**
- `content`: 변환된 텍스트(semantic markdown 또는 plain text)
- `structured_content`: `StructuredDocument` (스키마는 내부적으로 진화 가능)

## 6. CLI Interface (hwp-cli)

### 6.1 Commands

`crates/hwp-cli`는 현재 **2개 서브커맨드**를 제공합니다.

#### `hwp extract`

HWP에서 텍스트를 추출합니다.

```bash
hwp extract <FILE> [-o <OUTPUT>] [--verbose]
```

- `<FILE>`: 입력 `.hwp`
- `-o, --output`: 결과를 파일로 저장 (미지정 시 stdout)

#### `hwp info`

HWP의 FileHeader 플래그 정보를 출력합니다.

```bash
hwp info <FILE> [--verbose]
```

### 6.2 Output Examples

```bash
hwp info sample.hwp
```

```text
HWP File Information:
  Version: 5.1.0.0
  Encrypted: false
  Compressed: true
  Distributed: false
  Has Script: false
  Has DRM: false
  Has History: false
  Has Signature: false
```

## 7. Rate Limits

Option A 범위(`hwp-cli`, `hwp-mcp`, `hwp-wasm`)에는 **내장 rate limit 정책이 없습니다.**
- `hwp-cli`: 로컬 실행 도구 → rate limit 개념 없음
- `hwp-mcp`: 호스트 애플리케이션(IDE/Agent/서버)에서 호출 빈도를 제어
- `hwp-wasm`: 브라우저/호스트에서 호출 빈도를 제어

> 웹 API의 rate limit 헤더/정책/SDK 예시는 `hwp-web` 복구 시 적용하세요:
> - ../../future/hwp-web/API_SPEC_REST.md
