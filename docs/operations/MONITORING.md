# Monitoring Guide - HwpBridge

> **Version:** 1.0.0
> **Author:** @DevOps
> **Last Updated:** 2025-12-09

---

## 1. Overview

### 1.1 Monitoring Stack

```
┌─────────────────────────────────────────────────────────────┐
│                    Monitoring Stack                         │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │   hwp-web   │───▶│ Prometheus  │───▶│   Grafana   │     │
│  │  /metrics   │    │   (TSDB)    │    │ (Dashboard) │     │
│  └─────────────┘    └─────────────┘    └─────────────┘     │
│         │                  │                               │
│         │                  ▼                               │
│         │           ┌─────────────┐                        │
│         │           │Alertmanager │──▶ Slack/Email/PD     │
│         │           └─────────────┘                        │
│         │                                                  │
│         ▼                                                  │
│  ┌─────────────┐    ┌─────────────┐                        │
│  │    Loki     │◀───│  Promtail   │                        │
│  │   (Logs)    │    │ (Collector) │                        │
│  └─────────────┘    └─────────────┘                        │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 Key Metrics

| Category | Metrics |
|----------|---------|
| **Availability** | Uptime, Error rate |
| **Latency** | Response time (p50, p95, p99) |
| **Throughput** | Requests/sec, Bytes/sec |
| **Resources** | CPU, Memory, Disk |
| **Business** | Conversions/hour, Success rate |

---

## 2. Metrics Collection

### 2.1 Application Metrics (Prometheus)

**Exposed Endpoint:**

```
GET /metrics
```

**Example Output:**

```prometheus
# HELP hwp_requests_total Total number of HTTP requests
# TYPE hwp_requests_total counter
hwp_requests_total{method="POST",endpoint="/api/convert",status="200"} 1523
hwp_requests_total{method="POST",endpoint="/api/convert",status="422"} 45
hwp_requests_total{method="GET",endpoint="/health",status="200"} 89012

# HELP hwp_request_duration_seconds HTTP request duration
# TYPE hwp_request_duration_seconds histogram
hwp_request_duration_seconds_bucket{endpoint="/api/convert",le="0.1"} 800
hwp_request_duration_seconds_bucket{endpoint="/api/convert",le="0.5"} 1400
hwp_request_duration_seconds_bucket{endpoint="/api/convert",le="1.0"} 1500
hwp_request_duration_seconds_bucket{endpoint="/api/convert",le="+Inf"} 1523

# HELP hwp_parse_duration_seconds HWP parsing duration
# TYPE hwp_parse_duration_seconds histogram
hwp_parse_duration_seconds_bucket{le="0.01"} 500
hwp_parse_duration_seconds_bucket{le="0.1"} 1400
hwp_parse_duration_seconds_bucket{le="1.0"} 1520

# HELP hwp_file_size_bytes Uploaded file size
# TYPE hwp_file_size_bytes histogram
hwp_file_size_bytes_bucket{le="102400"} 800
hwp_file_size_bytes_bucket{le="1048576"} 1400
hwp_file_size_bytes_bucket{le="10485760"} 1523

# HELP hwp_conversion_success_total Successful conversions
# TYPE hwp_conversion_success_total counter
hwp_conversion_success_total 1478

# HELP hwp_conversion_failure_total Failed conversions
# TYPE hwp_conversion_failure_total counter
hwp_conversion_failure_total{reason="encrypted"} 30
hwp_conversion_failure_total{reason="distribution"} 10
hwp_conversion_failure_total{reason="parse_error"} 5
```

### 2.2 Prometheus Configuration

```yaml
# deploy/prometheus/prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'hwp-web'
    static_configs:
      - targets: ['hwp-web:3000']
    metrics_path: '/metrics'
    scrape_interval: 10s

  - job_name: 'node'
    static_configs:
      - targets: ['node-exporter:9100']

alerting:
  alertmanagers:
    - static_configs:
        - targets: ['alertmanager:9093']

rule_files:
  - 'alert_rules.yml'
