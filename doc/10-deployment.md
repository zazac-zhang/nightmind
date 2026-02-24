# 部署方案

## 1. 概述

NightMind 支持多种部署方式，从单机到云原生。

---

## 2. 部署架构

### 2.1 单机部署 (MVP)

```
┌─────────────────────────────────────────────────────────┐
│                    Single Server                        │
│  ┌─────────────────────────────────────────────────┐   │
│  │              NightMind (Axum + Rig)             │   │
│  └─────────────────────────────────────────────────┘   │
│                                                         │
│  ┌────────┐  ┌──────┐  ┌──────────┐                 │
│  │  PG    │  │Redis │  │  Qdrant  │                 │
│  └────────┘  └──────┘  └──────────┘                 │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

**适用场景**:
- 开发/测试环境
- 小规模部署 (< 100 用户)
- 单用户自托管

**资源配置**:
- CPU: 4 核
- 内存: 8GB
- 存储: 100GB SSD

### 2.2 云原生部署

```
                      ┌─────────────┐
                      │   Ingress   │
                      └──────┬──────┘
                             │
              ┌──────────────┼──────────────┐
              ▼              ▼              ▼
        ┌─────────┐    ┌─────────┐    ┌─────────┐
        │   Pod   │    │   Pod   │    │   Pod   │
        └─────────┘    └─────────┘    └─────────┘
              │              │              │
              └──────────────┼──────────────┘
                             ▼
                    ┌────────────────┐
                    │    Service     │
                    └────────┬───────┘
                             │
        ┌────────────┬───────┼────────┬───────────┐
        ▼            ▼       ▼        ▼           ▼
    ┌────────┐ ┌────────┐ ┌──────┐ ┌─────────┐ ┌───────┐
    │   PG   │ │ Redis  │ │Qdrant│ │   S3    │ │Prometheus│
    │ Primary│ │Cluster │ │Cluster│  Storage │  │Grafana │
    └────────┘ └────────┘ └──────┘ └─────────┘ └───────┘
```

**适用场景**:
- 生产环境
- 高可用要求
- 水平扩展需求

---

## 3. 容器化

### 3.1 Dockerfile

```dockerfile
# 构建阶段
FROM rust:1.83-alpine AS builder

WORKDIR /app

# 安装依赖
RUN apk add --no-cache \
    musl-dev \
    pkgconf \
    libressl-dev \
    postgresql-dev

# 复制依赖文件
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# 构建（优化）
RUN cargo build --release

# 运行阶段
FROM alpine:latest

WORKDIR /app

# 安装运行时依赖
RUN apk add --no-cache \
    ca-certificates \
    libgcc

# 复制二进制文件
COPY --from=builder /app/target/release/nightmind /app/nightmind

# 创建非 root 用户
RUN addgroup -g 1000 nightmind && \
    adduser -D -u 1000 -G nightmind nightmind

USER nightmind

# 暴露端口
EXPOSE 8080

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8080/health || exit 1

# 启动
CMD ["./nightmind"]
```

### 3.2 Docker Compose

```yaml
version: '3.8'

services:
  nightmind:
    build: .
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=postgresql://nightmind:password@postgres:5432/nightmind
      - REDIS_URL=redis://redis:6379
      - QDRANT_URL=http://qdrant:6334
      - RUST_LOG=info
    depends_on:
      - postgres
      - redis
      - qdrant
    restart: unless-stopped

  postgres:
    image: postgres:16-alpine
    environment:
      - POSTGRES_DB=nightmind
      - POSTGRES_USER=nightmind
      - POSTGRES_PASSWORD=password
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./migrations:/docker-entrypoint-initdb.d
    restart: unless-stopped

  redis:
    image: redis:7-alpine
    command: redis-server --appendonly yes
    volumes:
      - redis_data:/data
    restart: unless-stopped

  qdrant:
    image: qdrant/qdrant:v1.8.0
    ports:
      - "6333:6333"
      - "6334:6334"
    volumes:
      - qdrant_data:/qdrant/storage
    restart: unless-stopped

