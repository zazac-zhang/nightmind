# NightMind

> Close your eyes, open your mind.

**LLM 驱动的睡前语音学习伴侣** — 帮助广泛自学者通过自然语音对话巩固知识。

---

## 快速开始

```bash
# 克隆项目
git clone https://github.com/yourusername/nightmind.git
cd nightmind

# 配置环境变量
cp .env.example .env

# 启动依赖服务
make docker-up

# 运行开发服务器
make dev
```

---

## 核心特性

| 特性 | 描述 |
|------|------|
| **LLM 驱动** | 用语义理解替代硬性规则，灵活响应 |
| **语音优先** | 全语音交互，无需视觉界面 |
| **工具集成** | 与 Anki、Obsidian 双向同步 |
| **睡眠友好** | 节奏控制、疲劳感知、自然收尾 |

---

## 技术栈

```
语言：Rust (Edition 2024)
框架：Axum (Web) + Rig (Agent)
运行时：Tokio
数据库：PostgreSQL (pgvector) + Redis
AI: OpenAI GPT-4
```

---

## 架构概览

```
┌─────────────────────────────────────────────────────┐
│                    API Layer                        │
│  (handlers, websocket, router, middleware)         │
├─────────────────────────────────────────────────────┤
│                   Core Layer                        │
│     (LLM Agent, session management, content)        │
├─────────────────────────────────────────────────────┤
│                 Repository Layer                    │
│      (PostgreSQL models, Redis cache)               │
├─────────────────────────────────────────────────────┤
│                 Services Layer                      │
│     (TTS, STT, Anki, Obsidian, vector)              │
├─────────────────────────────────────────────────────┤
│                 Infrastructure                      │
│        (config, error, logging)                     │
└─────────────────────────────────────────────────────┘
```

详见 [架构文档](doc/architecture.md)。

---

## 开发计划

当前状态：**Phase 1** 基础对话

| 阶段 | 目标 | 状态 |
|------|------|------|
| Phase 1 | WebSocket + LLM 对话 | 🟡 |
| Phase 2 | 会话管理（简化 3 态） | 🔲 |
| Phase 3 | Anki/Obsidian 同步 | 🔲 |
| Phase 4 | TTS/STT 语音交互 | 🔲 |

详见 [开发计划](docs/plan.md)。

---

## 常用命令

```bash
# 构建和运行
make dev          # 运行开发服务器
make dev-watch    # 热重载
make build        # 构建 release

# 测试
make test         # 运行所有测试

# 数据库
make docker-up    # 启动服务
make db-migrate   # 运行迁移
make db-shell     # 连接数据库
```

---

## 文档索引

| 文档 | 描述 |
|------|------|
| [架构设计](doc/architecture.md) | 整体架构、技术选型 |
| [API 设计](doc/api.md) | WebSocket + REST API |
| [开发计划](docs/plan.md) | MVP 范围、任务清单 |

---

## 许可证

MIT License