```

### 2.3 Alert Rules

```yaml
# deploy/prometheus/alert_rules.yml
groups:
  - name: hwp-bridge
    rules:
      # High error rate
      - alert: HighErrorRate
        expr: |
          sum(rate(hwp_requests_total{status=~"5.."}[5m]))
          / sum(rate(hwp_requests_total[5m])) > 0.05
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value | humanizePercentage }}"

      # High latency
      - alert: HighLatency
        expr: |
          histogram_quantile(0.95, rate(hwp_request_duration_seconds_bucket[5m])) > 2
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High latency detected"
          description: "p95 latency is {{ $value }}s"

      # Service down
      - alert: ServiceDown
        expr: up{job="hwp-web"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "HwpBridge service is down"
          description: "{{ $labels.instance }} is unreachable"

      # High memory usage
      - alert: HighMemoryUsage
        expr: |
          process_resident_memory_bytes{job="hwp-web"}
          / 1024 / 1024 > 400
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage"
          description: "Memory usage is {{ $value }}MB"

      # Too many encrypted documents
      - alert: ManyEncryptedDocs
        expr: |
          increase(hwp_conversion_failure_total{reason="encrypted"}[1h]) > 100
        for: 0m
        labels:
          severity: info
        annotations:
          summary: "Many encrypted document uploads"
          description: "{{ $value }} encrypted docs in last hour"
```

---

## 3. Logging

### 3.1 Log Format

```rust
// Structured JSON logging
{
  "timestamp": "2025-01-15T10:30:00Z",
  "level": "INFO",
  "target": "hwp_web::routes::upload",
  "message": "File uploaded successfully",
  "request_id": "abc123",
  "file_size": 1048576,
  "parse_time_ms": 150,
  "version": "5.1.0.0"
}
```

### 3.2 Log Levels

| Level | Use Case |
|-------|----------|
| `ERROR` | 처리 불가 에러, 즉시 조치 필요 |
| `WARN` | 예상된 실패 (암호화 문서 등) |
| `INFO` | 주요 이벤트 (요청 완료, 변환 성공) |
| `DEBUG` | 상세 디버깅 정보 |
| `TRACE` | 매우 상세한 추적 |

### 3.3 Log Collection (Loki + Promtail)

**Promtail Configuration:**

```yaml
# deploy/promtail/promtail.yml
server:
  http_listen_port: 9080

positions:
  filename: /tmp/positions.yaml

clients:
  - url: http://loki:3100/loki/api/v1/push

scrape_configs:
  - job_name: hwp-bridge
    docker_sd_configs:
      - host: unix:///var/run/docker.sock
        refresh_interval: 5s
    relabel_configs:
      - source_labels: ['__meta_docker_container_name']
        regex: '/hwp-(.*)'
        target_label: 'service'
      - source_labels: ['__meta_docker_container_log_stream']
        target_label: 'stream'
    pipeline_stages:
      - json:
          expressions:
            level: level
            request_id: request_id
      - labels:
          level:
          request_id:
```

### 3.4 Log Queries (LogQL)

```logql
# 에러 로그
{service="hwp-web"} |= "ERROR"

# 특정 요청 추적
{service="hwp-web"} |~ "request_id.*abc123"

# 느린 요청 (>1s)
{service="hwp-web"} | json | parse_time_ms > 1000

# 암호화 문서 업로드
{service="hwp-web"} |= "Encrypted"

# 최근 1시간 에러 수
count_over_time({service="hwp-web"} |= "ERROR" [1h])
```

---

## 4. Dashboards (Grafana)

### 4.1 Overview Dashboard

```json
{
  "title": "HwpBridge Overview",
  "panels": [
    {
      "title": "Request Rate",
      "type": "graph",
      "targets": [
        {
          "expr": "sum(rate(hwp_requests_total[5m]))",
          "legendFormat": "Total"
        }
      ]
    },
    {
      "title": "Error Rate",
      "type": "stat",
      "targets": [
        {
          "expr": "sum(rate(hwp_requests_total{status=~\"5..\"}[5m])) / sum(rate(hwp_requests_total[5m])) * 100"
        }
      ],
      "thresholds": {
        "steps": [
          {"color": "green", "value": 0},
          {"color": "yellow", "value": 1},
          {"color": "red", "value": 5}
        ]
      }
    },
    {
      "title": "Latency (p95)",
      "type": "gauge",
      "targets": [
        {
          "expr": "histogram_quantile(0.95, rate(hwp_request_duration_seconds_bucket[5m]))"
        }
      ]
    },
    {
      "title": "Conversion Success Rate",
      "type": "stat",
      "targets": [
        {
          "expr": "sum(hwp_conversion_success_total) / (sum(hwp_conversion_success_total) + sum(hwp_conversion_failure_total)) * 100"
        }
      ]
    }
  ]
}
```

### 4.2 Key Panels

| Panel | Query | Alert Threshold |
|-------|-------|-----------------|
| Request Rate | `sum(rate(hwp_requests_total[5m]))` | - |
| Error Rate | `sum(rate(...{status=~"5.."})) / sum(rate(...))` | > 5% |
| p95 Latency | `histogram_quantile(0.95, ...)` | > 2s |
| Memory Usage | `process_resident_memory_bytes` | > 400MB |
| Active Connections | `hwp_active_connections` | > 100 |

### 4.3 Grafana Setup

```bash
# Docker Compose로 시작
docker compose --profile monitoring up -d

# 접속
open http://localhost:3001

# 기본 로그인: admin / admin
```

---

## 5. Health Monitoring

### 5.1 Health Check Endpoint

```rust
// GET /health
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_seconds": 86400,
  "checks": {
    "parser": {
      "status": "ok",
      "latency_ms": 5
    },
    "google_api": {
      "status": "ok",
      "latency_ms": 120
    },
    "redis": {
      "status": "ok",
      "latency_ms": 2
    }
  }
}
```

### 5.2 Uptime Monitoring

**External Services:**

- [UptimeRobot](https://uptimerobot.com) - Free tier
- [Pingdom](https://www.pingdom.com)
- [Better Uptime](https://betteruptime.com)

**Configuration:**

```yaml
# UptimeRobot config
monitors:
  - name: "HwpBridge API"
    url: "https://api.hwpbridge.io/health"
    type: "HTTP"
    interval: 60
    alert_contacts: ["email", "slack"]
