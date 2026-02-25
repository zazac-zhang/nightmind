# NightMind TODO

> 跟踪当前开发任务和进度

---

## 当前冲刺: Week 1 - 项目初始化

**时间**: 2024-01-15 ~ 2024-01-21
**目标**: 搭建基础开发环境

---

### 进行中 🚧

#### [ ] 项目结构搭建

- [ ] `src/` 目录结构
  - [x] `api/` - API 层
  - [x] `core/` - 核心业务逻辑
  - [x] `services/` - 服务层
  - [x] `repository/` - 数据访问层
  - [x] `config/` - 配置
- [x] `doc/` 文档目录 ✅
  - [x] 00-overview.md (产品白皮书)
  - [x] 01-architecture.md (整体架构)
  - [x] 02-core-modules.md (核心模块)
  - [x] 03-agent-system.md (Agent 系统)
  - [x] 04-session-management.md (会话管理)
  - [x] 05-content-processing.md (内容处理)
  - [x] 06-integrations.md (外部集成)
  - [x] 07-data-model.md (数据模型)
  - [x] 08-api-design.md (API 设计)
  - [x] 09-audio-handling.md (音频处理)
  - [x] 10-deployment.md (部署方案)
- [x] `migrations/` 数据库迁移文件 ✅ (3 个迁移)
- [ ] `tests/` 测试文件

#### [ ] 依赖配置

- [ ] Cargo.toml 完善
  - [ ] Axum + Tokio
  - [ ] SQLx (PostgreSQL)
  - [ ] Redis
  - [ ] Rig (Agent)
  - [ ] Serde
  - [ ] Tracing
  - [ ] anyhow/thiserror
- [ ] 开发依赖
  - [ ] tokio-test
  - [ ] mockall

#### [x] 数据库 Schema ✅

- [x] users 表
- [x] sessions 表
- [x] knowledge_points 表
- [x] review_intervals 表
- [x] snapshots 表
- [x] anki_cards 表
- [x] 向量索引 (pgvector)
- [x] 性能优化和实用函数

#### [x] Docker 开发环境 ✅

- [x] Dockerfile (生产环境)
- [x] Dockerfile.dev (开发环境)
- [x] docker-compose.yml (开发环境)
- [x] docker-compose.prod.yml (生产环境)
- [x] .env.example (环境变量模板)
- [x] Makefile (便捷命令)
- [x] docker/postgres/init/ (初始化脚本)
  - [x] PostgreSQL
  - [x] Redis
  - [x] Qdrant

---

### 待办 📋

#### [x] CI/CD 配置 ✅

- [x] GitHub Actions workflow
  - [x] ci.yml (代码质量、测试、构建)
  - [x] release.yml (多平台发布)
  - [x] dependencies.yml (自动更新依赖)
- [x] Dependabot 配置
- [x] Issue 模板 (Bug / Feature)
- [x] Pull Request 模板
- [x] CONTRIBUTING.md 贡献指南

#### [ ] 基础代码

- [ ] `main.rs` 入口
- [ ] `config/settings.rs` 配置加载
- [ ] 错误类型定义
- [ ] 基础中间件

#### [ ] 文档完善

- [ ] README.md 完善
- [ ] CONTRIBUTING.md
- [ ] LICENSE

---

## 未来任务

### Week 2: API 层搭建

#### API 路由

- [ ] `/api/v1/auth/*` - 认证相关
- [ ] `/api/v1/users/*` - 用户管理
- [ ] `/api/v1/sessions/*` - 会话管理
- [ ] `/api/v1/knowledge/*` - 知识管理
- [ ] `/api/v1/integrations/*` - 外部集成
- [ ] `/api/v1/ws/session` - WebSocket

#### WebSocket Handler

- [ ] 连接管理
- [ ] 消息协议
- [ ] 心跳机制
- [ ] 错误处理

#### 中间件

- [ ] JWT 认证
- [ ] 速率限制
- [ ] CORS
- [ ] 请求日志

---

### Week 3: 数据访问层

#### PostgreSQL

