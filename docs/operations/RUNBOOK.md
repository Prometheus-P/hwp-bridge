# Runbook - HwpBridge

> **Version:** 1.0.0
> **Author:** @DevOps
> **Last Updated:** 2025-12-09

---

## 1. Overview

이 문서는 HwpBridge 서비스의 운영 중 발생할 수 있는 장애 상황에 대한 대응 절차를 정의합니다.

### 1.1 On-Call Responsibilities

- 알람 확인 및 초기 대응 (15분 이내)
- 장애 등급 판단 및 에스컬레이션
- 복구 작업 수행
- 사후 분석 (Post-mortem) 문서 작성

### 1.2 Severity Levels

| Level | Impact | Response Time | Example |
|-------|--------|---------------|---------|
| **SEV1** | 전체 서비스 장애 | 15분 | 모든 요청 실패 |
| **SEV2** | 주요 기능 장애 | 30분 | 변환 실패 50%+ |
| **SEV3** | 부분 기능 장애 | 2시간 | 특정 파일 타입 실패 |
| **SEV4** | 경미한 이슈 | 1일 | 성능 저하 |

---

## 2. Quick Reference

### 2.1 Essential Commands

```bash
# 서비스 상태 확인
docker compose ps
kubectl get pods -n hwp-bridge

# 로그 확인
docker logs -f hwp-web --tail 100
kubectl logs -f deployment/hwp-web -n hwp-bridge

# 서비스 재시작
docker compose restart hwp-web
kubectl rollout restart deployment/hwp-web -n hwp-bridge

# Health check
curl -sf http://localhost:3000/health | jq

# 메트릭 확인
curl -sf http://localhost:3000/metrics | grep hwp_
```

### 2.2 Key URLs

| Service | URL |
|---------|-----|
| API Health | https://api.hwpbridge.io/health |
| Grafana | https://grafana.hwpbridge.io |
| Prometheus | https://prometheus.hwpbridge.io |
| Logs (Loki) | https://grafana.hwpbridge.io/explore |

### 2.3 Contact List

| Role | Contact |
|------|---------|
| On-Call | #hwp-oncall (Slack) |
| Team Lead | @team-lead |
| DevOps | #hwp-devops (Slack) |

---

## RB-001: Service Down

### Symptoms

- `ServiceDown` 알람 발생
- Health check 실패
- 모든 API 요청 실패

### Diagnosis

```bash
# 1. 컨테이너 상태 확인
docker compose ps
# 또는
kubectl get pods -n hwp-bridge

# 2. 컨테이너 로그 확인
docker logs hwp-web --tail 50
# 또는
kubectl logs deployment/hwp-web -n hwp-bridge --tail 50

# 3. 리소스 사용량 확인
docker stats hwp-web
# 또는
kubectl top pods -n hwp-bridge

# 4. 네트워크 연결 확인
curl -v http://localhost:3000/health
```

### Resolution

**Step 1: 서비스 재시작**

```bash
# Docker
docker compose restart hwp-web

# Kubernetes
kubectl rollout restart deployment/hwp-web -n hwp-bridge
```

**Step 2: 재시작 후에도 실패 시 - 로그 분석**

```bash
# OOM 확인
docker inspect hwp-web | jq '.[0].State'
kubectl describe pod -l app=hwp-web -n hwp-bridge | grep -A5 "Last State"

# OOM이면 메모리 limit 증가
docker compose down
# docker-compose.yml에서 메모리 증가
docker compose up -d
```

**Step 3: 롤백 (최근 배포 후 발생 시)**

```bash
# Docker
docker tag hwp-bridge/web:previous hwp-bridge/web:latest
docker compose up -d

# Kubernetes
kubectl rollout undo deployment/hwp-web -n hwp-bridge
```

### Escalation

- 15분 내 복구 실패 → Team Lead 호출
- 30분 내 복구 실패 → Engineering Manager 호출

---

## RB-002: High Error Rate

### Symptoms

- `HighErrorRate` 알람 (>5%)
- 5xx 응답 증가
- 사용자 불만 접수