```

### 5.3 Synthetic Monitoring

```bash
#!/bin/bash
# synthetic-test.sh - 주기적 실행

API_URL="https://api.hwpbridge.io"
TEST_FILE="./test.hwp"

# Health check
curl -sf "${API_URL}/health" || exit 1

# API test
response=$(curl -sf -X POST "${API_URL}/api/convert" \
  -F "file=@${TEST_FILE}" \
  -F "format=html")

if echo "$response" | jq -e '.success == true' > /dev/null; then
    echo "Synthetic test passed"
    exit 0
else
    echo "Synthetic test failed"
    exit 1
fi
```

---

## 6. Alerting

### 6.1 Alertmanager Configuration

```yaml
# deploy/alertmanager/alertmanager.yml
global:
  slack_api_url: 'https://hooks.slack.com/services/xxx'
  smtp_smarthost: 'smtp.gmail.com:587'
  smtp_from: 'alerts@hwpbridge.io'

route:
  group_by: ['alertname', 'severity']
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 4h
  receiver: 'default'
  routes:
    - match:
        severity: critical
      receiver: 'pagerduty'
    - match:
        severity: warning
      receiver: 'slack'

receivers:
  - name: 'default'
    email_configs:
      - to: 'team@hwpbridge.io'

  - name: 'slack'
    slack_configs:
      - channel: '#hwp-alerts'
        title: '{{ .GroupLabels.alertname }}'
        text: '{{ .CommonAnnotations.description }}'

  - name: 'pagerduty'
    pagerduty_configs:
      - service_key: 'xxx'
        severity: '{{ .GroupLabels.severity }}'
