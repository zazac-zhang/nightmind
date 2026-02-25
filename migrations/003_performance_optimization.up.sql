-- 性能优化和实用函数
-- Version: 003
-- Date: 2024-01-15
-- Depends: 002_add_vector_index

-- ============================================================================
-- 部分索引 (仅索引活跃数据)
-- ============================================================================

-- 仅索引进行中的会话
CREATE INDEX IF NOT EXISTS idx_sessions_active
    ON sessions(user_id, started_at DESC)
    WHERE ended_at IS NULL;

-- 仅索引待同步的 Anki 卡片
CREATE INDEX IF NOT EXISTS idx_anki_cards_pending_sync
    ON anki_cards(user_id, created_at DESC)
    WHERE sync_status = 'pending';

-- 仅索引未过期的快照 (24小时内)
CREATE INDEX IF NOT EXISTS idx_snapshots_recent
    ON snapshots(session_id, timestamp DESC)
    WHERE timestamp > NOW() - INTERVAL '24 hours';

-- ============================================================================
-- 复合索引 (优化常见查询)
-- ============================================================================

-- 用户知识点 + 来源 + 创建时间 (用于同步查询)
CREATE INDEX IF NOT EXISTS idx_knowledge_points_user_source_created
    ON knowledge_points(user_id, source_type, created_at DESC);

-- 会话用户 + 阶段 + 开始时间 (用于会话列表查询)
CREATE INDEX IF NOT EXISTS idx_sessions_user_phase_started
    ON sessions(user_id, phase, started_at DESC);

-- ============================================================================
-- 统计更新函数
-- ============================================================================

-- 更新会话指标
CREATE OR REPLACE FUNCTION update_session_metrics(
    p_session_id UUID
) RETURNS VOID AS $$
DECLARE
    v_metrics JSONB;
BEGIN
    -- 这里可以添加更复杂的指标计算逻辑
    -- 目前仅更新时间戳
    UPDATE sessions
    SET metrics = COALESCE(metrics, '{}'::jsonb) || jsonb_build_object(
        'last_updated', NOW()
    )
    WHERE id = p_session_id;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- 数据清理函数
-- ============================================================================

-- 清理过期快照
CREATE OR REPLACE FUNCTION cleanup_old_snapshots(
    p_days INT DEFAULT 30
) RETURNS INT AS $$
DECLARE
    v_deleted_count INT;
BEGIN
    DELETE FROM snapshots
    WHERE timestamp < NOW() - (p_days || ' days')::INTERVAL;

    GET DIAGNOSTICS v_deleted_count = ROW_COUNT;
    RETURN v_deleted_count;
END;
$$ LANGUAGE plpgsql;

-- 清理旧会话 (可选)
CREATE OR REPLACE FUNCTION cleanup_old_sessions(
    p_days INT DEFAULT 90
) RETURNS INT AS $$
DECLARE
    v_deleted_count INT;
BEGIN
    DELETE FROM sessions
    WHERE ended_at < NOW() - (p_days || ' days')::INTERVAL
        AND phase = 'closed';

    GET DIAGNOSTICS v_deleted_count = ROW_COUNT;
    RETURN v_deleted_count;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- 知识点关联函数
-- ============================================================================

-- 添加知识点关联
CREATE OR REPLACE FUNCTION add_knowledge_relation(
    p_point_id UUID,
    p_related_id UUID
) RETURNS VOID AS $$
BEGIN
    UPDATE knowledge_points
    SET related_ids = array_append(
        array_remove(related_ids, p_related_id),
        p_related_id
    )
    WHERE id = p_point_id
        AND p_related_id != id;  -- 防止自引用
END;
$$ LANGUAGE plpgsql;

-- 移除知识点关联
CREATE OR REPLACE FUNCTION remove_knowledge_relation(
    p_point_id UUID,
    p_related_id UUID
) RETURNS VOID AS $$
BEGIN
    UPDATE knowledge_points
    SET related_ids = array_remove(related_id, p_related_id)
    WHERE id = p_point_id;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- 复习调度函数 (FSRS 简化版)
-- ============================================================================

-- 更新复习间隔
CREATE OR REPLACE FUNCTION update_review_interval(
    p_interval_id UUID,
    p_rating INT,  -- 1=Again, 2=Hard, 3=Good, 4=Easy
    p_time_taken INT DEFAULT 0  -- 秒
) RETURNS VOID AS $$
DECLARE
    v_interval RECORD;
    v_new_interval FLOAT;
    v_new_ease FLOAT;
BEGIN
    SELECT * INTO v_interval
    FROM review_intervals
    WHERE id = p_interval_id;

    -- 简化的 FSRS 算法
    -- 实际使用时应该使用完整的 FSRS 公式
    CASE p_rating
        WHEN 1 THEN  -- Again
            v_new_interval := 1.0;
            v_new_ease := GREATEST(v_interval.ease_factor - 0.2, 1.3);
            v_interval.lapses := v_interval.lapses + 1;
        WHEN 2 THEN  -- Hard
            v_new_interval := v_interval.interval_days * 1.2;
            v_new_ease := v_interval.ease_factor - 0.15;
        WHEN 3 THEN  -- Good
            v_new_interval := v_interval.interval_days * v_interval.ease_factor;
            v_new_ease := v_interval.ease_factor;
        WHEN 4 THEN  -- Easy
            v_new_interval := v_interval.interval_days * v_interval.ease_factor * 1.3;
            v_new_ease := v_interval.ease_factor + 0.1;
        ELSE
            RAISE EXCEPTION 'Invalid rating: %', p_rating;
    END CASE;

    -- 更新间隔
    UPDATE review_intervals
    SET interval_days = v_new_interval,
        ease_factor = v_new_ease,
        next_review_date = CURRENT_DATE + (v_new_interval || ' days')::INTERVAL,
        last_review_date = CURRENT_DATE,
        total_reviews = total_reviews + 1,
        correct_reviews = correct_reviews + CASE WHEN p_rating >= 3 THEN 1 ELSE 0 END,
        updated_at = NOW()
    WHERE id = p_interval_id;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- 用户统计视图