- [ ] 连接池配置
- [ ] 用户 Repository
- [ ] 会话 Repository
- [ ] 知识点 Repository
- [ ] 快照 Repository

#### Redis

- [ ] 连接配置
- [ ] 会话缓存
- [ ] 快照缓存
- [ ] 速率限制

---

### Week 4: 会话管理

#### 状态机

- [ ] SessionState 定义
- [ ] 状态转换逻辑
- [ ] 转换触发器

#### SessionManager

- [ ] 创建会话
- [ ] 处理消息
- [ ] 状态转换
- [ ] 关闭会话

#### TopicStack

- [ ] push/pop
- [ ] 恢复提示生成
- [ ] 深度限制

#### 快照系统

- [ ] 保存快照
- [ ] 加载快照
- [ ] 自动清理

---

### Week 5-7: Agent 系统

#### Rig 集成

- [ ] Agent Builder 配置
- [ ] Preamble 定义
- [ ] 流式响应
- [ ] Prompt 模板

#### 工具系统

- [ ] ContentTransformerTool
- [ ] FatigueDetectorTool
- [ ] KnowledgeRetrieverTool
- [ ] AnkiIntegrationTool

#### 向量存储

- [ ] Qdrant 集成
- [ ] 文档索引
- [ ] 语义检索
- [ ] RAG 上下文

---

### Week 8-9: 内容处理

#### ContentTransformer

- [ ] 代码转比喻
- [ ] 公式转解释
- [ ] 列表转故事

#### RhythmController

- [ ] 阶段配置
- [ ] 时间管理
- [ ] 负荷自适应

#### 疲劳检测

- [ ] 多维度检测
- [ ] 阈值配置
- [ ] 建议生成

---

### Week 10-11: 外部集成

#### Anki

- [ ] AnkiConnect 客户端
- [ ] 卡片同步
- [ ] 复习流程

#### Obsidian

- [ ] 文件监听
- [ ] Markdown 解析
- [ ] 每日日志

#### Notion

- [ ] API 客户端
- [ ] 数据库同步

---

### Week 12-13: 音频处理

#### STT

- [ ] Whisper 集成
- [ ] 流式处理
- [ ] VAD 检测

#### TTS

- [ ] ElevenLabs 集成
- [ ] 流式合成
- [ ] 音频缓冲

---

## 技术债务

### 需要重构

- [ ] 错误处理统一
- [ ] 日志规范
- [ ] 配置管理优化

### 性能优化

- [ ] 数据库查询优化
- [ ] 缓存策略
- [ ] 连接池调优

### 测试覆盖

- [ ] 单元测试补充
- [ ] 集成测试
- [ ] 压力测试

---

## 问题追踪

### 当前问题

| ID | 描述 | 优先级 | 状态 |
|----|------|--------|------|
| #1 | Qdrant 本地部署方案 | P2 | 🔲 |
| #2 | Whisper 流式实现方案 | P1 | 🔲 |
| #3 | 并发会话隔离 | P1 | 🔲 |

### 已知限制

- WebSocket 长连接稳定性
- LLM API 速率限制
- 音频处理延迟

---

## 下周计划 (Week 2)

**目标**: API 层搭建完成

**重点**:
1. 实现 REST 路由框架
2. 实现 WebSocket handler
3. 实现 JWT 认证中间件
4. 实现速率限制

---

**最后更新**: 2024-01-15

---

## 最近更新 ✨

- ✅ 创建 CI/CD 配置
  - GitHub Actions (3 个 workflow)
  - Dependabot 自动依赖更新
  - Issue/PR 模板
  - CONTRIBUTING.md 贡献指南
- ✅ 创建 Docker 开发环境配置
  - docker-compose.yml (3 个核心服务)
  - Dockerfile + Dockerfile.dev
  - Makefile (30+ 便捷命令)
  - .env.example (完整配置模板)
- ✅ 创建 3 个数据库迁移文件 (001, 002, 003)
- ✅ 包含 6 个核心表 + pgvector 支持
- ✅ 添加性能优化索引和实用函数
- ✅ 完成所有 11 篇架构文档
