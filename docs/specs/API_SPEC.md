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

## 2. REST API (hwp-web)

### 2.1 Base URL

```
Development: http://localhost:3000
Production:  https://api.hwpbridge.io
```

### 2.2 Common Headers

| Header | Value | Required |
|--------|-------|----------|
| Content-Type | application/json or multipart/form-data | Yes |
| Authorization | Bearer {token} | For Google API |
| X-Request-ID | UUID | Optional |

### 2.3 Error Response Format

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable message",
    "details": { }
  }
}
```

### 2.4 Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| INVALID_FILE | 400 | 유효하지 않은 HWP 파일 |
| FILE_TOO_LARGE | 413 | 파일 크기 초과 (10MB) |
| ENCRYPTED_DOCUMENT | 422 | 암호화된 문서 |
| DISTRIBUTION_DOCUMENT | 422 | 배포용 문서 |
| UNSUPPORTED_VERSION | 422 | 지원하지 않는 HWP 버전 |
| PARSE_ERROR | 500 | 파싱 중 오류 발생 |
| GOOGLE_API_ERROR | 502 | Google API 오류 |
| RATE_LIMITED | 429 | 요청 한도 초과 |

---

## 3. Endpoints

### 3.1 Health Check

```
GET /health
```

**Response:**

```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_seconds": 3600
}
```

---

### 3.2 Convert HWP to HTML

```
POST /api/convert
```

**Request:**

```
Content-Type: multipart/form-data

file: (binary) HWP file
format: "html" | "markdown" (default: "html")
```

**Response (200 OK):**

```json
{
  "success": true,
  "data": {
    "content": "<html>...",
    "metadata": {
      "title": "문서 제목",
      "author": "작성자",
      "created_at": "2025-01-01T00:00:00Z",
      "version": "5.1.0.0",
      "page_count": 10
    },
    "images": [
      {
        "id": "BIN0001",
        "format": "png",
        "data_base64": "iVBORw0KGgo..."
      }
    ]
  }
}
```

**Error Response (422):**

```json
{
  "error": {
    "code": "ENCRYPTED_DOCUMENT",
    "message": "이 문서는 암호화되어 있어 변환할 수 없습니다."
  }
}
```

---

### 3.3 Get Document Info

```
GET /api/info
```

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| url | string | Yes | HWP 파일 URL (Google Drive 등) |

**Response (200 OK):**

```json
{
  "success": true,
  "data": {
    "filename": "document.hwp",
    "size_bytes": 1048576,
    "metadata": {
      "title": "문서 제목",
      "author": "작성자",
      "created_at": "2025-01-01T00:00:00Z",
      "version": "5.1.0.0",
      "is_encrypted": false,
      "is_distribution": false,
      "is_compressed": true
    }
  }
}
```

---

### 3.4 Convert to Google Docs

```
POST /api/convert/gdocs
```

**Request:**

```
Content-Type: multipart/form-data
Authorization: Bearer {google_oauth_token}

file: (binary) HWP file
folder_id: (optional) Google Drive folder ID
```

**Response (200 OK):**

```json
{
  "success": true,
  "data": {
    "docs_url": "https://docs.google.com/document/d/...",
    "docs_id": "1ABC...",
    "title": "문서 제목"
  }
}
```

---

### 3.5 OAuth Callback (Google)

```
GET /auth/google/callback
```

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| code | string | Authorization code |
| state | string | CSRF token |

**Response:** Redirect to frontend with token

---

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

### 5.1 read_hwp_summary

HWP 문서의 메타데이터를 반환합니다.

**Input Schema:**

```json
{
  "type": "object",
  "properties": {
    "path": {
      "type": "string",
      "description": "HWP 파일의 절대 경로"
    }
  },
  "required": ["path"]
}
```

**Output:**

```json
{
  "title": "문서 제목",
  "author": "작성자",
  "created_at": "2025-01-01",
  "version": "5.1.0.0",
  "page_count": 10,
  "is_encrypted": false,
  "is_distribution": false
}
```

**Error:**

```json
{
  "error": "ENCRYPTED_DOCUMENT",
  "message": "이 문서는 암호화되어 있어 읽을 수 없습니다."
}
```

---

### 5.2 read_hwp_content

HWP 문서의 본문을 Markdown 형식으로 반환합니다.

**Input Schema:**

```json
{
  "type": "object",
  "properties": {
    "path": {
      "type": "string",
      "description": "HWP 파일의 절대 경로"
    },
    "max_length": {
      "type": "integer",
      "description": "최대 문자 수 (기본값: 10000)",
      "default": 10000
    },
    "include_images": {
      "type": "boolean",
      "description": "이미지 포함 여부 (Base64)",
      "default": false
    }
  },
  "required": ["path"]
}
```

**Output:**

```json
{
  "content": "# 문서 제목\n\n본문 내용...",
  "truncated": false,
  "total_length": 5000
}
```

---

### 5.3 convert_to_gdocs

HWP 파일을 Google Docs로 변환하고 편집 링크를 반환합니다.

**Input Schema:**

```json
{
  "type": "object",
  "properties": {
    "path": {
      "type": "string",
      "description": "HWP 파일의 절대 경로"
    },
    "title": {
      "type": "string",
      "description": "Google Docs 문서 제목 (선택)"
    }
  },
  "required": ["path"]
}
```

**Output:**

```json
{
  "docs_url": "https://docs.google.com/document/d/...",
  "docs_id": "1ABC...",
  "title": "변환된 문서"
}
```

**Note:** Google OAuth 인증이 필요합니다. 환경 변수 또는 설정 파일에서 토큰을 읽습니다.

---

## 6. CLI Interface (hwp-cli)

### 6.1 Commands

```bash
# 문서 정보 출력
hwp-cli info <file.hwp>
  --json          # JSON 형식 출력

