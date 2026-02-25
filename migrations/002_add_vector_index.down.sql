-- 回滚向量搜索支持
-- Version: 002
-- Date: 2024-01-15

-- ============================================================================
-- 删除函数
-- ============================================================================
DROP FUNCTION IF EXISTS search_similar_knowledge(
    p_user_id UUID,
    p_query_vector VECTOR(1536),
    p_limit INT,
    p_threshold FLOAT
) CASCADE;

DROP FUNCTION IF EXISTS cosine_similarity(v1 VECTOR, v2 VECTOR) CASCADE;

-- ============================================================================
-- 删除视图
-- ============================================================================
DROP VIEW IF EXISTS v_due_reviews_with_context CASCADE;

-- ============================================================================
-- 删除索引
-- ============================================================================
DROP INDEX IF EXISTS idx_knowledge_points_embedding_ip CASCADE;
DROP INDEX IF EXISTS idx_knowledge_points_embedding_l2 CASCADE;
DROP INDEX IF EXISTS idx_knowledge_points_embedding_cosine CASCADE;
DROP INDEX IF EXISTS idx_knowledge_points_content_fts_chinese CASCADE;

-- ============================================================================
-- 删除 embedding 列
-- ============================================================================
ALTER TABLE knowledge_points
DROP COLUMN IF EXISTS embedding;

-- ============================================================================
-- 删除 pgvector 扩展
-- ============================================================================
DROP EXTENSION IF EXISTS vector CASCADE;

-- ============================================================================
-- 完成
-- ============================================================================
DO $$
BEGIN
    RAISE NOTICE 'Rollback 002_add_vector_index completed successfully';
END $$;
