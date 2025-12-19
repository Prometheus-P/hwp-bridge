# Monitoring - HwpBridge (Option A)

Option A는 기본적으로 **tracing 로그 기반**으로 관측합니다.

## 1. Logging
- `RUST_LOG`로 레벨 조정
  - 예: `RUST_LOG=info,hwp_core=debug,hwp_mcp=debug`

## 2. What to monitor
- 파싱 실패율 (PARSE_ERROR 빈도)
- 파일 크기/처리시간(호스트에서 측정 권장)
- 메모리 사용량 (컨테이너/프로세스 수준)

## 3. Planned (web) monitoring stack
`hwp-web` + reverse proxy + Prometheus/Grafana는 Option A 범위에서 제외되었습니다.  
관련 설정은 `../../future/`로 이동했습니다.

- `../../future/deploy/prometheus/`
- `../../future/deploy/nginx/`
