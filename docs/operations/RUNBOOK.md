# Runbook - HwpBridge (Option A)

> Option A는 `hwp-web`을 포함하지 않습니다. (웹 서버 런북은 `../../future/`)

## 1. Smoke checks

### 1.1 CLI
```bash
cargo run -p hwp-cli -- --help
cargo run -p hwp-cli -- info ./data/sample.hwp
cargo run -p hwp-cli -- extract ./data/sample.hwp -o /tmp/out.txt
```

### 1.2 MCP
```bash
cargo run -p hwp-mcp
```

- 정상이라면 stdout에 초기화/접속 로그가 출력됩니다.
- 실제 tool call은 MCP 호스트(IDE/Agent)가 수행합니다.

## 2. Common failures
- **PARSE_ERROR**: 파일이 손상/암호화/미지원 버전일 수 있음
- **InvalidParams**: MCP input schema(file base64)가 맞지 않음

## 3. Logging
```bash
RUST_LOG=info,hwp_mcp=debug cargo run -p hwp-mcp
```

## 4. Planned web runbook
- `../../future/hwp-web/RUNBOOK_WEB.md`