-- ============================================================================

-- 每日活动统计
CREATE OR REPLACE VIEW v_daily_activity AS
SELECT
    user_id,
    DATE(started_at) AS activity_date,
    COUNT(*) AS sessions_count,
    SUM(EXTRACT(EPOCH FROM (COALESCE(ended_at, NOW()) - started_at))) AS total_seconds,
    COUNT(DISTINCT phase) AS phases_completed
FROM sessions
WHERE started_at >= CURRENT_DATE - INTERVAL '30 days'
GROUP BY user_id, DATE(started_at)
ORDER BY user_id, activity_date DESC;

-- 学习进度统计
CREATE OR REPLACE VIEW v_learning_progress AS
SELECT
    kp.user_id,
    COUNT(*) AS total_points,
    COUNT(*) FILTER (WHERE kp.last_reviewed_at IS NOT NULL) AS reviewed_points,
    COUNT(*) FILTER (WHERE ri.next_review_date <= CURRENT_DATE) AS due_today,
    AVG(ri.interval_days) AS avg_interval,
    AVG(ri.ease_factor) AS avg_ease,
    SUM(ri.total_reviews) AS total_reviews
FROM knowledge_points kp
LEFT JOIN review_intervals ri ON kp.id = ri.knowledge_point_id
GROUP BY kp.user_id;

-- ============================================================================
-- 性能监控视图
-- ============================================================================

-- 表大小统计
CREATE OR REPLACE VIEW v_table_sizes AS
SELECT
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS total_size,
    pg_size_pretty(pg_relation_size(schemaname||'.'||tablename)) AS table_size,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename) - pg_relation_size(schemaname||'.'||tablename)) AS index_size
FROM pg_tables
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;

-- 慢查询统计 (需要 pg_stat_statements 扩展)
-- CREATE EXTENSION IF NOT EXISTS pg_stat_statements;
-- CREATE OR REPLACE VIEW v_slow_queries AS
-- SELECT
--     query,
--     calls,
--     total_exec_time / 1000 AS total_time_seconds,
--     mean_exec_time / 1000 AS avg_time_seconds,
--     stddev_exec_time / 1000 AS stddev_time_seconds
-- FROM pg_stat_statements
-- ORDER BY mean_exec_time DESC
-- LIMIT 20;

-- ============================================================================
-- 定时任务 (需要 pg_cron 扩展，可选)
-- ============================================================================
-- CREATE EXTENSION IF NOT EXISTS pg_cron;

-- 每天凌晨 2 点清理 30 天前的快照
-- SELECT cron.schedule('cleanup-old-snapshots', '0 2 * * *',
--     'SELECT cleanup_old_snapshots(30);'
-- );

-- ============================================================================
-- 安全函数
-- ============================================================================

-- 检查用户是否有权限访问指定会话
CREATE OR REPLACE FUNCTION check_session_access(
    p_user_id UUID,
    p_session_id UUID
) RETURNS BOOLEAN AS $$
DECLARE
    v_exists INT;
BEGIN
    SELECT 1 INTO v_exists
    FROM sessions
    WHERE id = p_session_id AND user_id = p_user_id;

    RETURN COALESCE(v_exists, 0) = 1;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- 检查用户是否有权限访问指定知识点
CREATE OR REPLACE FUNCTION check_knowledge_access(
    p_user_id UUID,
    p_knowledge_id UUID
) RETURNS BOOLEAN AS $$
DECLARE
    v_exists INT;
BEGIN
    SELECT 1 INTO v_exists
    FROM knowledge_points
    WHERE id = p_knowledge_id AND user_id = p_user_id;

    RETURN COALESCE(v_exists, 0) = 1;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- ============================================================================
-- 为列添加注释
-- ============================================================================

COMMENT ON FUNCTION update_review_interval(p_interval_id UUID, p_rating INT, p_time_taken INT) IS
'更新复习间隔，使用简化版 FSRS 算法。rating: 1=Again, 2=Hard, 3=Good, 4=Easy';

COMMENT ON FUNCTION cleanup_old_snapshots(p_days INT) IS
'清理指定天数之前的快照，默认 30 天';

COMMENT ON FUNCTION search_similar_knowledge(p_user_id UUID, p_query_vector VECTOR(1536), p_limit INT, p_threshold FLOAT) IS
'基于向量相似度搜索知识点，返回最相似的 N 个结果';

COMMENT ON FUNCTION check_session_access(p_user_id UUID, p_session_id UUID) IS
'检查用户是否有权限访问指定会话 (安全函数)';

COMMENT ON FUNCTION check_knowledge_access(p_user_id UUID, p_knowledge_id UUID) IS
'检查用户是否有权限访问指定知识点 (安全函数)';

-- ============================================================================
-- 完成
-- ============================================================================
DO $$
BEGIN
    RAISE NOTICE 'Migration 003_performance_optimization completed successfully';
END $$;
