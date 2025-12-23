# HwpBridge

Rust-first HWP parsing/conversion toolkit + MCP server.

- **CLI**: local conversion & inspection
- **MCP**: use HWP tools inside AI clients (Claude Desktop, etc.)
- **Spec-first**: format notes + fixtures + regression tests

> Option A: `hwp-web` is parked in `/future/` (not part of the build).

## Docs
- `docs/specs/V1_ACCEPTANCE.md` — release gate / success criteria
- `docs/specs/P0_CHECKLIST.md` — P0 implementation checklist
- `docs/guides/TEST_MATRIX.md` — test matrix
- `docs/guides/CORPUS.md` — corpus design
- `docs/guides/V1_RELEASE_GATE.md` — how to run the V1 gate


## Install / Build

```bash
# workspace build
cargo build --release

# CLI
cargo run -p hwp-cli -- --help

# MCP server (stdio)
cargo run -p hwp-mcp

# MCP server (HTTP, for Smithery/container)
HWP_MCP_TRANSPORT=http PORT=8081 cargo run -p hwp-mcp
```

## HTTP endpoints

- `GET /health` → `ok`
- `POST /mcp` → JSON-RPC 2.0 (MCP)
- `GET /mcp` → SSE stream (server log events). Requires `mcp-session-id` (or `?sid=` for the optional dev UI).

### Optional dev UI (local testing)

The dev UI is **disabled by default**. Enable only for local testing.

```bash
# build + run with the UI enabled
HWP_DEV_UI=1 HWP_DEV_UI_TOKEN=changeme \
  HWP_MCP_TRANSPORT=http PORT=8081 \
  cargo run -p hwp-mcp --features dev-ui

# open
open http://localhost:8081/__dev/ui/login
```

The dev UI is protected by a short-lived session cookie and a shared token. Do not enable it in public deployments.

## CLI usage

```bash
# show basic info
cargo run -p hwp-cli -- info ./samples/example.hwp

# extract plain text
cargo run -p hwp-cli -- extract ./samples/example.hwp --output out.txt
```

## MCP tools

- `hwp.inspect` — parse + return a structured summary and metadata
- `hwp.to_markdown` — semantic markdown (or plain text)
- `hwp.extract` — plain text fast path
- `hwp.to_json` — structured JSON output (text + structured payload)

## Installing via Smithery

If you publish this repo to Smithery, users can install it like:

```bash
npx -y @smithery/cli install @prometheus-p/hwp-bridge --client claude
```

(You will replace the server id with the one Smithery assigns.)


## Quality gate (private corpus)

This repo does **not** commit HWP documents. Put your files under `corpus/local/` (gitignored), then run the V1 gate:

```bash
cargo build --release -p hwp-cli
python3 scripts/corpus_scan.py
python3 scripts/v1_gate.py --ci --min-corpus-size 100
```

CI automation is provided in `.github/workflows/v1-gate.yml` (requires `CORPUS_ZIP_URL` secret).

## License

Copyright (c) 2025 HwpBridge. All Rights Reserved.

This software is **proprietary**. See `LICENSE` for terms.

- **Personal Use**: Free for evaluation and non-commercial use
- **Commercial Use**: Requires a license - see `COMMERCIAL_LICENSE.md`

Contact: parkdavid31@gmail.com


## Limitations

- **HWPX (.hwpx) is not supported yet**. Requests will return `UNSUPPORTED_FORMAT`. Only **HWP v5 `.hwp` (OLE/CFB)** is supported in v1.
