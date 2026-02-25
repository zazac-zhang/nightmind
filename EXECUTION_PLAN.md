# NightMind 开发执行计划

> 从零到可运行的最小可行产品 (MVP)

---

## 📋 总览

本计划按照模块依赖顺序，逐步构建 NightMind 的核心功能。

**目标**: 实现一个基础的会话式 AI 助手，能够通过 WebSocket 进行对话。

---

## 🎯 阶段概览

| 阶段 | 时间 | 目标 |
|------|------|------|
| **Phase 0** | 1 天 | 基础设施搭建 |
| **Phase 1** | 2 天 | 配置与错误处理 |
| **Phase 2** | 3 天 | 数据访问层 |
| **Phase 3** | 3 天 | 核心 Agent 系统 |
| **Phase 4** | 3 天 | API 层与 WebSocket |
| **Phase 5** | 2 天 | 集成与测试 |

---

## Phase 0: 基础设施搭建

### 目标
建立项目基础结构和开发工具。

### 任务清单

- [ ] **Task 0.1**: 初始化项目结构
  - [ ] 创建 `src/` 目录结构
  - [ ] 创建 `lib.rs` 和 `main.rs`
  - [ ] 验证 `cargo build` 成功

```bash
# 创建目录结构
mkdir -p src/{api,core/{agent,session,content},services,integration,vector,repository,config}
touch src/lib.rs
```

**验收**: `cargo build` 无错误

---

## Phase 1: 配置与错误处理

### 目标
实现配置加载和统一的错误处理系统。

### Task 1.1: 配置系统

**文件**: `src/config/settings.rs`

```rust
// 需要实现的内容：
// - Settings 结构体定义
// - 从环境变量/文件加载配置
// - 数据库配置
// - AI 服务配置
// - 集成服务配置
```

**步骤**:
1. 定义 `Settings` 结构体
2. 实现 `Settings::load()` 方法
3. 实现环境变量解析
4. 添加配置验证

**验收**: 能够从 `.env` 文件加载配置

### Task 1.2: 错误处理

**文件**: `src/error.rs`

```rust
// 需要实现的内容：
// - NightMindError 枚举定义
// - 错误转换 trait
// - API Error 响应类型
```

**步骤**:
1. 定义核心错误类型
2. 为外部库实现错误转换
3. 实现 `IntoResponse` for API errors
4. 添加错误辅助函数

**验收**: 错误能正确传播和序列化

### Task 1.3: 日志初始化

**文件**: `src/config/logging.rs`

```rust
// 需要实现的内容：
// - init_tracing() 函数
// - 日志格式配置
// - 日志级别配置
```

**验收**: 应用启动时能看到日志输出

---

## Phase 2: 数据访问层

### 目标
实现数据库操作的基础设施。

### Task 2.1: 数据库连接池

**文件**: `src/repository/db.rs`

```rust
// 需要实现的内容：
// - create_pool() 函数
// - PgPool 管理器
// - 连接池配置
```

**步骤**:
1. 配置 SQLx 连接池
2. 实现连接池获取
3. 添加健康检查

**验收**: 能成功连接数据库

### Task 2.2: Redis 连接

**文件**: `src/repository/redis.rs`

```rust
// 需要实现的内容：
// - RedisClient 包装器
// - 连接管理
// - 基本操作封装
```

**验收**: Redis 读写正常

### Task 2.3: 用户 Repository

**文件**: `src/repository/user.rs`

```rust
// 需要实现的内容：
// - UserRepository trait
// - 创建用户
// - 查找用户（by id, email, username）
// - 更新用户
```

**步骤**:
1. 定义 `User` 模型
2. 实现 CRUD 操作
3. 添加单元测试

**验收**: 用户数据能正确存储和检索

### Task 2.4: 会话 Repository

**文件**: `src/repository/session.rs`

```rust
// 需要实现的内容：
// - SessionRepository trait
// - 创建会话
// - 更新会话状态
// - 查询活跃会话
```

**验收**: 会话数据能正确管理

---

## Phase 3: 核心 Agent 系统

### 目标
实现基础的 AI Agent 对话能力。

### Task 3.1: Agent Builder

**文件**: `src/core/agent/builder.rs`

```rust
// 需要实现的内容：
// - NightMindAgent 结构体
// - Agent 初始化
// - 流式响应处理
```

**步骤**:
1. 集成 Rig Agent Builder
2. 定义系统 Prompt
3. 实现流式消息处理

