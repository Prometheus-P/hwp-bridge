# MCP Server (hwp-mcp)

Transport: **stdio**.

## Tools

### 1) `hwp.inspect`
Input:
- `file.name` (string)
- `file.content` (base64)

Output:
- Text summary: doc title + flags + section count
- Structured payload: `StructuredDocument`

### 2) `hwp.to_markdown`
Input:
- `file`
- `format`: `"markdown"` (default) or `"text"`

Output:
- Text: converted markdown or plain text
- Structured payload: `StructuredDocument`

### 3) `hwp.extract`
Input:
- `file`

Output:
- Text: plain text extraction
- Structured payload: `StructuredDocument`

### 4) `hwp.to_json`
Input:
- `file`
- `pretty` (bool, default false)

Output:
- Text: JSON string (pretty or compact)
- Structured payload: `StructuredDocument`

## Safety
- Max input size is enforced via `HWP_MAX_FILE_BYTES` (default 25MB).