# 텍스트 변환
hwp-cli convert <file.hwp> -o <output>
  --format html|markdown|text
  --include-images
  --no-styles

# 이미지 추출
hwp-cli extract-images <file.hwp> -d <directory>
  --format png|original

# 버전 정보
hwp-cli --version
```

### 6.2 Exit Codes

| Code | Description |
|------|-------------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments |
| 10 | File not found |
| 11 | Invalid HWP file |
| 12 | Encrypted document |
| 13 | Distribution document |
| 14 | Unsupported version |
| 20 | Parse error |

### 6.3 Output Examples

**Info Command:**

```bash
$ hwp-cli info document.hwp

╔════════════════════════════════════════╗
║  HWP Document Info                     ║
╠════════════════════════════════════════╣
║  Title:      문서 제목                 ║
║  Author:     작성자                    ║
║  Version:    5.1.0.0                   ║
║  Created:    2025-01-01                ║
║  Encrypted:  No                        ║
║  Compressed: Yes                       ║
║  Pages:      10                        ║
╚════════════════════════════════════════╝
```

**Convert Command:**

```bash
$ hwp-cli convert document.hwp -o output.html --format html

✓ Parsed document.hwp (5.1.0.0)
✓ Extracted 3 images
✓ Converted to HTML
✓ Saved to output.html (125 KB)
```

---

## 7. Rate Limits

### 7.1 Web API

| Endpoint | Limit | Window |
|----------|-------|--------|
| /api/convert | 10 req | 1 min |
| /api/info | 30 req | 1 min |
| /api/convert/gdocs | 5 req | 1 min |

### 7.2 Rate Limit Headers

```
X-RateLimit-Limit: 10
X-RateLimit-Remaining: 7
X-RateLimit-Reset: 1704067200
```

---

## 8. WebSocket Events (Future)

Reserved for real-time conversion progress:

```json
// Progress event
{
  "event": "progress",
  "data": {
    "stage": "parsing",
    "percent": 50
  }
}

// Complete event
{
  "event": "complete",
  "data": {
    "docs_url": "https://..."
  }
}
```

---

## 9. SDK Examples

### 9.1 curl

```bash
# Convert HWP to HTML
curl -X POST http://localhost:3000/api/convert \
  -F "file=@document.hwp" \
  -F "format=html"

# Get document info
curl "http://localhost:3000/api/info?url=https://..."
```

### 9.2 JavaScript

```javascript
const formData = new FormData();
formData.append('file', hwpFile);
formData.append('format', 'html');

const response = await fetch('/api/convert', {
  method: 'POST',
  body: formData
});

const { data } = await response.json();
console.log(data.content);
```

### 9.3 Python

```python
import requests

with open('document.hwp', 'rb') as f:
    response = requests.post(
        'http://localhost:3000/api/convert',
        files={'file': f},
        data={'format': 'html'}
    )

result = response.json()
print(result['data']['content'])
```

---

## Appendix: OpenAPI Spec Reference

Full OpenAPI 3.0 specification available at:
- Development: `http://localhost:3000/openapi.json`
- Documentation: `http://localhost:3000/docs`

---

**Author:** @Architect
**Date:** 2025-12-09
