# Deployment Guide - HwpBridge

> **Version:** 1.0.0
> **Author:** @DevOps
> **Last Updated:** 2025-12-09

---

## 1. Overview

### 1.1 Deployment Options

| Option | Use Case | Complexity |
|--------|----------|------------|
| **Docker Compose** | 로컬 개발, 단일 서버 | Low |
| **Google Cloud Run** | 서버리스, 자동 스케일링 | Medium |
| **Kubernetes** | 대규모, 멀티 리전 | High |
| **Binary** | 임베디드, 에어갭 환경 | Low |

### 1.2 System Requirements

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| CPU | 1 core | 2 cores |
| Memory | 256MB | 512MB |
| Disk | 100MB | 500MB |
| OS | Linux (glibc 2.31+) | Debian 12 / Ubuntu 22.04 |

---

## 2. Quick Start (Docker)

### 2.1 Prerequisites

```bash
# Docker & Docker Compose 설치
docker --version  # 24.0+
docker compose version  # 2.20+
```

### 2.2 Run with Docker Compose

```bash
# 1. 프로젝트 클론
git clone https://github.com/user/hwp-bridge.git
cd hwp-bridge

# 2. 환경 변수 설정
cp .env.example .env
# .env 파일 편집

# 3. 빌드 및 실행
docker compose up -d

# 4. 상태 확인
docker compose ps
curl http://localhost:3000/health
```

### 2.3 Docker Build Options

```bash
# hwp-web만 빌드
docker build --target hwp-web -t hwp-bridge/web:latest .

# hwp-mcp만 빌드
docker build --target hwp-mcp -t hwp-bridge/mcp:latest .

# 모든 바이너리 포함
docker build --target all -t hwp-bridge/all:latest .
```

---

## 3. Google Cloud Run Deployment

### 3.1 Prerequisites

```bash
# gcloud CLI 설치 및 인증
gcloud auth login
gcloud config set project YOUR_PROJECT_ID

# API 활성화
gcloud services enable run.googleapis.com
gcloud services enable artifactregistry.googleapis.com
```

### 3.2 Manual Deployment

```bash
# 1. Artifact Registry 생성 (최초 1회)
gcloud artifacts repositories create hwp-bridge \
  --repository-format=docker \
  --location=asia-northeast3

# 2. Docker 인증 설정
gcloud auth configure-docker asia-northeast3-docker.pkg.dev

# 3. 이미지 빌드 및 푸시
docker build --target hwp-web -t asia-northeast3-docker.pkg.dev/PROJECT_ID/hwp-bridge/hwp-web:latest .
docker push asia-northeast3-docker.pkg.dev/PROJECT_ID/hwp-bridge/hwp-web:latest

# 4. Cloud Run 배포
gcloud run deploy hwp-web \
  --image=asia-northeast3-docker.pkg.dev/PROJECT_ID/hwp-bridge/hwp-web:latest \
  --region=asia-northeast3 \
  --platform=managed \
  --allow-unauthenticated \
  --memory=512Mi \
  --cpu=1 \
  --min-instances=0 \
  --max-instances=10 \
  --port=3000
```

### 3.3 Environment Variables (Cloud Run)

```bash
gcloud run services update hwp-web \
  --set-env-vars="RUST_LOG=info" \
  --set-env-vars="GOOGLE_CLIENT_ID=xxx" \
  --set-secrets="GOOGLE_CLIENT_SECRET=google-secret:latest"
```

### 3.4 Custom Domain

```bash
# 도메인 매핑
gcloud run domain-mappings create \
  --service=hwp-web \
  --domain=api.hwpbridge.io \
  --region=asia-northeast3
```

---

## 4. Kubernetes Deployment

### 4.1 Prerequisites

```bash
# kubectl 설치
kubectl version --client

# 클러스터 연결
kubectl config use-context your-cluster
```

### 4.2 Kubernetes Manifests

**Namespace:**

```yaml
# k8s/namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: hwp-bridge
```

**Deployment:**