### Diagnosis

```bash
# 1. 에러 로그 확인
docker logs hwp-web 2>&1 | grep -i error | tail -20

# 2. 에러 타입별 분류
curl -s localhost:3000/metrics | grep hwp_requests_total

# 3. 최근 변경사항 확인
git log --oneline -10

# 4. 외부 의존성 확인 (Google API 등)
curl -sf https://www.googleapis.com/ > /dev/null && echo "Google OK" || echo "Google FAIL"
```

### Resolution

**Case 1: 파싱 에러 급증**

```bash
# 특정 파일 패턴 확인
docker logs hwp-web 2>&1 | grep ParseError

# 문제 파일 샘플 수집 (사용자 동의 필요)
# 로컬에서 재현 테스트
cargo test -p hwp-core
```

**Case 2: Google API 에러**

```bash
# API quota 확인
# Google Cloud Console > APIs & Services > Quotas

# 임시 조치: Google 연동 비활성화
docker compose down
# 환경변수에서 GOOGLE_CLIENT_ID 제거
docker compose up -d
```

**Case 3: 메모리 부족 (OOM)**

```bash
# 메모리 사용량 확인
docker stats hwp-web

# 메모리 limit 증가
# docker-compose.yml 수정 후 재시작
docker compose up -d
```

### Post-Incident

- [ ] 에러 샘플 수집
- [ ] 근본 원인 분석
- [ ] 테스트 케이스 추가
- [ ] Post-mortem 문서 작성

---

## RB-003: High Latency

### Symptoms

- `HighLatency` 알람 (p95 > 2s)
- 사용자 타임아웃 보고
- 요청 큐 증가

### Diagnosis

```bash
# 1. 현재 레이턴시 확인
curl -s localhost:3000/metrics | grep hwp_request_duration

# 2. 활성 연결 수 확인
curl -s localhost:3000/metrics | grep hwp_active

# 3. 리소스 사용량
docker stats hwp-web
top -p $(pgrep hwp-web)

# 4. 대용량 파일 처리 확인
docker logs hwp-web 2>&1 | grep -E "file_size.*[0-9]{7,}"
```

### Resolution

**Case 1: CPU 병목**

```bash
# 인스턴스 추가 (수평 확장)
docker compose up -d --scale hwp-web=3

# Kubernetes
kubectl scale deployment hwp-web --replicas=5 -n hwp-bridge
```

**Case 2: 대용량 파일 처리**

```bash
# 파일 크기 제한 강화 (임시)
# MAX_UPLOAD_SIZE_MB=5 설정

# 장기: 스트리밍 처리 구현
```

**Case 3: 외부 API 지연**

```bash
# Google API 응답 시간 확인
curl -w "@curl-format.txt" -s -o /dev/null https://www.googleapis.com/

# 캐싱 활성화
# REDIS_URL 환경변수 설정
```

---

## RB-004: Memory Leak

### Symptoms

- `HighMemoryUsage` 알람
- 메모리 사용량 지속 증가
- OOM Kill 발생

### Diagnosis

```bash
# 1. 메모리 추이 확인 (Prometheus)
# process_resident_memory_bytes{job="hwp-web"}

# 2. 힙 프로파일 (개발 환경)
MALLOC_CONF="prof:true" ./hwp-web
jeprof ./hwp-web ./heap.prof

# 3. 요청 수 대비 메모리 비율
curl -s localhost:3000/metrics | grep -E "(memory|requests_total)"
```

### Resolution

**즉시 조치:**

```bash
# 서비스 재시작 (임시 완화)
docker compose restart hwp-web

# 메모리 limit 증가 (임시)
# docker-compose.yml 수정
```

**근본 해결:**

```bash
# 1. 메모리 프로파일링
cargo build --release --features memory-profiling

# 2. 코드 리뷰
# - 큰 버퍼 해제 확인
# - Drop trait 구현 확인
# - Arc/Rc 순환 참조 확인

# 3. 수정 및 배포
```

---

## RB-005: Disk Full

