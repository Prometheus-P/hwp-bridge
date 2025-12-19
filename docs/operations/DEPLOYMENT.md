# Deployment Guide - HwpBridge (Option A)

> Option A는 `hwp-web`(REST 서버)을 포함하지 않습니다.  
> `hwp-web` 배포/Cloud Run/K8s 내용은 `../../future/hwp-web/`로 이동했습니다.

## 1. What you can run in Option A
- `hwp-cli`: 로컬/서버에서 HWP 텍스트 추출 및 헤더 정보 출력
- `hwp-mcp`: MCP stdio 서버(호스트/에이전트가 붙어서 사용)
- `hwp-wasm`: 브라우저/JS 런타임에서 HWP 파싱/변환(번들링 필요)

## 2. Quick Start (Docker) — hwp-mcp

```bash
docker compose up -d --build
docker compose logs -f hwp-mcp
```

> `hwp-mcp`는 HTTP 서버가 아니라 **stdio 기반 MCP**입니다.  
> 실제 사용은 IDE/Agent가 컨테이너의 stdin/stdout에 연결해 tool call을 수행합니다.

### 2.1 Mount files
`docker-compose.yml`에서 `./data:/app/data:ro`를 기본으로 마운트합니다.  
호스트가 파일을 읽어서 MCP 입력(base64)로 넘길 수 있다면, 마운트는 선택입니다.

## 3. Build (local)

### 3.1 CLI
```bash
cargo build -p hwp-cli --release
./target/release/hwp --help
```

### 3.2 MCP server
```bash
cargo build -p hwp-mcp --release
RUST_LOG=info ./target/release/hwp-mcp
```

### 3.3 WASM
```bash
# wasm-pack 필요
wasm-pack build crates/hwp-wasm --release --target web
```

## 4. Environment variables
- `RUST_LOG` (예: `info,hwp_mcp=debug`)

## 5. Planned: hwp-web deployment
- `../../future/hwp-web/DEPLOYMENT_WEB.md`