```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: hwp-web
  namespace: hwp-bridge
spec:
  replicas: 2
  selector:
    matchLabels:
      app: hwp-web
  template:
    metadata:
      labels:
        app: hwp-web
    spec:
      containers:
        - name: hwp-web
          image: ghcr.io/user/hwp-bridge:latest
          ports:
            - containerPort: 3000
          env:
            - name: RUST_LOG
              value: "info"
            - name: PORT
              value: "3000"
          resources:
            requests:
              memory: "256Mi"
              cpu: "250m"
            limits:
              memory: "512Mi"
              cpu: "500m"
          livenessProbe:
            httpGet:
              path: /health
              port: 3000
            initialDelaySeconds: 5
            periodSeconds: 10
          readinessProbe:
            httpGet:
              path: /health
              port: 3000
            initialDelaySeconds: 3
            periodSeconds: 5
```

**Service:**

```yaml
# k8s/service.yaml
apiVersion: v1
kind: Service
metadata:
  name: hwp-web
  namespace: hwp-bridge
spec:
  selector:
    app: hwp-web
  ports:
    - port: 80
      targetPort: 3000
  type: ClusterIP
```

**Ingress:**

```yaml
# k8s/ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: hwp-web
  namespace: hwp-bridge
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
spec:
  tls:
    - hosts:
        - api.hwpbridge.io
      secretName: hwp-web-tls
  rules:
    - host: api.hwpbridge.io
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: hwp-web
                port:
                  number: 80
```

### 4.3 Deploy to Kubernetes

```bash
# 배포
kubectl apply -f k8s/

# 상태 확인
kubectl get pods -n hwp-bridge
kubectl get svc -n hwp-bridge

# 로그 확인
kubectl logs -f deployment/hwp-web -n hwp-bridge
```

---

## 5. Binary Deployment

### 5.1 Download Pre-built Binaries

```bash
# GitHub Releases에서 다운로드
VERSION=v0.1.0
OS=linux  # linux, darwin, windows
ARCH=amd64  # amd64, arm64

curl -LO "https://github.com/user/hwp-bridge/releases/download/${VERSION}/hwp-bridge-${VERSION}-${OS}-${ARCH}.tar.gz"
tar -xzf hwp-bridge-${VERSION}-${OS}-${ARCH}.tar.gz
```

### 5.2 Build from Source

```bash
# Rust 설치
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 빌드
git clone https://github.com/user/hwp-bridge.git
cd hwp-bridge
cargo build --release --workspace

# 바이너리 위치
ls -la target/release/hwp-*
```

### 5.3 Systemd Service

```ini
# /etc/systemd/system/hwp-web.service
[Unit]
Description=HwpBridge Web Server
After=network.target

[Service]
Type=simple
User=hwp
Group=hwp
WorkingDirectory=/opt/hwp-bridge
ExecStart=/opt/hwp-bridge/hwp-web
Restart=always
RestartSec=5

Environment=RUST_LOG=info
Environment=PORT=3000

# Security
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
PrivateTmp=true

[Install]
WantedBy=multi-user.target
```

```bash
# 서비스 등록 및 시작
sudo systemctl daemon-reload
sudo systemctl enable hwp-web
sudo systemctl start hwp-web
sudo systemctl status hwp-web
```

---

## 6. Configuration

### 6.1 Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `RUST_LOG` | No | `info` | 로그 레벨 |
| `PORT` | No | `3000` | 서버 포트 |
| `GOOGLE_CLIENT_ID` | Yes* | - | Google OAuth Client ID |
| `GOOGLE_CLIENT_SECRET` | Yes* | - | Google OAuth Secret |
| `REDIS_URL` | No | - | Redis 캐시 URL |
| `MAX_UPLOAD_SIZE_MB` | No | `10` | 최대 업로드 크기 |

*Google Docs 변환 기능 사용 시 필수

### 6.2 Configuration File (future)

```toml
# config.toml
[server]
port = 3000
host = "0.0.0.0"

[limits]
max_upload_size_mb = 10
request_timeout_secs = 60

[google]
client_id = "${GOOGLE_CLIENT_ID}"
redirect_uri = "https://hwpbridge.io/auth/google/callback"

[logging]
level = "info"
format = "json"
```

---

## 7. SSL/TLS Setup

