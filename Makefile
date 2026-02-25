# ============================================================
# NightMind Makefile
# ============================================================

.PHONY: help build dev test clean docker-up docker-down docker-logs db-migrate

# 默认目标
.DEFAULT_GOAL := help

# ============================================================
# 帮助信息
# ============================================================
help: ## 显示帮助信息
	@echo "NightMind 开发命令"
	@echo ""
	@echo "使用方法: make [target]"
	@echo ""
	@echo "可用命令:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'

# ============================================================
# 开发
# ============================================================
dev: ## 启动开发服务器
	cargo run

dev-watch: ## 启动开发服务器（热重载）
	cargo install cargo-watch 2>/dev/null || true
	cargo watch -x run

build: ## 构建项目
	cargo build --release

check: ## 运行检查（clippy + fmt）
	cargo clippy -- -D warnings
	cargo fmt -- --check

test: ## 运行测试
	cargo test --all-features

test-coverage: ## 生成测试覆盖率
	cargo install cargo-tarpaulin 2>/dev/null || true
	cargo tarpaulin --out Html --output-dir coverage

# ============================================================
# Docker
# ============================================================
docker-up: ## 启动所有 Docker 服务
	docker-compose up -d
	@echo "等待服务启动..."
	@sleep 5
	@echo "服务状态:"
	docker-compose ps

docker-down: ## 停止所有 Docker 服务
	docker-compose down

docker-restart: ## 重启所有 Docker 服务
	docker-compose restart

docker-logs: ## 查看 Docker 日志
	docker-compose logs -f

docker-logs-app: docker-logs ## 查看 NightMind 应用日志
docker-logs-postgres: ## 查看 PostgreSQL 日志
	docker-compose logs -f postgres
docker-logs-redis: ## 查看 Redis 日志
	docker-compose logs -f redis
docker-logs-qdrant: ## 查看 Qdrant 日志
	docker-compose logs -f qdrant

docker-build: ## 构建生产 Docker 镜像
	docker build -t nightmind:latest -f Dockerfile .

docker-build-dev: ## 构建开发 Docker 镜像
	docker build -t nightmind:dev -f Dockerfile.dev .

# ============================================================
# 工具服务
# ============================================================
tools-up: ## 启动开发工具（pgAdmin + Redis Commander）
	docker-compose --profile tools up -d

tools-down: ## 停止开发工具
	docker-compose --profile tools down

# ============================================================
# 数据库
# ============================================================
db-shell: ## 连接到 PostgreSQL Shell
	docker-compose exec postgres psql -U nightmind -d nightmind

db-migrate: ## 运行数据库迁移
	cargo install sqlx-cli 2>/dev/null || true
	sqlx database create
	sqlx migrate run

db-migrate-rollback: ## 回滚最后一次迁移
	cargo install sqlx-cli 2>/dev/null || true
	sqlx migrate revert

db-reset: ## 重置数据库（删除并重新创建）
	docker-compose down -v
	docker-compose up -d postgres redis qdrant
	@echo "等待数据库启动..."
	@sleep 10
	$(MAKE) db-migrate

db-backup: ## 备份数据库
	@mkdir -p backups
	docker-compose exec -T postgres pg_dump -U nightmind nightmind \
		| gzip > backups/nightmind_$(shell date +%Y%m%d_%H%M%S).sql.gz
	@echo "备份已保存到 backups/"

db-restore: ## 从备份恢复数据库（使用 FILE= 参数）
	@if [ -z "$(FILE)" ]; then \
		echo "请指定备份文件: make db-restore FILE=backups/nightmind_xxx.sql.gz"; \
		exit 1; \
	fi
	gunzip -c $(FILE) | docker-compose exec -T postgres psql -U nightmind -d nightmind

# ============================================================
# Redis
# ============================================================
redis-shell: ## 连接到 Redis Shell
	docker-compose exec redis redis-cli

redis-flush: ## 清空 Redis
	docker-compose exec redis redis-cli FLUSHALL

# ============================================================
# Qdrant
# ============================================================
qdrant-dashboard: ## 打开 Qdrant Dashboard
	@echo "Qdrant Dashboard: http://localhost:6333/dashboard"
	@open http://localhost:6333/dashboard 2>/dev/null || true

# ============================================================
# 代码质量
# ============================================================
fmt: ## 格式化代码
	cargo fmt

lint: ## 运行 linter
	cargo clippy --all-targets --all-features

audit: ## 审计依赖安全性
	cargo audit

docs: ## 生成文档
	cargo doc --no-deps --open

# ============================================================
# 清理
# ============================================================
clean: ## 清理构建产物
	cargo clean

clean-all: ## 清理所有（包括 Docker）
	$(MAKE) clean
	docker-compose down -v
	docker system prune -f

# ============================================================
# Git
# ============================================================
pre-commit: check ## 提交前检查（git hook）

init-git-hooks: ## 初始化 Git hooks
	@echo "安装 Git hooks..."
	@echo "#!/bin/sh\nmake check" > .git/hooks/pre-commit
	@chmod +x .git/hooks/pre-commit
	@echo "Git hooks 安装完成！"

# ============================================================
# 测试数据
# ============================================================
seed-db: ## 插入测试数据
	docker-compose exec -T postgres psql -U nightmind -d nightmind << SQL
-- 插入测试用户
INSERT INTO users (username, email, password_hash, preferences) VALUES
(
    'test_user',
    'test@example.com',
    '\$2a\$12\$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyYzW5qBOl1i',
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
deploy: docker-build ## 部署到生产（需要配置）
	@echo "部署功能需要额外配置..."
	@echo "请参考 doc/10-deployment.md"

# ============================================================
# 信息
# ============================================================
info: ## 显示项目信息
	@echo "NightMind - 睡前认知巩固 AI 伴学 Agent"
	@echo ""
	@echo "项目信息:"
	@echo "  版本: 0.1.0"
	@echo "  Rust: $(shell rustc --version)"
	@echo "  Docker: $(shell docker --version 2>/dev/null || echo '未安装')"
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
setup: ## 初始化开发环境
	@echo "初始化 NightMind 开发环境..."
	@cp .env.example .env 2>/dev/null || echo ".env 已存在"
	@$(MAKE) docker-up
	@echo "等待服务启动..."
	@sleep 10
	@$(MAKE) db-migrate
	@echo ""
	@echo "开发环境已就绪！"
	@$(MAKE) info

quickstart: ## 快速开始（setup + dev）
	@$(MAKE) setup
	@$(MAKE) dev
