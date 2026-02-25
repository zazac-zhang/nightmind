# ============================================================
# NightMind Justfile
# ============================================================

# 默认目标
default:
    @just --list

# ============================================================
# 帮助信息
# ============================================================
# 显示帮助信息
alias help := default

# ============================================================
# 开发
# ============================================================
# 启动开发服务器
dev:
    cargo run

# 启动开发服务器（热重载）
dev-watch:
    cargo install cargo-watch 2>/dev/null || true
    cargo watch -x run

# 构建项目
build:
    cargo build --release

# 运行检查（clippy + fmt）
check:
    cargo clippy -- -D warnings
    cargo fmt -- --check

# 运行测试
test:
    cargo test --all-features

# 生成测试覆盖率
test-coverage:
    cargo install cargo-tarpaulin 2>/dev/null || true
    cargo tarpaulin --out Html --output-dir coverage

# ============================================================
# Docker
# ============================================================
# 启动所有 Docker 服务
docker-up:
    docker-compose up -d
    @echo "等待服务启动..."
    @sleep 5
    @echo "服务状态:"
    docker-compose ps

# 停止所有 Docker 服务
docker-down:
    docker-compose down

# 重启所有 Docker 服务
docker-restart:
    docker-compose restart

# 查看 Docker 日志
docker-logs *args="":
    docker-compose logs -f {{args}}

# 查看 NightMind 应用日志
docker-logs-app:
    just docker-logs

# 查看 PostgreSQL 日志
docker-logs-postgres:
    docker-compose logs -f postgres

# 查看 Redis 日志
docker-logs-redis:
    docker-compose logs -f redis

# 查看 Qdrant 日志
docker-logs-qdrant:
    docker-compose logs -f qdrant

# 构建生产 Docker 镜像
docker-build:
    docker build -t nightmind:latest -f Dockerfile .

# 构建开发 Docker 镜像
docker-build-dev:
    docker build -t nightmind:dev -f Dockerfile.dev .

# ============================================================
# 工具服务
# ============================================================
# 启动开发工具（pgAdmin + Redis Commander）
tools-up:
    docker-compose --profile tools up -d

# 停止开发工具
tools-down:
    docker-compose --profile tools down

# ============================================================
# 数据库
# ============================================================
# 连接到 PostgreSQL Shell
db-shell:
    docker-compose exec postgres psql -U nightmind -d nightmind

# 运行数据库迁移
db-migrate:
    cargo install sqlx-cli 2>/dev/null || true
    sqlx database create
    sqlx migrate run

# 回滚最后一次迁移
db-migrate-rollback:
    cargo install sqlx-cli 2>/dev/null || true
    sqlx migrate revert

# 重置数据库（删除并重新创建）
db-reset:
    just docker-down -v
    docker-compose up -d postgres redis qdrant
    @echo "等待数据库启动..."
    @sleep 10
    just db-migrate

# 备份数据库
db-backup:
    mkdir -p backups
    docker-compose exec -T postgres pg_dump -U nightmind nightmind | gzip > backups/nightmind_`date +%Y%m%d_%H%M%S`.sql.gz
    @echo "备份已保存到 backups/"

# 从备份恢复数据库
db-restore file:
    gunzip -c {{file}} | docker-compose exec -T postgres psql -U nightmind -d nightmind

# ============================================================
# Redis
# ============================================================
# 连接到 Redis Shell
redis-shell:
    docker-compose exec redis redis-cli

# 清空 Redis
redis-flush:
    docker-compose exec redis redis-cli FLUSHALL

# ============================================================
# Qdrant
# ============================================================
# 打开 Qdrant Dashboard
qdrant-dashboard:
    @echo "Qdrant Dashboard: http://localhost:6333/dashboard"
    open http://localhost:6333/dashboard 2>/dev/null || true

# ============================================================
# 代码质量
# ============================================================
# 格式化代码
fmt:
    cargo fmt

# 运行 linter
lint:
    cargo clippy --all-targets --all-features

# 审计依赖安全性
audit:
    cargo audit

# 生成文档
docs:
    cargo doc --no-deps --open

# ============================================================
# 清理
# ============================================================
# 清理构建产物
clean:
    cargo clean

# 清理所有（包括 Docker）
clean-all:
    just clean
    docker-compose down -v
    docker system prune -f

# ============================================================
# Git
# ============================================================
# 提交前检查（git hook）
pre-commit: check

# 初始化 Git hooks
init-git-hooks:
    @echo "安装 Git hooks..."
    @echo "#!/bin/sh\njust check" > .git/hooks/pre-commit
    @chmod +x .git/hooks/pre-commit
    @echo "Git hooks 安装完成！"

# ============================================================
# 测试数据
# ============================================================
# 插入测试数据
seed-db:
    docker-compose exec -T postgres psql -U nightmind -d nightmind <<SQL
    -- 插入测试用户
    INSERT INTO users (username, email, password_hash, preferences) VALUES
    (
        'test_user',
        'test@example.com',
        '$2a$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyYzW5qBOl1i',
        '{
            "voice": {"gender": "female", "speed": 1.0},
            "session": {"default_duration": 3600, "fatigue_threshold": 70}
        }'::jsonb
    ) ON CONFLICT (email) DO NOTHING;

    -- 插入测试知识点
    INSERT INTO knowledge_points (user_id, content, content_type, source_type, title, tags)
    SELECT
        (SELECT id FROM users WHERE email = 'test@example.com'),
        '装饰器是 Python 中用于修改函数或类行为的工具，就像给礼物加包装纸一样。',
        'concept',
        'manual',
        '装饰器概念',
        ARRAY['python', 'decorator']
    ON CONFLICT DO NOTHING;
SQL

# ============================================================
# 生产部署
# ============================================================
# 部署到生产（需要配置）
deploy: docker-build
    @echo "部署功能需要额外配置..."
    @echo "请参考 doc/10-deployment.md"

# ============================================================
# 信息
# ============================================================
# 显示项目信息
info:
    @echo "NightMind - 睡前认知巩固 AI 伴学 Agent"
    @echo ""
    @echo "项目信息:"
    @echo "  版本: 0.1.0"
    @echo "  Rust: `rustc --version`"
    @echo "  Docker: `docker --version 2>/dev/null || echo '未安装'`"
    @echo ""
    @echo "服务端口:"
    @echo "  NightMind: http://localhost:8080"
    @echo "  PostgreSQL: localhost:5432"
    @echo "  Redis: localhost:6379"
    @echo "  Qdrant: http://localhost:6333"
    @echo "  pgAdmin: http://localhost:5050"
    @echo "  Redis Commander: http://localhost:8081"

# ============================================================
# 快速开始
# ============================================================
# 初始化开发环境
setup:
    @echo "初始化 NightMind 开发环境..."
    cp .env.example .env 2>/dev/null || echo ".env 已存在"
    just docker-up
    @echo "等待服务启动..."
    @sleep 10
    just db-migrate
    @echo ""
    @echo "开发环境已就绪！"
    just info

# 快速开始（setup + dev）
quickstart: setup dev
