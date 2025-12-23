# MCP Server (hwp-mcp)

Transport: **stdio**.

## Tools

### 1) `read_hwp_summary`
Input:
- `file.name` (string)
- `file.content` (base64)

Output:
- Text summary: doc title + flags + section count
- Structured payload: `StructuredDocument`

### 2) `read_hwp_content`
Input:
- `file`
- `format`: `"semantic-markdown"` (default) or `"plain"`

Output:
- Text: converted markdown or plain text
- Structured payload: `StructuredDocument`

### 3) `convert_to_gdocs`
Input:
- `file`

Output:
- Text: not implemented error (P2)

## Safety
- Max input size is enforced via `HWP_MAX_FILE_BYTES` (default 25MB).