### Symptoms

- 디스크 사용량 알람
- 로그 쓰기 실패
- 서비스 장애

### Diagnosis

```bash
# 디스크 사용량 확인
df -h

# 큰 파일 찾기
du -sh /var/log/* | sort -rh | head

# 오래된 로그 확인
find /var/log -name "*.log" -mtime +7
```

### Resolution

```bash
# 1. 오래된 로그 삭제
find /var/log -name "*.log" -mtime +7 -delete

# 2. Docker 정리
docker system prune -af --volumes

# 3. 로그 로테이션 설정 확인
cat /etc/logrotate.d/hwp-bridge
```

---

## RB-006: SSL Certificate Expiry

### Symptoms

- `CertificateExpiringSoon` 알람
- HTTPS 연결 실패 (만료 후)

### Diagnosis

```bash
# 인증서 만료일 확인
echo | openssl s_client -connect api.hwpbridge.io:443 2>/dev/null | \
  openssl x509 -noout -dates
```

### Resolution

```bash
# Let's Encrypt 갱신
sudo certbot renew

# 수동 갱신 (필요시)
sudo certbot certonly --standalone -d api.hwpbridge.io

# Nginx 재시작
docker compose restart nginx
```

---

## RB-007: Database Connection Issues

### Symptoms

- 데이터베이스 연결 실패
- 타임아웃 에러

### Diagnosis

```bash
# Redis 연결 확인
redis-cli -h localhost ping

# PostgreSQL 연결 확인 (future)
# psql -h localhost -U hwp -d hwpbridge -c "SELECT 1"
```

### Resolution

```bash
# Redis 재시작
docker compose restart redis

# 연결 풀 설정 확인
# 최대 연결 수 증가 필요 시 설정 변경
```

---

## 3. Post-Incident Procedures

### 3.1 Incident Documentation

```markdown
## Incident Report: [Title]

**Date:** YYYY-MM-DD
**Duration:** HH:MM - HH:MM (X hours)
**Severity:** SEV1/SEV2/SEV3/SEV4
**Impact:** [영향 범위 설명]

### Timeline
- HH:MM - 알람 발생
- HH:MM - 조사 시작
- HH:MM - 원인 파악
- HH:MM - 복구 완료

### Root Cause
[근본 원인 설명]

### Resolution
[해결 방법]

### Action Items
- [ ] [예방 조치 1]
- [ ] [예방 조치 2]

### Lessons Learned
[교훈]
```

### 3.2 Post-Mortem Meeting

1. 타임라인 리뷰
2. 근본 원인 분석 (5 Whys)
3. 액션 아이템 도출
4. 프로세스 개선 논의

### 3.3 Follow-up Tasks

- [ ] 모니터링 알람 추가/조정
- [ ] 문서 업데이트
- [ ] 테스트 케이스 추가
- [ ] 자동화 개선

---

## 4. Preventive Maintenance

### 4.1 Daily Checks

- [ ] 서비스 상태 확인 (`/health`)
- [ ] 에러율 확인 (< 0.1%)
- [ ] 디스크 사용량 확인 (< 70%)

### 4.2 Weekly Checks

- [ ] 로그 리뷰 (이상 패턴)
- [ ] 성능 메트릭 리뷰
- [ ] 보안 업데이트 확인

### 4.3 Monthly Tasks

- [ ] 인증서 만료일 확인
- [ ] 용량 계획 리뷰
- [ ] 재해 복구 테스트
- [ ] 런북 업데이트

---

## 5. Emergency Contacts

### Internal

| Role | Name | Phone | Slack |
|------|------|-------|-------|
| Primary On-Call | - | - | @oncall |
| Backup On-Call | - | - | @oncall-backup |
| Team Lead | - | - | @team-lead |
| Engineering Manager | - | - | @eng-manager |

### External

| Service | Support |
|---------|---------|
| Google Cloud | https://cloud.google.com/support |
| AWS | https://aws.amazon.com/support |
| Domain Registrar | - |

---

**"Hope is not a strategy. Runbooks are."**