### 7.1 Let's Encrypt (Certbot)

```bash
# Certbot 설치
sudo apt install certbot

# 인증서 발급
sudo certbot certonly --standalone -d api.hwpbridge.io

# 자동 갱신 확인
sudo certbot renew --dry-run
```

### 7.2 Self-signed (Development)

```bash
# 개발용 자체 서명 인증서
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout deploy/nginx/ssl/key.pem \
  -out deploy/nginx/ssl/cert.pem \
  -subj "/CN=localhost"
```

---

## 8. Health Checks

### 8.1 Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | 기본 헬스 체크 |
| `/health/live` | GET | Liveness probe |
| `/health/ready` | GET | Readiness probe |

### 8.2 Response Format

```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_seconds": 3600,
  "checks": {
    "parser": "ok",
    "google_api": "ok"
  }
}
```

### 8.3 Monitoring Script

```bash
#!/bin/bash
# health-check.sh

ENDPOINT="http://localhost:3000/health"
TIMEOUT=5

response=$(curl -sf -m $TIMEOUT "$ENDPOINT")
if [ $? -eq 0 ]; then
    echo "OK: $response"
    exit 0
else
    echo "FAIL: Health check failed"
    exit 1
fi
```

---

## 9. Scaling

### 9.1 Horizontal Scaling

```bash
# Docker Compose
docker compose up -d --scale hwp-web=3

# Kubernetes
kubectl scale deployment hwp-web --replicas=5 -n hwp-bridge

# Cloud Run (자동)
gcloud run services update hwp-web --max-instances=20
```

### 9.2 Load Balancer Configuration

```nginx
# Nginx upstream (Round Robin)
upstream hwp_backend {
    server hwp-web-1:3000;
    server hwp-web-2:3000;
    server hwp-web-3:3000;
    keepalive 32;
}
```

### 9.3 Auto-scaling (Kubernetes HPA)

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: hwp-web
  namespace: hwp-bridge
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: hwp-web
  minReplicas: 2
  maxReplicas: 10
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70
```

---

## 10. Rollback

### 10.1 Docker

```bash
# 이전 이미지로 롤백
docker compose down
docker tag hwp-bridge/web:previous hwp-bridge/web:latest
docker compose up -d
```

### 10.2 Kubernetes

```bash
# 롤백
kubectl rollout undo deployment/hwp-web -n hwp-bridge

# 특정 리비전으로 롤백
kubectl rollout undo deployment/hwp-web --to-revision=2 -n hwp-bridge

# 롤백 히스토리
kubectl rollout history deployment/hwp-web -n hwp-bridge
```

### 10.3 Cloud Run

```bash
# 이전 리비전으로 트래픽 이동
gcloud run services update-traffic hwp-web \
  --to-revisions=hwp-web-00002=100
```

---

## 11. Troubleshooting

### 11.1 Common Issues

| Issue | Cause | Solution |
|-------|-------|----------|
| Container won't start | Port conflict | `docker ps` 확인, 포트 변경 |
| Health check fail | Service not ready | 로그 확인, startup probe 조정 |
| OOM killed | 메모리 부족 | 리소스 limit 증가 |
| SSL error | 인증서 만료 | certbot renew |

### 11.2 Debug Commands

```bash
# Docker 로그
docker logs -f hwp-web

# Kubernetes 로그
kubectl logs -f deployment/hwp-web -n hwp-bridge

# 컨테이너 진입
docker exec -it hwp-web /bin/sh

# 네트워크 디버깅
curl -v http://localhost:3000/health
```

---

## 12. Checklist

### 12.1 Pre-deployment

- [ ] 환경 변수 설정 완료
- [ ] SSL 인증서 준비
- [ ] DNS 설정 완료
- [ ] 방화벽 포트 오픈 (3000, 80, 443)
- [ ] 백업 전략 수립

### 12.2 Post-deployment

- [ ] Health check 정상
- [ ] 로그 수집 확인
- [ ] 모니터링 알람 설정
- [ ] 롤백 절차 테스트

---

**Next:** [MONITORING.md](./MONITORING.md) - 모니터링 설정 가이드
