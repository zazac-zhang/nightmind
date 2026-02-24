# 数据模型设计

## 1. 概述

NightMind 使用 PostgreSQL 作为主数据库，Redis 作为缓存层，Qdrant 作为向量存储。

---

## 2. 数据库设计

### 2.1 ER 图

```
┌─────────────┐     ┌─────────────┐     ┌──────────────────┐
│    users    │────<│  sessions   │>────│   snapshots      │
└─────────────┘     └─────────────┘     └──────────────────┘
       │
       │
       ▼
┌───────────────────────────────────────────────────────────┐
│                    knowledge_points                       │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────────┐ │
│  │ embeddings   │  │  relations   │  │ review_intervals│ │
│  └──────────────┘  └──────────────┘  └─────────────────┘ │
└───────────────────────────────────────────────────────────┘
```

### 2.2 表结构定义

#### users

```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(100) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    -- 用户偏好
    preferences JSONB DEFAULT '{}',

    -- 外部集成配置
    integration_config JSONB DEFAULT '{}',

    INDEX idx_users_username (username),
    INDEX idx_users_email (email)
);

-- preferences 字段结构
{
    "voice": {
        "gender": "female",
        "speed": 1.0,
        "provider": "elevenlabs"
    },
    "session": {
        "default_duration": 3600,
        "auto_start_time": "22:00",
        "fatigue_threshold": 70
    },
    "content": {
        "density_level": "medium",
        "metaphor_preference": "lifestyle"
    }
}
```

#### sessions

```sql
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
    sync_status JSONB DEFAULT '{}',

    INDEX idx_sessions_user_id (user_id),
    INDEX idx_sessions_started_at (started_at DESC),
    INDEX idx_sessions_ended_at (ended_at DESC)
);

-- state 字段结构
{
    "current_topic": "装饰器概念",
    "topic_stack_depth": 2,
    "last_interaction": "2024-01-15T22:30:00Z",
    "cognitive_load": "medium",
    "fatigue_score": 45
}

-- metrics 字段结构
{
    "user_turns": 15,
    "agent_turns": 20,
    "avg_response_time": 3.5,
    "total_silence": 120,
    "interruptions": 2,
    "response_quality": 0.85
}

-- sync_status 字段结构
{
    "anki": "synced",
    "obsidian": "synced",
    "notion": "pending",
    "last_sync": "2024-01-15T23:00:00Z"
}
```

#### knowledge_points

```sql
CREATE TABLE knowledge_points (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- 内容
    content TEXT NOT NULL,
    content_type VARCHAR(50) NOT NULL,  -- concept, fact, procedure, story

    -- 来源
    source_type VARCHAR(50) NOT NULL,   -- anki, obsidian, notion, readwise, manual
    source_id VARCHAR(255),             -- 原始 ID

    -- 向量
    embedding VECTOR(1536),

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
    last_reviewed_at TIMESTAMPTZ,

    INDEX idx_knowledge_points_user_id (user_id),
    INDEX idx_knowledge_points_source (source_type),
    INDEX idx_knowledge_points_tags (tags),
    INDEX idx_knowledge_points_last_reviewed (last_reviewed_at),

    -- 向量相似度搜索（pgvector）
    INDEX idx_knowledge_points_embedding ON knowledge_points
        USING ivfflat (embedding vector_cosine_ops)
        WITH (lists = 100)
);

-- 全文搜索
CREATE INDEX idx_knowledge_points_content_fts
    ON knowledge_points
    USING gin(to_tsvector('english', content));
```

#### review_intervals

```sql
CREATE TABLE review_intervals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    knowledge_point_id UUID NOT NULL REFERENCES knowledge_points(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- FSRS 参数
    interval_days FLOAT NOT NULL DEFAULT 1,
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

    UNIQUE(knowledge_point_id, user_id),
    INDEX idx_review_intervals_user_next (user_id, next_review_date),
    INDEX idx_review_intervals_knowledge (knowledge_point_id)
);
```

#### snapshots

```sql
CREATE TABLE snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,

    -- 快照数据
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    state JSONB NOT NULL,
    topic_stack JSONB NOT NULL,
    context JSONB NOT NULL,
    metrics JSONB NOT NULL,

    INDEX idx_snapshots_session_id (session_id),
    INDEX idx_snapshots_timestamp (timestamp DESC)
);
```

#### anki_cards

```sql
CREATE TABLE anki_cards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- Anki 数据
    anki_card_id BIGINT UNIQUE,          -- Anki 卡片 ID
    anki_note_id BIGINT,                 -- Anki 笔记 ID
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
    last_synced_at TIMESTAMPTZ,

    INDEX idx_anki_cards_user_id (user_id),
    INDEX idx_anki_cards_anki_id (anki_card_id),
    INDEX idx_anki_cards_sync_status (sync_status)
);
```

---

## 3. Redis 数据结构

### 3.1 缓存 Key 设计

| Pattern | 用途 | TTL |
|---------|------|-----|
| `session:{id}` | 会话状态（热数据） | 1h |
| `session:{id}:ws` | WebSocket 连接信息 | 持久 |
| `user:{id}:sessions` | 用户活跃会话列表 | 24h |
| `snapshot:{id}` | 会话快照（快速恢复） | 1h |
| `user:{id}:due_cards` | 今日待复习卡片 | 12h |
| `user:{id}:queue` | 用户任务队列 | 持久 |
| `rate_limit:{user_id}:{endpoint}` | 速率限制 | 按配置 |

### 3.2 数据结构示例

#### session:{id}

