-- NightMind 初始数据库 Schema
-- Version: 001
-- Date: 2024-01-15

-- 启用必要扩展
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";

-- ============================================================================
-- 用户表
-- ============================================================================
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(100) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    -- 用户偏好 (JSONB)
    preferences JSONB DEFAULT '{}',

    -- 外部集成配置 (JSONB)
    integration_config JSONB DEFAULT '{}'
);

-- 用户偏好索引
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);

-- ============================================================================
-- 会话表
-- ============================================================================
CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- 时间
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at TIMESTAMPTZ,
    planned_duration INT DEFAULT 3600,  -- 计划时长（秒）

    -- 状态
    phase VARCHAR(50) NOT NULL DEFAULT 'warmup',  -- warmup, deep_dive, review, seed, closing, closed
    state JSONB NOT NULL DEFAULT '{}',

    -- 内容
    topics_discussed JSONB DEFAULT '[]',  -- 话题列表
    insights_generated JSONB DEFAULT '[]', -- 产生的洞察

    -- 指标
    metrics JSONB DEFAULT '{}',

    -- 外部同步状态
    sync_status JSONB DEFAULT '{}'
);

-- 会话索引
CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_started_at ON sessions(started_at DESC);
CREATE INDEX idx_sessions_ended_at ON sessions(ended_at DESC);
CREATE INDEX idx_sessions_phase ON sessions(phase);
CREATE INDEX idx_sessions_user_started ON sessions(user_id, started_at DESC);

-- ============================================================================
-- 知识点表
-- ============================================================================
CREATE TABLE knowledge_points (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- 内容
    content TEXT NOT NULL,
    content_type VARCHAR(50) NOT NULL,  -- concept, fact, procedure, story

    -- 来源
    source_type VARCHAR(50) NOT NULL,   -- anki, obsidian, notion, readwise, manual
    source_id VARCHAR(255),             -- 原始 ID

    -- 向量 (需要 pgvector 扩展，在下一个迁移中添加)
    -- embedding VECTOR(1536),

    -- 元数据
    title VARCHAR(500),
    summary TEXT,
    tags TEXT[] DEFAULT '{}',

    -- 关系
    parent_id UUID REFERENCES knowledge_points(id) ON DELETE SET NULL,
    related_ids UUID[] DEFAULT '{}',

    -- 时间
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    last_reviewed_at TIMESTAMPTZ
);

-- 知识点索引
CREATE INDEX idx_knowledge_points_user_id ON knowledge_points(user_id);
CREATE INDEX idx_knowledge_points_source ON knowledge_points(source_type);
CREATE INDEX idx_knowledge_points_tags ON knowledge_points USING GIN(tags);
CREATE INDEX idx_knowledge_points_last_reviewed ON knowledge_points(last_reviewed_at DESC);
CREATE INDEX idx_knowledge_points_content_type ON knowledge_points(content_type);
CREATE INDEX idx_knowledge_points_parent_id ON knowledge_points(parent_id);

-- 全文搜索索引
CREATE INDEX idx_knowledge_points_content_fts
    ON knowledge_points
    USING gin(to_tsvector('english', content));

-- 标签全文搜索
CREATE INDEX idx_knowledge_points_tags_fts
    ON knowledge_points
    USING gin(to_tsvector('english', array_to_string(tags, ' ')));

-- ============================================================================
-- 复习间隔表 (FSRS)
-- ============================================================================
CREATE TABLE review_intervals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    knowledge_point_id UUID NOT NULL REFERENCES knowledge_points(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- FSRS 参数
    interval_days FLOAT NOT NULL DEFAULT 1.0,
    ease_factor FLOAT NOT NULL DEFAULT 2.5,
    stability FLOAT,
    retrievability FLOAT,

    -- 调度
    next_review_date DATE NOT NULL,
    last_review_date DATE,

    -- 统计
    total_reviews INT DEFAULT 0,
    correct_reviews INT DEFAULT 0,
    lapses INT DEFAULT 0,

    -- 时间
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(knowledge_point_id, user_id)
);

-- 复习间隔索引
CREATE INDEX idx_review_intervals_user_next ON review_intervals(user_id, next_review_date);
CREATE INDEX idx_review_intervals_knowledge ON review_intervals(knowledge_point_id);
CREATE INDEX idx_review_intervals_next_review ON review_intervals(next_review_date)
    WHERE next_review_date <= CURRENT_DATE;

