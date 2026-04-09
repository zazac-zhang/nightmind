# NightMind 架构文档

## 架构概览

NightMind 采用分层架构：API 层 → 核心层 → 服务层 → 数据层。

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              NightMind Architecture                          │
├─────────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐      ┌──────────────────────────────────────────────────┐  │
│  │   Clients   │◄────►│              API Layer (Axum)                     │  │
│  │ (Web/Mobile)│      │  WebSocket / HTTP Router / Middleware            │  │
│  └─────────────┘      └──────────────────────────────────────────────────┘  │
│                                        │                                      │
│                                        ▼                                      │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │                        Core Layer (Rig + Agents)                       │  │
│  │  SessionManager / NightMindAgent / TopicStack                          │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                        │                                      │
│                                        ▼                                      │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │                         Service Layer                                  │  │
│  │  TTS / STT / AnkiService / ObsidianService / VectorStore               │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                        │                                      │
│                                        ▼                                      │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │                      Data & Storage Layer                              │  │
│  │  PostgreSQL (pgvector) / Redis                                         │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 技术选型

| 组件 | 技术 | 理由 |
|------|------|------|
| **语言** | Rust (Edition 2024) | 内存安全、高性能 |
| **Web 框架** | Axum 0.8 | 异步、类型安全、WebSocket 原生支持 |
| **Agent 框架** | Rig 0.7 | Rust 原生、支持 OpenAI/Claude |
| **运行时** | Tokio | 成熟的异步运行时 |
| **数据库** | PostgreSQL 16+ | 主数据存储，pgvector 支持 |
| **缓存** | Redis 7+ | 会话状态缓存 |
| **AI** | OpenAI GPT-4 | LLM/Embedding/Whisper/TTS |

**MVP 简化**:
- 仅使用 PostgreSQL + pgvector（Qdrant 延后）
- 会话状态简化为 3 态（Active/Closing/Closed）

---

## 核心模块

### 模块结构

```
src/
├── main.rs / lib.rs
├── api/
│   ├── router.rs              # 路由配置
│   ├── middleware.rs          # 认证、CORS、限流
│   └── handlers/
│       ├── websocket.rs       # WebSocket 处理
│       └── rest.rs            # REST API
├── core/
│   ├── agent/
│   │   ├── builder.rs         # NightMindAgent 构建
│   │   └── prompts.rs         # System prompt
│   ├── session/
│   │   ├── manager.rs         # SessionManager
│   │   ├── state.rs           # 简化状态机 (3 态)
│   │   └── topic_stack.rs     # 话题栈
│   └── content/
│       └── transformer.rs     # LLM 驱动内容转换
├── services/
│   ├── tts.rs                 # 语音合成
│   ├── stt.rs                 # 语音识别
│   ├── integration/
│   │   ├── anki.rs            # AnkiConnect
│   │   └── obsidian.rs        # 文件同步
│   └── vector.rs              # pgvector 封装
├── repository/
│   ├── models/                # 数据模型
│   ├── postgres.rs            # PostgreSQL 操作
│   └── redis.rs               # Redis 操作
└── config/
    └── settings.rs            # 配置加载
```

### 模块职责

| 模块 | 职责 |
|------|------|
| **API 层** | 协议转换、路由、认证授权 |
| **核心层** | 业务逻辑、状态管理、Agent 编排 |
| **服务层** | 外部依赖抽象（STT/TTS、集成） |
| **数据层** | 持久化、缓存、向量检索 |

---

## 会话状态机（简化）

**MVP**: 3 态替代原有 5 态

```rust
pub enum SessionState {
    Active,     // 正常对话中
    Closing,    // 准备结束（用户主动/LLM 感知疲劳）
    Closed,     // 已结束
}
```

状态转换：
- `Active → Closing`: 用户主动结束 或 LLM 检测到疲劳
- `Closing → Closed`: 完成收尾
- 任何状态 → `Closed`: 异常断开

**原有 5 态** (Warmup/DeepDive/Review/Seed/Closing) 延后到需要时再实现。

---

## 数据流

```
用户输入 (WebSocket/文本)
    ↓
SessionManager (处理消息、保存上下文)
    ↓
NightMindAgent (LLM 调用)
    ↓
    ├─→ (可选) VectorStore (知识检索)
    ├─→ (可选) ContentTransformer (语义转换)
    └─→ (可选) IntegrationService (Anki/Obsidian 同步)
    ↓
响应 → WebSocket 输出 / TTS 音频
```

---

## 关键设计决策

### ADR-001: LLM 驱动优先

**问题**: 内容转换系统规则过多（19 种模式检测），维护成本高

**决策**: 
- 主要转换逻辑交给 LLM（语义理解）
- 保留规则作为 fallback（可选）

### ADR-002: PostgreSQL 优先

**问题**: 是否需要独立的 Qdrant 向量数据库

**决策**: 
- MVP 仅使用 PostgreSQL + pgvector
- Qdrant 延后到需要水平扩展时

### ADR-003: 简化状态机

**问题**: 5 阶段状态机过于僵化

**决策**: 
- MVP 使用 3 态
- 状态转换由 LLM 感知用户状态

---

## 部署架构

### 单机部署 (MVP)

```
┌─────────────────────────────────────┐
│         Single Server               │
│  ┌───────────────────────────────┐  │
│  │  NightMind (Axum + Rig)       │  │
│  └───────────────────────────────┘  │
│  ┌─────────┐  ┌───┐                 │
│  │  PG     │  │Rd │                 │
│  └─────────┘  └───┘                 │
└─────────────────────────────────────┘
```

### 容器化

使用 `docker-compose.yml` 启动：
- PostgreSQL (5432) + pgvector
- Redis (6379)

---

## 性能目标

| 指标 | 目标 |
|------|------|
| **首字延迟** | < 500ms |
| **WebSocket 延迟** | < 100ms |
| **并发会话** | 100+ (单机 MVP) |

---

## 设计原则

1. **LLM 优先**: 用语义理解替代硬性规则
2. **分层解耦**: API 层仅负责协议转换
3. **流式优先**: LLM 响应流式返回
4. **可观测性**: 结构化日志 (`tracing`)