**验收**: Agent 能响应消息

### Task 3.2: Prompt 模板

**文件**: `src/core/agent/prompts.rs`

```rust
// 需要实现的内容：
// - SYSTEM_PROMPT 常量
// - 阶段性 Prompt 模板
// - Prompt 渲染函数
```

**验收**: Prompt 能正确渲染

### Task 3.3: 会话状态机

**文件**: `src/core/session/state.rs`

```rust
// 需要实现的内容：
// - SessionState 枚举
// - 状态转换逻辑
// - 状态验证
```

**验收**: 状态能正确转换

### Task 3.4: 会话管理器

**文件**: `src/core/session/manager.rs`

```rust
// 需要实现的内容：
// - SessionManager 结构体
// - 创建会话
// - 处理消息
// - 广播事件
```

**验收**: 会话能完整创建和管理

---

## Phase 4: API 层与 WebSocket

### 目标
实现 HTTP API 和 WebSocket 通信。

### Task 4.1: 路由定义

**文件**: `src/api/router.rs`

```rust
// 需要实现的内容：
// - create_router() 函数
// - API 路由定义
// - 状态管理
```

**验收**: 路由能正确注册

### Task 4.2: 中间件

**文件**: `src/api/middleware.rs`

```rust
// 需要实现的内容：
// - auth_middleware (JWT)
// - rate_limit_middleware
// - request_id_middleware
// - error_handler_middleware
```

**验收**: 中间件能正确拦截请求

### Task 4.3: REST Handlers

**文件**: `src/api/handlers/rest.rs`

```rust
// 需要实现的内容：
// - 健康检查
// - 用户注册/登录
// - 创建会话
// - 获取会话列表
```

**验收**: REST API 能正常响应

### Task 4.4: WebSocket Handler

**文件**: `src/api/handlers/websocket.rs`

```rust
// 需要实现的内容：
// - WebSocket 升级
// - 消息类型定义
// - 消息处理循环
// - 广播机制
```

**验收**: WebSocket 连接能双向通信

### Task 4.5: DTO 定义

**文件**: `src/api/dto/mod.rs`

```rust
// 需要实现的内容：
// - 请求 DTO
// - 响应 DTO
// - WebSocket 消息 DTO
```

**验收**: 数据能正确序列化/反序列化

---

## Phase 5: 集成与测试

### 目标
整合所有模块并进行端到端测试。

### Task 5.1: 主入口

**文件**: `src/main.rs`

```rust
// 需要实现的内容：
// - 加载配置
// - 初始化日志
// - 初始化数据库
// - 启动 HTTP 服务器
```

**验收**: 应用能正常启动和关闭

### Task 5.2: 集成测试

**文件**: `tests/integration.rs`

```rust
// 需要实现的内容：
// - WebSocket 连接测试
// - 消息发送接收测试
// - 会话生命周期测试
```

**验收**: 核心流程测试通过

### Task 5.3: 压力测试

```bash
# 使用工具进行压力测试
# - WebSocket 连接数
# - 消息吞吐量
# - 响应延迟
```

**验收**: 达到性能目标

---

## 📝 详细执行步骤

### Step 1: 创建基础文件结构 (15 分钟)

```bash
# 在 src/ 目录下创建所有模块的 mod.rs
cd src
for dir in api core/{agent,session,content} services integration repository config; do
    mkdir -p $dir
    echo "pub mod $dir;" >> mod.rs 2>/dev/null || true
    touch $dir/mod.rs
done

# 创建核心文件
touch lib.rs main.rs
touch config/settings.rs
touch error.rs
```

### Step 2: 实现错误处理 (30 分钟)

```rust
// src/error.rs
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum NightMindError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("AI service error: {0}")]
    AiService(String),

    #[error("Not found: {0}")]
    NotFound(String),
}

impl IntoResponse for NightMindError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            NightMindError::Database(e) => {
                tracing::error!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error")
            }
            NightMindError::NotFound(_) => (StatusCode::NOT_FOUND, "Not found"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error"),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}

pub type Result<T> = std::result::Result<T, NightMindError>;
```

### Step 3: 实现配置加载 (30 分钟)

