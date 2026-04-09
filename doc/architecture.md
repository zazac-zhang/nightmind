# NightMind 架构文档

## 架构概览

NightMind 采用分层架构，从上到下分为：API 层、核心层、服务层、数据层。

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
│  │  SessionManager / TopicStack / StateSnapshotter / NightMindAgent       │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                        │                                      │
│                                        ▼                                      │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │                         Service Layer                                  │  │
│  │  AudioService / STT / TTS / Integration Services / VectorStore         │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│                                        │                                      │
│                                        ▼                                      │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │                      Data & Storage Layer                              │  │
│  │  PostgreSQL (pgvector) / Redis / Qdrant / File Storage                 │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 技术选型

| 组件 | 技术 | 理由 |
|------|------|------|
| **语言** | Rust (Edition 2024) | 内存安全、高性能 |
| **Web 框架** | Axum 0.8 | 异步、类型安全、WebSocket 原生支持 |
| **Agent 框架** | Rig 0.7 | Rust 原生、支持 20+ LLM |
| **运行时** | Tokio | 成熟的异步运行时 |
| **数据库** | PostgreSQL 16+ | 主数据存储，pgvector 支持 |
| **缓存** | Redis 7+ | 会话状态缓存 |
| **向量库** | Qdrant | 语义检索、RAG |
| **AI** | OpenAI GPT-4 | LLM/Embedding/Whisper/TTS |

---

## 核心模块

### 模块结构

```
src/
├── main.rs / lib.rs
├── api/                       # API 层
│   ├── router.rs              # 路由配置
│   ├── middleware.rs          # 认证、CORS、限流
│   └── handlers/
│       ├── websocket.rs       # WebSocket 处理
│       └── rest.rs            # REST API
├── core/                      # 核心业务逻辑
│   ├── agent/                 # Rig Agent
│   │   ├── builder.rs         # Agent 构建
│   │   └── prompts.rs         # Prompt 模板
│   ├── session/               # 会话管理
│   │   ├── manager.rs         # SessionManager
│   │   ├── state.rs           # 状态机
│   │   └── topic_stack.rs     # 话题栈
│   └── content/               # 内容处理
│       └── transformer.rs     # 内容转换
├── services/                  # 服务层
│   ├── audio.rs               # 音频流处理
│   ├── stt.rs                 # 语音识别
│   ├── tts.rs                 # 语音合成
│   ├── integration/           # 外部集成
│   └── vector/                # 向量存储
├── repository/                # 数据访问层
│   ├── models/                # 数据模型
│   ├── postgres.rs            # PostgreSQL 操作
│   └── redis.rs               # Redis 操作
└── config/                    # 配置
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

## 会话状态机

```rust
pub enum SessionState {
    Warmup,    // 00-05m: 暖场校准
    DeepDive,  // 05-25m: 核心深度
    Review,    // 25-45m: 关联巩固
    Seed,      // 45-55m: 种子沉淀
    Closing,   // 55-60m: 温柔收尾
    Closed,
}
```

状态转换规则：
- 正常流程：`Warmup → DeepDive → Review → Seed → Closing → Closed`
- 提前结束：任何状态 → `Closing`（用户主动/疲劳检测/入睡检测）

---

## 数据流

```
用户输入 (WebSocket)
    ↓
SessionManager (处理消息、保存快照)
    ↓
NightMindAgent (LLM 调用、工具编排)
    ↓
    ├─→ VectorStore (知识检索)
    ├─→ ContentTransformer (内容转换)
    └─→ IntegrationService (外部同步)
    ↓
TTS Service → WebSocket 输出
```

---

## 部署架构

### 单机部署 (MVP)

```
┌─────────────────────────────────────┐
│         Single Server               │
│  ┌───────────────────────────────┐  │
│  │  NightMind (Axum + Rig)       │  │
│  └───────────────────────────────┘  │
│  ┌─────────┐  ┌───┐  ┌─────────┐   │
│  │  PG     │  │Rd │  │ Qdrant  │   │
│  └─────────┘  └───┘  └─────────┘   │
└─────────────────────────────────────┘
```

### 容器化

使用 `docker-compose.yml` 启动：
- PostgreSQL (5432)
- Redis (6379)
- Qdrant (6333)

详见 `docs/deployment.md`（可选）。

---

## 性能目标

| 指标 | 目标 |
|------|------|
| **首字延迟** | < 500ms |
| **WebSocket 延迟** | < 100ms |
| **并发会话** | 1000+ (单机) |
| **向量检索** | < 50ms |

---

## 关键设计原则

1. **分层解耦**: API 层仅负责协议转换，核心层处理业务逻辑
2. **依赖倒置**: 核心层定义 trait，服务层实现
3. **流式优先**: LLM 响应、音频数据均流式处理
4. **可观测性**: 结构化日志 (`tracing`)、指标导出 (`prometheus`)
