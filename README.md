# NightMind

> Close your eyes, open your mind.

**基于睡眠认知科学的 AI 伴学 Agent** — 利用睡眠依赖型记忆巩固原理，在睡前黄金窗口期进行深度知识巩固。

---

## 文档索引

### 快速导航

| 文档 | 描述 |
|------|------|
| [产品白皮书](doc/00-overview.md) | 产品愿景、设计哲学与核心价值 |
| [项目规划](plan.md) | 14 周开发路线图与里程碑 |
| [执行计划](EXECUTION_PLAN.md) | 📋 详细开发步骤 (从这里开始!) |
| [任务清单](TASKS.md) | ✅ Phase 0-5 开发任务清单 |
| [TODO 清单](todo.md) | 当前任务与进度追踪 |

### 架构文档

| 文档 | 描述 | 状态 |
|------|------|------|
| [01-architecture.md](doc/01-architecture.md) | 整体架构设计、技术栈选型 | 📝 |
| [02-core-modules.md](doc/02-core-modules.md) | 核心模块划分、职责边界 | 📝 |
| [03-agent-system.md](doc/03-agent-system.md) | Rig Agent 系统设计 | 📝 |
| [04-session-management.md](doc/04-session-management.md) | 会话管理、状态机、快照恢复 | 📝 |
| [05-content-processing.md](doc/05-content-processing.md) | 内容转换、节奏控制、疲劳检测 | 📝 |
| [06-integrations.md](doc/06-integrations.md) | Anki/Obsidian/Notion 集成方案 | 📝 |
| [07-data-model.md](doc/07-data-model.md) | 数据模型、存储设计 | 📝 |
| [08-api-design.md](doc/08-api-design.md) | API 接口设计（WebSocket + REST） | 📝 |
| [09-audio-handling.md](doc/09-audio-handling.md) | 音频流处理（STT/TTS） | 📝 |
| [10-deployment.md](doc/10-deployment.md) | 部署方案、运维监控 | 📝 |

---

## 核心特性

- **零交互原则** — 全语音交互，无需阅读文字
- **认知优先** — 用户专注思考，Agent 处理所有技术细节
- **无缝恢复** — 支持随时打断，自动恢复上下文
- **生态集成** — 与 Anki、Obsidian、Notion 无缝对接

---

## 技术栈

```
语言:     Rust (Edition 2024)
框架:     Axum (Web) + Rig (Agent)
运行时:   Tokio (Async)
数据库:   PostgreSQL + Redis + Qdrant
AI:       OpenAI GPT-4 + Whisper + TTS
```

---

## 快速开始

```bash
# 克隆项目
git clone https://github.com/yourusername/nightmind.git
cd nightmind

# 配置环境变量
cp .env.example .env
# 编辑 .env 填入 API Keys

# 运行
cargo run

# 运行测试
cargo test

# 查看文档
cargo doc --open
```

---

## 开发状态

> 🚧 项目处于 **Phase 1** 基础设施建设阶段

当前进度详见 [plan.md](plan.md) 和 [todo.md](todo.md)。

---

## 贡献指南

欢迎贡献！请先阅读 [架构文档](doc/01-architecture.md) 了解整体设计。

---

## 许可证

MIT License

---

*"The best ideas come as jokes. Make your thinking as funny as possible."* — NightMind