```rust
// src/config/settings.rs
use config::{Config, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub redis: RedisSettings,
    pub ai: AiSettings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseSettings {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedisSettings {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AiSettings {
    pub openai_api_key: String,
    pub model: String,
}

impl Settings {
    pub fn load() -> anyhow::Result<Self> {
        let settings = Config::builder()
            .add_source(File::with_name("config/settings").required(false))
            .add_source(Environment::with_prefix("NIGHTMIND"))
            .build()?;

        Ok(settings.try_deserialize()?)
    }
}
```

### Step 4: 实现数据库连接 (20 分钟)

```rust
// src/repository/db.rs
use sqlx::PgPool;
use crate::config::Settings;

pub async fn create_pool(settings: &Settings) -> anyhow::Result<PgPool> {
    let pool = PgPool::connect(&settings.database.url).await?;
    Ok(pool)
}
```

### Step 5: 实现 Agent 对话 (45 分钟)

```rust
// src/core/agent/builder.rs
use rig::{
    agent::AgentBuilder,
    providers::openai,
};
use crate::config::Settings;

const SYSTEM_PROMPT: &str = r#"
你是 NightMind，一位专业的睡前认知巩固导师。

你的使命是帮助用户在睡前通过深度对话巩固今日所学。

核心原则：
1. 使用简洁、口语化的语言
2. 每段回答控制在 15 秒内可读完
3. 使用比喻和类比解释复杂概念
"#;

pub struct NightMindAgent {
    agent: Agent<openai::Client>,
}

impl NightMindAgent {
    pub async fn new(settings: &Settings) -> anyhow::Result<Self> {
        let client = openai::Client::from_env();

        let agent = client
            .agent(&settings.ai.model)
            .preamble(SYSTEM_PROMPT)
            .build();

        Ok(Self { agent })
    }

    pub async fn prompt(&self, message: &str) -> anyhow::Result<String> {
        Ok(self.agent.prompt(message).await?)
    }
}
```

### Step 6: 实现 WebSocket Handler (60 分钟)

```rust
// src/api/handlers/websocket.rs
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(agent): State<crate::core::agent::NightMindAgent>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, agent))
}

async fn handle_socket(
    mut socket: WebSocket,
    agent: crate::core::agent::NightMindAgent,
) {
    while let Some(Ok(msg)) = socket.next().await {
        match msg {
            Message::Text(text) => {
                if let Ok(response) = agent.prompt(&text).await {
                    if let Err(e) = socket.send(Message::Text(response)).await {
                        tracing::error!("Failed to send message: {:?}", e);
                        break;
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
}
```

### Step 7: 实现主入口 (30 分钟)

```rust
// src/main.rs
use nightmind::{
    config::Settings,
    core::agent::NightMindAgent,
    error::Result,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into())
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 加载配置
    let settings = Settings::load()?;
    tracing::info!("NightMind starting...");

    // 初始化 Agent
    let agent = NightMindAgent::new(&settings).await?;

    // 构建路由
    let app = nightmind::api::router::create_router(agent);

    // 启动服务器
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("Server listening on http://0.0.0.0:8080");

    axum::serve(listener, app).await?;

    Ok(())
}
```

---

## ✅ 验收标准

### 最小 MVP 目标

- [x] 应用能启动
- [x] WebSocket 能连接
- [x] 发送消息能得到 AI 回复
- [x] 错误能正确处理
- [x] 日志能正常输出

### 测试命令

```bash
# 1. 启动服务
make docker-up
make db-migrate
cargo run

# 2. 测试 WebSocket
wscat -c ws://localhost:8080/api/v1/ws/session

# 3. 发送消息
{"type":"text_input","data":{"text":"你好"}}

# 4. 应该收到回复
{"type":"text_response","data":{"text":"你好！我是 NightMind..."}}
```

---

## 📅 时间估算

| 阶段 | 预计时间 | 实际时间 |
|------|----------|----------|
| Phase 0 | 0.5 天 | ___ |
| Phase 1 | 1 天 | ___ |
| Phase 2 | 2 天 | ___ |
| Phase 3 | 2 天 | ___ |
| Phase 4 | 2 天 | ___ |
| Phase 5 | 1 天 | ___ |
| **总计** | **8.5 天** | ___ |

---

## 🔗 相关文档

- [架构设计](../doc/01-architecture.md)
- [核心模块](../doc/02-core-modules.md)
- [Agent 系统](../doc/03-agent-system.md)
- [API 设计](../doc/08-api-design.md)

---

**开始日期**: ___________
**完成日期**: ___________
**负责人**: ___________
