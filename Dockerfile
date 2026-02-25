# ============================================================
# 生产环境 Dockerfile
# 多阶段构建，优化镜像大小
# ============================================================

# ============================================================
# 构建阶段
# ============================================================
FROM rust:1.83-alpine AS builder

# 安装构建依赖
RUN apk add --no-cache \
    musl-dev \
    pkgconf \
    openssl-dev \
    git

WORKDIR /app

# 复制 Cargo 文件
COPY Cargo.toml Cargo.lock ./

# 创建虚拟 src 目录来预编译依赖
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# 复制真实源码
COPY src ./src

# 构建应用
RUN touch src/main.rs && \
    cargo build --release && \
    strip /app/target/release/nightmind

# ============================================================
# 运行时阶段
# ============================================================
FROM alpine:latest

# 安装运行时依赖
RUN apk add --no-cache \
    ca-certificates \
    libgcc \
    curl \
    tzdata \
    && rm -rf /var/cache/apk/*

# 设置时区
ENV TZ=UTC

WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=builder /app/target/release/nightmind /app/nightmind

# 创建非 root 用户
RUN addgroup -g 1000 nightmind && \
    adduser -D -u 1000 -G nightmind nightmind && \
    chown -R nightmind:nightmind /app

USER nightmind

# 暴露端口
EXPOSE 8080

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# 启动应用
CMD ["./nightmind"]

# ============================================================
# 构建命令:
# docker build -t nightmind:latest -f Dockerfile .
# ============================================================
# 运行命令:
# docker run -p 8080:8080 --env-file .env nightmind:latest
# ============================================================
