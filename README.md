# NightMind

> Close your eyes, open your mind.

**基于睡眠认知科学的 AI 伴学 Agent** — 利用睡前黄金窗口期进行深度知识巩固。

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
| **零交互** | 全语音交互，无需阅读文字 |
| **认知优先** | 用户专注思考，Agent 处理技术细节 |
| **无缝恢复** | 随时打断，自动恢复上下文 |
| **生态集成** | 与 Anki、Obsidian、Notion 无缝对接 |

---

## 技术栈

```
语言：Rust (Edition 2024)
框架：Axum (Web) + Rig (Agent)
运行时：Tokio
数据库：PostgreSQL + Redis + Qdrant
AI: OpenAI GPT-4 + Whisper + TTS
```

---

## 架构概览

```
┌─────────────────────────────────────────────────────┐
│                    API Layer                        │
│  (handlers, websocket, router, middleware)         │
├─────────────────────────────────────────────────────┤
│                   Core Layer                        │
│     (agent system, session management, content)     │
├─────────────────────────────────────────────────────┤
│                 Repository Layer                    │
│      (database models, user, session, knowledge)    │
├─────────────────────────────────────────────────────┤
│                 Services Layer                      │
│     (audio, STT, TTS, integration, vector)          │
├─────────────────────────────────────────────────────┤
│                 Infrastructure                      │
│        (config, error, logging)                     │
└─────────────────────────────────────────────────────┘
```

详见 [架构文档](doc/architecture.md)。

---

## 开发计划

当前状态：**Phase 1** 基础设施建设

| 阶段 | 目标 | 状态 |
|------|------|------|
| Phase 0 | 基础设施搭建 | ✅ |
| Phase 1 | 配置与错误处理 | 🟡 |
| Phase 2 | 数据访问层 | 🔲 |
| Phase 3 | Agent 系统 | 🔲 |
| Phase 4 | API 层 | 🔲 |
| Phase 5 | 集成测试 | 🔲 |

详见 [开发计划](docs/plan.md)。

---

## 常用命令

```bash
# 构建和运行
make build        # 构建 release
make dev          # 运行开发服务器
make dev-watch    # 热重载

# 测试
make test         # 运行所有测试
make test-coverage # 生成覆盖率报告

# 数据库
make db-migrate   # 运行迁移
make db-shell     # 连接数据库
make db-reset     # 重置数据库

# Docker
make docker-up    # 启动服务
make docker-down  # 停止服务
make docker-logs  # 查看日志
```

---

## 文档索引

| 文档 | 描述 |
|------|------|
| [架构设计](doc/architecture.md) | 整体架构、技术选型、模块划分 |
| [API 设计](doc/api.md) | WebSocket + REST API 接口定义 |
| [开发计划](docs/plan.md) | 开发路线图与任务清单 |

---

## 许可证

MIT License