```redis
HSET session:123e4567-e89b-12d3-a456-426614174000
  state "deep_dive"
  user_id "999e4567-e89b-12d3-a456-426614174000"
  started_at "1705334400"
  last_activity "1705338000"
  current_topic "装饰器概念"
  fatigue_score "45"

EXPIRE session:123e4567-e89b-12d3-a456-426614174000 3600
```

#### user:{id}:queue

```redis
LPUSH user:999:queue "task:sync_anki:1705338900"
LPUSH user:999:queue "task:create_cards:1705339000"

# 消费任务
RPOP user:999:queue
```

---

## 4. 向量数据结构

### 4.1 Qdrant Collection

```json
{
  "collection_name": "knowledge_points",
  "vectors": {
    "size": 1536,
    "distance": "Cosine"
  },
  "payload_schema": {
    "user_id": "uuid",
    "content_type": "keyword",
    "source_type": "keyword",
    "tags": "array<string>",
    "created_at": "integer"
  }
}
```

### 4.2 Payload 结构

```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "user_id": "999e4567-e89b-12d3-a456-426614174000",
  "content": "装饰器本质是函数包装器...",
  "content_type": "concept",
  "source_type": "anki",
  "tags": ["python", "decorator", "函数式编程"],
  "created_at": 1705334400
}
```

### 4.3 过滤器示例

```rust
// 仅搜索用户的内容，且来自 Anki
let filter = qdrant::filters::Filter::must([
    qdrant::filters::Condition::field("user_id").match_uuid(user_id),
    qdrant::filters::Condition::field("source_type").match("anki"),
]);

// 搜索最近 7 天的内容
let time_filter = qdrant::filters::Filter::must([
    qdrant::filters::Condition::field("user_id").match_uuid(user_id),
    qdrant::filters::Condition::field("created_at")
        .range(Utc::now() - Duration::from_secs(86400 * 7)..Utc::now()),
]);
```

---

## 5. Rust 数据模型

### 5.1 核心模型

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub preferences: UserPreferences,
    pub integration_config: IntegrationConfig,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub voice: VoicePreferences,
    pub session: SessionPreferences,
    pub content: ContentPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub state: SessionState,
    pub topic_stack: TopicStack,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub metrics: SessionMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgePoint {
    pub id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub content_type: ContentType,
    pub source_type: SourceType,
    pub source_id: Option<String>,
    pub embedding: Option<Vec<f32>>,
    pub title: Option<String>,
    pub summary: Option<String>,
    pub tags: Vec<String>>,
    pub parent_id: Option<Uuid>,
    pub related_ids: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_reviewed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewInterval {
    pub id: Uuid,
    pub knowledge_point_id: Uuid,
    pub user_id: Uuid,
    pub interval_days: f32,
    pub ease_factor: f32,
    pub stability: Option<f32>,
    pub retrievability: Option<f32>,
    pub next_review_date: NaiveDate,
    pub last_review_date: Option<NaiveDate>,
    pub total_reviews: i32,
    pub correct_reviews: i32,
    pub lapses: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    Concept,
    Fact,
    Procedure,
    Story,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceType {
    Anki,
    Obsidian,
    Notion,
    Readwise,
    Manual,
}
```

### 5.2 数据库映射

```rust
use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub preferences: Json<UserPreferences>,
    pub integration_config: Json<IntegrationConfig>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        Self {
            id: row.id,
            username: row.username,
            email: row.email,
            preferences: row.preferences.0,
            integration_config: row.integration_config.0,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
```

---

## 6. 数据迁移策略

### 6.1 迁移工具

```rust
// 使用 sqlx-cli 管理迁移
// migrations/001_initial.down.sql

pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}
```

### 6.2 版本控制

```
migrations/
├── 001_initial.up.sql
├── 001_initial.down.sql
├── 002_add_vector_index.up.sql
├── 002_add_vector_index.down.sql
├── 003_add_review_intervals.up.sql
└── 003_add_review_intervals.down.sql
```

---

## 7. 数据一致性

### 7.1 事务处理

```rust
pub async fn create_session_with_initial_snapshot(
    pool: &PgPool,
    user_id: Uuid,
    initial_state: SessionState,
) -> Result<Session> {
    let mut tx = pool.begin().await?;

    // 1. 创建会话
    let session = sqlx::query_as!(
        Session,
        "INSERT INTO sessions (user_id, state)
         VALUES ($1, $2)
         RETURNING *",
        user_id,
        initial_state as SessionState
    )
    .fetch_one(&mut *tx)
    .await?;

    // 2. 创建初始快照
    sqlx::query!(
        "INSERT INTO snapshots (session_id, state, topic_stack, context, metrics)
         VALUES ($1, $2, $3, $4, $5)",
        session.id,
        serde_json::to_value(initial_state)?,
        serde_json::to_value(TopicStack::new(3))?,
        serde_json::to_value(SessionContext::default())?,
        serde_json::to_value(SessionMetrics::default())?
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(session)
}
```

### 7.2 最终一致性

对于外部集成（Anki、Obsidian），采用最终一致性：

```rust
pub async fn sync_session_results(
    session_id: Uuid,
    orchestrator: &IntegrationOrchestrator,
) -> Result<()> {
    // 获取会话摘要
    let summary = get_session_summary(session_id).await?;

    // 尝试同步所有集成，失败不回滚
    let results = orchestrator.sync_with_fallback(&summary).await;

    // 记录同步状态
    update_sync_status(session_id, results).await?;

    Ok(())
}
```
