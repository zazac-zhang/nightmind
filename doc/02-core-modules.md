# 核心模块设计

## 1. 模块划分

```
src/
├── main.rs                    # 入口
├── lib.rs
│
├── api/                       # API 层
│   ├── mod.rs
│   ├── router.rs              # 路由配置
│   ├── middleware.rs          # 中间件
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── websocket.rs       # WebSocket 处理
│   │   └── rest.rs            # REST API
│   └── dto/                   # 数据传输对象
│
├── core/                      # 核心业务逻辑
│   ├── mod.rs
│   ├── agent/                 # Rig Agent 相关
│   │   ├── mod.rs
│   │   ├── builder.rs         # Agent 构建器
│   │   ├── tools.rs           # 工具定义
│   │   └── prompts.rs         # Prompt 模板
│   ├── session/               # 会话管理
│   │   ├── mod.rs
│   │   ├── manager.rs         # SessionManager
│   │   ├── state.rs           # 状态机
│   │   ├── snapshot.rs        # 快照机制
│   │   └── topic_stack.rs     # 话题栈
│   └── content/               # 内容处理
│       ├── mod.rs
│       ├── transformer.rs     # 内容转换
│       └── rhythm.rs          # 节奏控制
│
├── services/                  # 服务层
│   ├── mod.rs
│   ├── audio.rs               # 音频流处理
│   ├── stt.rs                 # 语音识别
│   ├── tts.rs                 # 语音合成
│   ├── integration/           # 外部集成
│   │   ├── mod.rs
│   │   ├── anki.rs
│   │   ├── obsidian.rs
│   │   └── notion.rs
│   └── vector/                # 向量存储
│       ├── mod.rs
│       └── store.rs
│
├── repository/                # 数据访问层
│   ├── mod.rs
│   ├── postgres.rs
│   └── redis.rs
│
└── config/                    # 配置
    ├── mod.rs
    └── settings.rs
```

---

## 2. 模块职责

### 2.1 API 层 (`api/`)

**职责**: 协议转换、请求路由、认证授权

| 模块 | 职责 |
|------|------|
| `router.rs` | 定义 HTTP/WebSocket 路由，组合 handler |
| `middleware.rs` | 认证、CORS、限流、日志 |
| `handlers/websocket.rs` | WebSocket 连接管理、消息分发 |
| `handlers/rest.rs` | REST API 处理（用户、会话 CRUD） |
| `dto/` | 请求/响应数据结构定义 |

**关键接口**:

```rust
// WebSocket Handler
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(session_manager): State<SessionManager>,
) -> impl IntoResponse;

// REST Handlers
pub async fn create_session(
    State(manager): State<SessionManager>,
    Json(req): Json<CreateSessionRequest>,
) -> Result<Json<SessionResponse>, ApiError>;

pub async fn get_session(
    Path(id): Path<Uuid>,
    State(manager): State<SessionManager>,
) -> Result<Json<SessionDetail>, ApiError>;
```

### 2.2 核心层 (`core/`)

**职责**: 业务逻辑、状态管理、Agent 编排

#### `core/agent/` - Agent 系统

| 模块 | 职责 |
|------|------|
| `builder.rs` | 构建 Rig Agent，配置 preamble/tools/context |
| `tools.rs` | 定义 Agent 可用工具 |
| `prompts.rs` | Prompt 模板管理 |

**关键接口**:

```rust
pub struct NightMindAgent {
    agent: Agent<CompletionModel, VectorStoreIndex>,
}

impl NightMindAgent {
    pub async fn new(client: &Client, vector_store: &VectorStoreService)
        -> Result<Self>;

    pub async fn prompt(&self, message: &str) -> Result<String>;

    pub async fn prompt_stream(&self, message: &str)
        -> Result<impl Stream<Item = String>>;
}
```

#### `core/session/` - 会话管理