```

### 6.2 Alert Severity Matrix

| Severity | Response Time | Examples |
|----------|---------------|----------|
| **Critical** | < 15 min | Service down, Data loss |
| **Warning** | < 1 hour | High latency, High error rate |
| **Info** | Next business day | Capacity planning |

### 6.3 Escalation Policy

```
Level 1 (0-15min)  → On-call engineer (Slack + PagerDuty)
Level 2 (15-30min) → Team lead
Level 3 (30-60min) → Engineering manager
```

---

## 7. Tracing (OpenTelemetry)

### 7.1 Configuration

```rust
// Cargo.toml
[dependencies]
opentelemetry = "0.21"
opentelemetry-otlp = "0.14"
tracing-opentelemetry = "0.22"
```

### 7.2 Span Example

```rust
use tracing::{instrument, info_span};

#[instrument(skip(data), fields(file_size = data.len()))]
pub fn parse_hwp(data: &[u8]) -> Result<HwpDocument, HwpError> {
    let _header_span = info_span!("parse_header").entered();
    let header = parse_file_header(data)?;

    let _body_span = info_span!("parse_body").entered();
    let body = parse_body_text(data)?;

    Ok(HwpDocument { header, body })
}
```

### 7.3 Jaeger Setup

```yaml
# docker-compose.yml
services:
  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "16686:16686"  # UI
      - "4317:4317"    # OTLP gRPC
    environment:
      - COLLECTOR_OTLP_ENABLED=true
```

---

## 8. SLIs/SLOs

### 8.1 Service Level Indicators

| SLI | Definition | Measurement |
|-----|------------|-------------|
| **Availability** | 성공 요청 비율 | `success / total * 100` |
| **Latency** | 응답 시간 | p50, p95, p99 |
| **Throughput** | 처리량 | requests/sec |
| **Error Rate** | 에러 비율 | `errors / total * 100` |

### 8.2 Service Level Objectives

| SLO | Target | Measurement Window |
|-----|--------|-------------------|
| Availability | 99.9% | 30 days |
| Latency (p95) | < 2s | 30 days |
| Error Rate | < 0.1% | 30 days |

### 8.3 Error Budget

```
Monthly Error Budget = (1 - SLO) × Total Requests

Example:
- SLO: 99.9%
- Monthly Requests: 1,000,000
- Error Budget: 0.1% × 1,000,000 = 1,000 errors/month
```

---

## 9. Capacity Planning

### 9.1 Current Metrics

```promql
# Daily request count
sum(increase(hwp_requests_total[24h]))

# Peak requests per second
max_over_time(rate(hwp_requests_total[1m])[24h:1m])

# Average file size
avg(hwp_file_size_bytes)

# Memory per request
process_resident_memory_bytes / hwp_active_connections
```

### 9.2 Capacity Thresholds

| Resource | Warning | Critical | Action |
|----------|---------|----------|--------|
| CPU | 70% | 85% | Scale horizontally |
| Memory | 70% | 85% | Increase limit |
| Disk | 70% | 85% | Expand volume |
| Connections | 80 | 100 | Add instances |

### 9.3 Scaling Recommendations

```
Requests/sec × Avg Latency = Required Connections
Required Connections / Connections per Instance = Required Instances

Example:
- 100 req/s × 0.5s = 50 concurrent
- 50 / 25 per instance = 2 instances (+ buffer = 3)
```

---

## 10. Runbook Integration

모든 알람은 [RUNBOOK.md](./RUNBOOK.md)의 해당 섹션으로 연결:

| Alert | Runbook Section |
|-------|-----------------|
| ServiceDown | RB-001: Service Recovery |
| HighErrorRate | RB-002: Error Investigation |
| HighLatency | RB-003: Performance Issues |
| HighMemoryUsage | RB-004: Memory Leak |

---

**Next:** [RUNBOOK.md](./RUNBOOK.md) - 장애 대응 가이드
