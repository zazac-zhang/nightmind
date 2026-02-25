#!/bin/bash
# PostgreSQL 初始化脚本
# 在容器启动时自动执行

set -e

echo "Initializing PostgreSQL extensions..."

# 启用 UUID 生成
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
    CREATE EXTENSION IF NOT EXISTS "pg_trgm";
EOSQL

echo "PostgreSQL extensions initialized successfully!"