-- ============================================================================
-- 快照表
-- ============================================================================
CREATE TABLE snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,

    -- 快照数据
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    state JSONB NOT NULL,
    topic_stack JSONB NOT NULL,
    context JSONB NOT NULL,
    metrics JSONB NOT NULL
);

-- 快照索引
CREATE INDEX idx_snapshots_session_id ON snapshots(session_id);
CREATE INDEX idx_snapshots_timestamp ON snapshots(timestamp DESC);
CREATE INDEX idx_snapshots_session_timestamp ON snapshots(session_id, timestamp DESC);

-- ============================================================================
-- Anki 卡片表
-- ============================================================================
CREATE TABLE anki_cards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- Anki 数据
    anki_card_id BIGINT UNIQUE,
    anki_note_id BIGINT,
    anki_deck VARCHAR(255),

    -- 本地数据
    front TEXT NOT NULL,
    back TEXT NOT NULL,
    tags TEXT[] DEFAULT '{}',

    -- 同步状态
    sync_status VARCHAR(50) DEFAULT 'pending',  -- pending, synced, failed

    -- 时间
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    last_synced_at TIMESTAMPTZ
);

-- Anki 卡片索引
CREATE INDEX idx_anki_cards_user_id ON anki_cards(user_id);
CREATE INDEX idx_anki_cards_anki_id ON anki_cards(anki_card_id);
CREATE INDEX idx_anki_cards_sync_status ON anki_cards(sync_status);
CREATE INDEX idx_anki_cards_user_sync ON anki_cards(user_id, sync_status);

-- ============================================================================
-- 触发器: 自动更新 updated_at
-- ============================================================================
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- 为需要的表添加触发器
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_knowledge_points_updated_at BEFORE UPDATE ON knowledge_points
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_review_intervals_updated_at BEFORE UPDATE ON review_intervals
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_anki_cards_updated_at BEFORE UPDATE ON anki_cards
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- 视图: 待复习知识点
-- ============================================================================
CREATE OR REPLACE VIEW v_due_reviews AS
SELECT
    kp.id,
    kp.user_id,
    kp.content,
    kp.content_type,
    kp.title,
    kp.tags,
    ri.interval_days,
    ri.ease_factor,
    ri.stability,
    ri.retrievability,
    ri.next_review_date,
    ri.total_reviews,
    ri.lapses,
    EXTRACT(DAY FROM (CURRENT_DATE - ri.next_review_date)) AS overdue_days
FROM knowledge_points kp
INNER JOIN review_intervals ri ON kp.id = ri.knowledge_point_id
WHERE ri.next_review_date <= CURRENT_DATE
ORDER BY ri.next_review_date ASC, ri.retrievability ASC;

-- ============================================================================
-- 视图: 用户活动统计
-- ============================================================================
CREATE OR REPLACE VIEW v_user_activity AS
SELECT
    u.id AS user_id,
    u.username,
    COUNT(s.id) AS total_sessions,
    COUNT(s.id) FILTER (WHERE s.ended_at IS NOT NULL) AS completed_sessions,
    SUM(EXTRACT(EPOCH FROM (s.ended_at - s.started_at))) AS total_session_seconds,
    MAX(s.started_at) AS last_session_at
FROM users u
LEFT JOIN sessions s ON u.id = s.user_id
GROUP BY u.id, u.username;

-- ============================================================================
-- 示例数据 (可选，用于开发测试)
-- ============================================================================
-- 插入测试用户
INSERT INTO users (username, email, password_hash, preferences) VALUES
(
    'test_user',
    'test@example.com',
    '$2a$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyYzW5qBOl1i', -- "password" 的 bcrypt hash
    '{
        "voice": {"gender": "female", "speed": 1.0, "provider": "elevenlabs"},
        "session": {"default_duration": 3600, "auto_start_time": "22:00", "fatigue_threshold": 70},
        "content": {"density_level": "medium", "metaphor_preference": "lifestyle"}
    }'::jsonb
) ON CONFLICT (email) DO NOTHING;

-- ============================================================================
-- 完成
-- ============================================================================
-- 迁移完成标记
DO $$
BEGIN
    RAISE NOTICE 'Migration 001_initial completed successfully';
END $$;
