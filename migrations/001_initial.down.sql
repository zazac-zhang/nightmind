-- NightMind 初始数据库 Schema 回滚
-- Version: 001
-- Date: 2024-01-15

-- ============================================================================
-- 删除视图
-- ============================================================================
DROP VIEW IF EXISTS v_user_activity CASCADE;
DROP VIEW IF EXISTS v_due_reviews CASCADE;

-- ============================================================================
-- 删除触发器
-- ============================================================================
DROP TRIGGER IF EXISTS update_anki_cards_updated_at ON anki_cards;
DROP TRIGGER IF EXISTS update_review_intervals_updated_at ON review_intervals;
DROP TRIGGER IF EXISTS update_knowledge_points_updated_at ON knowledge_points;
DROP TRIGGER IF EXISTS update_users_updated_at ON users;

DROP FUNCTION IF EXISTS update_updated_at_column() CASCADE;

-- ============================================================================
-- 删除表 (按依赖关系逆序)
-- ============================================================================
DROP TABLE IF EXISTS anki_cards CASCADE;
DROP TABLE IF EXISTS snapshots CASCADE;
DROP TABLE IF EXISTS review_intervals CASCADE;
DROP TABLE IF EXISTS knowledge_points CASCADE;
DROP TABLE IF EXISTS sessions CASCADE;
DROP TABLE IF EXISTS users CASCADE;

-- ============================================================================
-- 删除扩展
-- ============================================================================
DROP EXTENSION IF EXISTS "pg_trgm" CASCADE;
DROP EXTENSION IF EXISTS "uuid-ossp" CASCADE;

-- ============================================================================
-- 完成
-- ============================================================================
DO $$
BEGIN
    RAISE NOTICE 'Rollback 001_initial completed successfully';
END $$;
