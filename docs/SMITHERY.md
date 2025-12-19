# Smithery publish notes (HwpBridge)

This repo includes `smithery.yaml` so Smithery can build and run the MCP server.

## What Smithery needs
- `smithery.yaml` (this repo uses **HTTP / Streamable HTTP** for container hosting)
- A build recipe. Here we use the root `Dockerfile` (multi-stage) to produce the `hwp-mcp` runtime image.

## Expected start command
Smithery runs the command returned by `commandFunction`:
- `hwp-mcp` (HTTP transport)

Endpoints:
- `GET /health`
- `POST /mcp`
- `GET /mcp` (SSE logs; requires `mcp-session-id`)

## Recommended registry setup
- Repository: `Prometheus-P/hwp-bridge`
- Server name: `@prometheus-p/hwp-bridge` (or whatever Smithery assigns)
- Category tags: `documents`, `conversion`, `korean`, `office`

## Local smoke test
```bash
docker build -t hwp-bridge-mcp --target hwp-mcp .
docker run --rm -p 8081:8081 -e RUST_LOG=info -e PORT=8081 -e HWP_MCP_TRANSPORT=http hwp-bridge-mcp
```

## Notes
- HTTP / Streamable HTTP is convenient for container hosting (Smithery, K8s, etc.).
- If you need maximum compatibility with legacy clients, keep a stdio-only variant (this repo still supports `HWP_MCP_TRANSPORT=stdio`).
- If you add heavy dependencies, increase `timeout` in `smithery.yaml`.


## Runtime Limits (recommended)
- `HWP_MAX_FILE_BYTES` (already supported)
- `HWP_MAX_DECOMPRESSED_BYTES_PER_SECTION` (default 67108864)
- `HWP_MAX_RECORDS_PER_SECTION` (default 200000)