| 模块 | 职责 |
|------|------|
| `manager.rs` | 会话生命周期管理、消息路由 |
| `state.rs` | 会话状态机定义 |
| `snapshot.rs` | 快照保存与恢复 |
| `topic_stack.rs` | 话题栈管理 |

**关键接口**:

```rust
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<Uuid, Session>>>,
    agent: Arc<NightMindAgent>,
    snapshot_store: Arc<SnapshotStore>,
}

impl SessionManager {
    pub async fn create_session(&self, id: Uuid, tx: broadcast::Sender<SessionEvent>)
        -> Result<()>;

    pub async fn handle_transcript(&self, session_id: Uuid, transcript: String);

    pub async fn restore_session(&self, session_id: Uuid) -> Result<Snapshot>;

    pub async fn close_session(&self, session_id: Uuid);
}
```

**状态机**:

```rust
pub enum SessionState {
    Warmup,      // 0-5min: 暖场校准
    DeepDive,    // 5-25min: 核心深度
    Review,      // 25-45min: 关联巩固
    Seed,        // 45-55min: 种子沉淀
    Closing,     // 55-60min: 温柔收尾
    Closed,
}
```

#### `core/content/` - 内容处理

| 模块 | 职责 |
|------|------|
| `transformer.rs` | 内容格式转换（代码→比喻，公式→解释） |
| `rhythm.rs` | 会话节奏控制、疲劳检测 |

**关键接口**:

```rust
pub struct ContentTransformer;

impl ContentTransformer {
    pub async fn to_voice_friendly(&self, content: &str) -> Result<String>;

    pub fn validate(&self, content: &str) -> ValidationResult;
}

pub struct RhythmController {
    phase_durations: HashMap<SessionState, Duration>,
}

impl RhythmController {
    pub fn current_phase(&self, elapsed: Duration) -> SessionState;

    pub fn should_transition(&self, session: &Session) -> bool;

    pub fn adjust_content_density(&self, fatigue_score: u8) -> DensityLevel;
}
```

### 2.3 服务层 (`services/`)

**职责**: 外部依赖抽象、集成服务

| 模块 | 职责 |
|------|------|
| `audio.rs` | 音频流编解码、格式转换 |
| `stt.rs` | 语音识别服务封装 |
| `tts.rs` | 语音合成服务封装 |
| `integration/anki.rs` | AnkiConnect 集成 |
| `integration/obsidian.rs` | Obsidian 文件同步 |
| `integration/notion.rs` | Notion API 集成 |
| `vector/store.rs` | 向量存储抽象 |

**关键接口**:

```rust
// STT Service
#[async_trait]
pub trait SttService: Send + Sync {
    async fn transcribe(&self, audio_data: Vec<u8>) -> Result<String>;
    async fn transcribe_stream(&self, stream: AudioStream)
        -> Result<impl Stream<Item = String>>;
}

// TTS Service
#[async_trait]
pub trait TtsService: Send + Sync {
    async fn synthesize(&self, text: &str) -> Result<Vec<u8>>;

    async fn synthesize_stream(&self, text: &str)
        -> Result<impl Stream<Item = Vec<u8>>>;
}

// Vector Store
#[async_trait]
pub trait VectorStore: Send + Sync {
    async fn add_documents(&self, docs: Vec<Document>) -> Result<()>;

    async fn search(&self, query: &str, limit: usize)
        -> Result<Vec<SearchResult>>;

    fn index(&self, embedding_model: EmbeddingModel) -> VectorStoreIndex;
}
```

### 2.4 数据访问层 (`repository/`)

**职责**: 数据库操作、缓存管理

| 模块 | 职责 |
|------|------|
| `postgres.rs` | PostgreSQL CRUD 操作 |
| `redis.rs` | Redis 缓存操作 |

**关键接口**:

