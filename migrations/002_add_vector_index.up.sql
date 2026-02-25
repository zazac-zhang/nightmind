-- 添加向量搜索支持
-- Version: 002
-- Date: 2024-01-15
-- Depends: 001_initial

-- ============================================================================
-- 启用 pgvector 扩展
-- ============================================================================
CREATE EXTENSION IF NOT EXISTS vector;

-- ============================================================================
-- 为 knowledge_points 添加 embedding 列
-- ============================================================================
ALTER TABLE knowledge_points
ADD COLUMN IF NOT EXISTS embedding VECTOR(1536);

-- ============================================================================
-- 创建向量索引 (IVFFlat)
-- ============================================================================
-- 使用余弦距离的 IVFFlat 索引
-- lists 参数推荐为行数的平方根 (预估 100000 行 -> lists = 316，取整为 100)
CREATE INDEX IF NOT EXISTS idx_knowledge_points_embedding_cosine
    ON knowledge_points
    USING ivfflat (embedding vector_cosine_ops)
    WITH (lists = 100);

-- 可选：创建 L2 距离索引 (如果需要)
CREATE INDEX IF NOT EXISTS idx_knowledge_points_embedding_l2
    ON knowledge_points
    USING ivfflat (embedding vector_l2_ops)
    WITH (lists = 100);

-- 可选：创建内积索引 (如果需要)
CREATE INDEX IF NOT EXISTS idx_knowledge_points_embedding_ip
    ON knowledge_points
    USING ivfflat (embedding vector_ip_ops)
    WITH (lists = 100);

-- ============================================================================
-- 更新知识内容全文搜索 (支持中文)
-- ============================================================================
-- 为中文内容添加全文搜索支持
CREATE INDEX IF NOT EXISTS idx_knowledge_points_content_fts_chinese
    ON knowledge_points
    USING gin(to_tsvector('simple', content));

-- ============================================================================
-- 向量相似度搜索辅助函数
-- ============================================================================

-- 计算两个向量的余弦相似度
CREATE OR REPLACE FUNCTION cosine_similarity(v1 VECTOR, v2 VECTOR)
RETURNS FLOAT AS $$
BEGIN
    RETURN 1 - (v1 <=> v2);
END;
$$ LANGUAGE plpgsql IMMUTABLE STRICT;

-- 批量向量搜索 (返回最相似的 N 个知识点)
CREATE OR REPLACE FUNCTION search_similar_knowledge(
    p_user_id UUID,
    p_query_vector VECTOR(1536),
    p_limit INT DEFAULT 10,
    p_threshold FLOAT DEFAULT 0.7
)
RETURNS TABLE (
    id UUID,
    content TEXT,
    title VARCHAR(500),
    similarity FLOAT,
    content_type VARCHAR(50),
    tags TEXT[]
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        kp.id,
        kp.content,
        kp.title,
        cosine_similarity(p_query_vector, kp.embedding) AS similarity,
        kp.content_type,
        kp.tags
    FROM knowledge_points kp
    WHERE kp.user_id = p_user_id
        AND kp.embedding IS NOT NULL
        AND cosine_similarity(p_query_vector, kp.embedding) >= p_threshold
    ORDER BY kp.embedding <=> p_query_vector
    LIMIT p_limit;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- 更新后的待复习视图 (包含相似度建议)
-- ============================================================================
CREATE OR REPLACE VIEW v_due_reviews_with_context AS
SELECT
    dr.*,
    -- 可以在这里添加相关知识点推荐
    NULL::JSONB AS related_points
FROM v_due_reviews dr;

-- ============================================================================
-- 为 embeddings 添加注释
-- ============================================================================
COMMENT ON COLUMN knowledge_points.embedding IS
'OpenAI text-embedding-3-small 向量 (1536 维度)，用于语义相似度搜索';

COMMENT ON INDEX idx_knowledge_points_embedding_cosine IS
'IVFFlat 索引，使用余弦距离进行向量相似度搜索';

-- ============================================================================
-- 完成
-- ============================================================================
DO $$
BEGIN
    RAISE NOTICE 'Migration 002_add_vector_index completed successfully';
END $$;
