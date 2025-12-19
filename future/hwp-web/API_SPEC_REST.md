# API Spec — hwp-web (Planned)

This file contains the planned REST/web-facing API specification that is **excluded in Option A**.

## 2. REST API (hwp-web) — planned (disabled)

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