```rust
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub async fn create(&self, user: &NewUser) -> Result<User>;

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>>;

    pub async fn update_preferences(&self, id: Uuid, prefs: Value)
        -> Result<()>;
}

pub struct SessionRepository {
    pool: PgPool,
    redis: redis::Client,
}

impl SessionRepository {
    pub async fn save_snapshot(&self, snapshot: &Snapshot) -> Result<()>;

    pub async fn load_snapshot(&self, session_id: Uuid) -> Result<Option<Snapshot>>;

    pub async fn cache_state(&self, session_id: Uuid, state: &SessionState)
        -> Result<()>;
}
```

### 2.5 配置 (`config/`)

**职责**: 应用配置管理

```rust
pub struct Settings {
    pub server: ServerSettings,
    pub database: DatabaseSettings,
    pub ai: AiSettings,
    pub integration: IntegrationSettings,
}

pub struct AiSettings {
    pub openai_api_key: String,
    pub openai_model: String,
    pub embedding_model: String,
}

impl Settings {
    pub fn load() -> Result<Self> {
        // 从环境变量 / 配置文件加载
    }
}
```

---

## 3. 模块间通信

### 3.1 同步通信

```rust
// 通过依赖注入直接调用
impl SessionManager {
    pub async fn handle_transcript(&self, session_id: Uuid, transcript: String) {
        // 调用 Agent
        let response = self.agent.prompt(&transcript).await?;

        // 更新状态
        self.update_state(session_id, response).await?;
    }
}
```

### 3.2 异步通信

```rust
// 使用 broadcast channel
pub enum SessionEvent {
    AudioTTS(Vec<u8>),
    HapticFeedback(HapticPattern),
    TextResponse(String),
}

// WebSocket Handler 发送
let (tx, _) = broadcast::channel(32);
session_manager.create_session(id, tx).await?;

// SessionManager 广播
self.event_tx.send(SessionEvent::TextResponse(response))?;
```

### 3.3 流式通信

```rust
// Agent 流式响应
pub async fn prompt_stream(&self, message: &str)
    -> Result<impl Stream<Item = String>>
{
    let stream = self.agent.prompt_stream(message).await?;
    Ok(stream)
}

// 处理流
let mut stream = agent.prompt_stream(message).await?;
while let Some(chunk) = stream.next().await {
    self.event_tx.send(SessionEvent::TextResponse(chunk?))?;
}
```

---

## 4. 错误处理

### 4.1 错误类型定义

```rust
#[derive(Debug, thiserror::Error)]
pub enum NightMindError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("AI service error: {0}")]
    AiService(String),

    #[error("Session not found: {0}")]
    SessionNotFound(Uuid),

    #[error("Invalid state transition: {0:?} -> {1:?}")]
    InvalidTransition(SessionState, SessionState),
}
```

### 4.2 错误传播

```rust
// 使用 anyhow
pub async fn handle_transcript(&self, transcript: String)
    -> anyhow::Result<()>
{
    let response = self.agent.prompt(&transcript)
        .context("Failed to get agent response")?;
    Ok(())
}
```

---

## 5. 测试策略

### 5.1 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_topic_stack_push() {
        let mut stack = TopicStack::new(3);
        stack.push(topic).unwrap();
        assert_eq!(stack.depth(), 1);
    }
}
```

### 5.2 集成测试

```rust
#[tokio::test]
async fn test_session_flow() {
    let manager = SessionManager::new(test_config()).await;
    manager.create_session(id, tx).await.unwrap();
    manager.handle_transcript(id, "你好".to_string()).await;
    // 验证状态变化
}
```

### 5.3 Mock 外部依赖

```rust
#[mockall::automock]
pub trait SttService {
    async fn transcribe(&self, data: Vec<u8>) -> Result<String>;
}

#[tokio::test]
async fn test_with_mock_stt() {
    let mut mock_stt = MockSttService::new();
    mock_stt.expect_transcribe()
        .returning(|_| Ok("测试文本".to_string()));

    // 使用 mock 进行测试
}
```