volumes:
  postgres_data:
  redis_data:
  qdrant_data:
```

---

## 4. Kubernetes 部署

### 4.1 Namespace

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: nightmind
  labels:
    name: nightmind
```

### 4.2 ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: nightmind-config
  namespace: nightmind
data:
  RUST_LOG: "info"
  QDRANT_URL: "http://qdrant:6334"
```

### 4.3 Secret

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: nightmind-secrets
  namespace: nightmind
type: Opaque
stringData:
  DATABASE_URL: "postgresql://nightmind:password@postgres:5432/nightmind"
  REDIS_URL: "redis://redis:6379"
  OPENAI_API_KEY: "sk-..."
  ELEVENLABS_API_KEY: "..."
```

### 4.4 Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: nightmind
  namespace: nightmind
spec:
  replicas: 3
  selector:
    matchLabels:
      app: nightmind
  template:
    metadata:
      labels:
        app: nightmind
    spec:
      containers:
      - name: nightmind
        image: nightmind:latest
        ports:
        - containerPort: 8080
          name: http
        envFrom:
        - configMapRef:
            name: nightmind-config
        - secretRef:
            name: nightmind-secrets
        resources:
          requests:
            cpu: 500m
            memory: 512Mi
          limits:
            cpu: 2000m
            memory: 2Gi
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
```

### 4.5 Service

```yaml
apiVersion: v1
kind: Service
metadata:
  name: nightmind
  namespace: nightmind
spec:
  selector:
    app: nightmind
  ports:
  - port: 80
    targetPort: 8080
    name: http
  type: ClusterIP
```

### 4.6 Ingress

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: nightmind
  namespace: nightmind
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/websocket-services: nightmind
    nginx.ingress.kubernetes.io/proxy-read-timeout: "3600"
    nginx.ingress.kubernetes.io/proxy-send-timeout: "3600"
spec:
  ingressClassName: nginx
  tls:
  - hosts:
    - api.nightmind.dev
    secretName: nightmind-tls
  rules:
  - host: api.nightmind.dev
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: nightmind
            port:
              number: 80
```

### 4.7 HPA (自动扩缩容)

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: nightmind-hpa
  namespace: nightmind
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: nightmind
  minReplicas: 2
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

---

## 5. 监控与可观测性

### 5.1 Prometheus 指标

```rust
use prometheus::{Counter, Histogram, Gauge};

lazy_static! {
    // 请求计数
    static ref REQUESTS_TOTAL: Counter = Counter::new(
        "nightmind_requests_total",
        "Total number of requests"
    ).unwrap();

    // 响应时间
    static ref RESPONSE_TIME: Histogram = Histogram::new(
        "nightmind_response_time_seconds",
        "Response time in seconds"
    ).unwrap();

    // 活跃会话数
    static ref ACTIVE_SESSIONS: Gauge = Gauge::new(
        "nightmind_active_sessions",
        "Number of active sessions"
    ).unwrap();

    // LLM Token 消耗
    static ref LLM_TOKENS_TOTAL: Counter = Counter::new(
        "nightmind_llm_tokens_total",
        "Total number of LLM tokens consumed"
    ).unwrap();
}

// 在代码中使用
REQUESTS_TOTAL.inc();
RESPONSE_TIME.observe(duration.as_secs_f64());
```

### 5.2 Grafana Dashboard

```json
{
  "dashboard": {
    "title": "NightMind Dashboard",
    "panels": [
      {
        "title": "Request Rate",
        "targets": [{
          "expr": "rate(nightmind_requests_total[5m])"
        }]
      },
      {
        "title": "Response Time",
        "targets": [{
          "expr": "histogram_quantile(0.95, nightmind_response_time_seconds)"
        }]
      },
      {
        "title": "Active Sessions",
        "targets": [{
          "expr": "nightmind_active_sessions"
        }]
      },
      {
        "title": "LLM Token Usage",
        "targets": [{
          "expr": "rate(nightmind_llm_tokens_total[1h])"
        }]
      }
    ]
  }
}
```

