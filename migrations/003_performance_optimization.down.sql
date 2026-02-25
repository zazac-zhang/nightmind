-- 回滚性能优化和实用函数
-- Version: 003
-- Date: 2024-01-15

-- ============================================================================
-- 删除注释
-- ============================================================================
COMMENT ON FUNCTION check_knowledge_access(p_user_id UUID, p_knowledge_id UUID) IS NULL;
COMMENT ON FUNCTION check_session_access(p_user_id UUID, p_session_id UUID) IS NULL;
COMMENT ON FUNCTION search_similar_knowledge(p_user_id UUID, p_query_vector VECTOR(1536), p_limit INT, p_threshold FLOAT) IS NULL;
COMMENT ON FUNCTION cleanup_old_snapshots(p_days INT) IS NULL;
COMMENT ON FUNCTION update_review_interval(p_interval_id UUID, p_rating INT, p_time_taken INT) IS NULL;

-- ============================================================================
-- 删除安全函数
-- ============================================================================
DROP FUNCTION IF EXISTS check_knowledge_access(p_user_id UUID, p_knowledge_id UUID) CASCADE;
DROP FUNCTION IF EXISTS check_session_access(p_user_id UUID, p_session_id UUID) CASCADE;

-- ============================================================================
-- 删除定时任务 (如果有)
-- ============================================================================
-- SELECT cron.unschedule('cleanup-old-snapshots');

-- ============================================================================
-- 删除视图
-- ============================================================================
DROP VIEW IF EXISTS v_table_sizes CASCADE;
DROP VIEW IF EXISTS v_learning_progress CASCADE;
DROP VIEW IF EXISTS v_daily_activity CASCADE;

-- ============================================================================
-- 删除函数
-- ============================================================================
DROP FUNCTION IF EXISTS update_review_interval(p_interval_id UUID, p_rating INT, p_time_taken INT) CASCADE;
DROP FUNCTION IF EXISTS remove_knowledge_relation(p_point_id UUID, p_related_id UUID) CASCADE;
DROP FUNCTION IF EXISTS add_knowledge_relation(p_point_id UUID, p_related_id UUID) CASCADE;
DROP FUNCTION IF EXISTS cleanup_old_sessions(p_days INT) CASCADE;
DROP FUNCTION IF EXISTS cleanup_old_snapshots(p_days INT) CASCADE;
DROP FUNCTION IF EXISTS update_session_metrics(p_session_id UUID) CASCADE;

-- ============================================================================
-- 删除索引
-- ============================================================================
DROP INDEX IF EXISTS idx_sessions_user_phase_started CASCADE;
DROP INDEX IF EXISTS idx_knowledge_points_user_source_created CASCADE;
DROP INDEX IF EXISTS idx_anki_cards_pending_sync CASCADE;
DROP INDEX IF EXISTS idx_sessions_active CASCADE;
DROP INDEX IF EXISTS idx_snapshots_recent CASCADE;

-- ============================================================================
-- 完成
-- ============================================================================
DO $$
BEGIN
    RAISE NOTICE 'Rollback 003_performance_optimization completed successfully';
END $$;