### 5.3 分布式追踪

```rust
use tracing::{info, instrument};
use tracing_opentelemetry::OpenTelemetryLayer;

#[instrument(skip(self))]
pub async fn handle_transcript(&self, session_id: Uuid, transcript: String) {
    info!(session_id = %session_id, transcript_length = transcript.len(), "Handling transcript");

    // 自动生成 span
}
```

---

## 6. 日志管理

### 6.1 结构化日志

```rust
use tracing::{info, warn, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn init_logging() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_level(true)
        )
        .with(tracing_opentelemetry::layer())
        .init();
}

// 使用
info!(user_id = %user_id, session_count = 3, "User created session");
warn!(fatigue_score = 75, "User fatigue threshold approaching");
error!(error = %err, "Failed to transcribe audio");
```

### 6.2 日志输出格式

```json
{
  "timestamp": "2024-01-15T22:30:00Z",
  "level": "INFO",
  "target": "nightmind::session::manager",
  "fields": {
    "message": "Session created",
    "user_id": "999e4567-e89b-12d3-a456-426614174000",
    "session_id": "123e4567-e89b-12d3-a456-426614174000"
  },
  "span": {
    "id": 123456789,
    "name": "create_session"
  }
}
```

---

## 7. CI/CD

### 7.1 GitHub Actions

```yaml
name: CI/CD

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_DB: nightmind_test
          POSTGRES_USER: test
          POSTGRES_PASSWORD: test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    steps:
    - uses: actions/checkout@v4
    - uses: actions-rust-lang/setup-rust-toolchain@v1
    - name: Run tests
      env:
        DATABASE_URL: postgresql://test:test@localhost/nightmind_test
      run: |
        cargo test --all-features
        cargo clippy -- -D warnings
        cargo fmt -- --check

  build:
    needs: test
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - uses: docker/setup-buildx-action@v3
    - uses: docker/login-action@v3
      with:
        registry: ghcr.io
        username: ${{ github.actor }}
        password: ${{ secrets.GITHUB_TOKEN }}
    - uses: docker/build-push-action@v5
      with:
        context: .
        push: true
        tags: ghcr.io/${{ github.repository }}:latest
        cache-from: type=gha
        cache-to: type=gha,mode=max
```

---

## 8. 备份与恢复

### 8.1 数据库备份

```bash
#!/bin/bash
# backup.sh

BACKUP_DIR="/backups/postgres"
DATE=$(date +%Y%m%d_%H%M%S)
FILENAME="nightmind_${DATE}.sql.gz"

pg_dump -h postgres -U nightmind -d nightmind | gzip > "${BACKUP_DIR}/${FILENAME}"

# 保留最近 30 天的备份
find ${BACKUP_DIR} -name "nightmind_*.sql.gz" -mtime +30 -delete
```

### 8.2 Redis 备份

```bash
# 触发 RDB 快照
redis-cli BGSAVE

# 复制 RDB 文件
cp /var/lib/redis/dump.rdb /backups/redis/dump_$(date +%Y%m%d).rdb
```

---

## 9. 安全配置

### 9.1 网络策略

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: nightmind-netpol
  namespace: nightmind
spec:
  podSelector:
    matchLabels:
      app: nightmind
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: ingress-nginx
    ports:
    - protocol: TCP
      port: 8080
  egress:
  - to:
    - podSelector:
        matchLabels:
          app: postgres
    ports:
    - protocol: TCP
      port: 5432
  - to:
    - podSelector:
        matchLabels:
          app: redis
    ports:
    - protocol: TCP
      port: 6379
```

### 9.2 Pod Security

```yaml
apiVersion: v1
kind: PodSecurityPolicy
metadata:
  name: nightmind-psp
spec:
  privileged: false
  runAsUser:
    rule: MustRunAsNonRoot
  seLinux:
    rule: RunAsAny
  fsGroup:
    rule: MustRunAs
    ranges:
    - min: 1000
      max: 65535
```
