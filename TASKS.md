# NightMind 开发任务清单

> 按照 Phase 0-5 的顺序执行，直到 MVP 完成

---

## ✅ 已完成

### 项目初始化
- [x] Git 仓库初始化
- [x] 文档体系 (11 篇架构文档)
- [x] 数据库迁移 (3 个迁移文件)
- [x] Docker 开发环境
- [x] CI/CD 配置
- [x] Cargo.toml 完善

---

## 🚀 Phase 0: 基础设施 (预计 2 小时)

**目标**: 建立项目基础结构

### Step 0.1: 创建目录结构 (15 分钟)
- [ ] 创建 `src/` 子目录
  ```bash
  mkdir -p src/api/{handlers,dto}
  mkdir -p src/core/{agent,session,content}
  mkdir -p src/services/{integration,vector}
  mkdir -p src/{repository,config}
  mkdir -p tests examples
  ```

### Step 0.2: 创建模块文件 (15 分钟)
- [ ] 创建各模块的 `mod.rs`
- [ ] 创建 `src/lib.rs`
- [ ] 更新 `src/main.rs`
- [ ] 验证 `cargo check` 通过

### Step 0.3: 验证构建 (10 分钟)
- [ ] `cargo build` 成功
- [ ] `cargo test` 无错误
- [ ] `cargo clippy` 无警告

**验收**: `cargo build` 成功，无编译错误

---

## 📦 Phase 1: 配置与错误 (预计 2 小时)

**目标**: 实现配置加载和错误处理

### Step 1.1: 错误处理 (45 分钟)
- [ ] 创建 `src/error.rs`
  - [ ] 定义 `NightMindError` 枚举
  - [ ] 实现 `IntoResponse` trait
  - [ ] 添加错误转换
  - [ ] 编写错误测试

### Step 1.2: 配置系统 (45 分钟)
- [ ] 创建 `src/config/mod.rs`
- [ ] 创建 `src/config/settings.rs`
  - [ ] 定义 `Settings` 结构体
  - [ ] 实现 `Settings::load()`
  - [ ] 添加配置验证
- [ ] 创建 `src/config/logging.rs`
  - [ ] 实现 `init_tracing()`

### Step 1.3: 环境变量 (30 分钟)
- [ ] 更新 `.env.example`
- [ ] 创建本地 `.env` 文件
- [ ] 测试配置加载

**验收**: 配置能从 `.env` 正确加载

---

## 🗄️ Phase 2: 数据访问层 (预计 4 小时)

**目标**: 实现数据库操作

### Step 2.1: 数据库连接 (1 小时)
- [ ] 创建 `src/repository/mod.rs`
- [ ] 创建 `src/repository/db.rs`
  - [ ] 实现 `create_pool()`
  - [ ] 添加连接池配置
  - [ ] 实现健康检查

### Step 2.2: Redis 连接 (1 小时)
- [ ] 创建 `src/repository/redis.rs`
  - [ ] 实现 `RedisClient` 包装器
  - [ ] 实现基本操作 (get/set/delete)
  - [ ] 添加连接管理

### Step 2.3: 数据模型 (1 小时)
- [ ] 创建 `src/repository/models.rs`
  - [ ] 定义 `User` 模型
  - [ ] 定义 `Session` 模型
  - [ ] 实现 SQLx `FromRow`

### Step 2.4: Repository 实现 (1 小时)
- [ ] 创建 `src/repository/user.rs`
  - [ ] 实现 `UserRepository`
  - [ ] CRUD 操作
- [ ] 创建 `src/repository/session.rs`
  - [ ] 实现 `SessionRepository`
  - [ ] 基本查询操作

**验收**: 数据库 CRUD 操作正常

---

## 🤖 Phase 3: Agent 系统 (预计 6 小时)

**目标**: 实现 AI Agent 对话能力

### Step 3.1: Agent 基础 (2 小时)
- [ ] 创建 `src/core/agent/mod.rs`
- [ ] 创建 `src/core/agent/builder.rs`
  - [ ] 定义 `NightMindAgent` 结构体
  - [ ] 集成 Rig Agent Builder
  - [ ] 实现流式响应

### Step 3.2: Prompt 系统 (1 小时)
- [ ] 创建 `src/core/agent/prompts.rs`
  - [ ] 定义 `SYSTEM_PROMPT`
  - [ ] 实现阶段 Prompt 模板
  - [ ] 实现 Prompt 渲染函数

### Step 3.3: 会话状态 (1 小时)
- [ ] 创建 `src/core/session/mod.rs`
- [ ] 创建 `src/core/session/state.rs`
  - [ ] 定义 `SessionState` 枚举
  - [ ] 实现状态转换逻辑
  - [ ] 添加状态验证

### Step 3.4: 会话管理器 (2 小时)
- [ ] 创建 `src/core/session/manager.rs`
  - [ ] 定义 `SessionManager` 结构体
  - [ ] 实现创建会话
  - [ ] 实现处理消息
  - [ ] 实现事件广播

**验收**: Agent 能响应消息

---

## 🌐 Phase 4: API 层 (预计 6 小时)

**目标**: 实现 HTTP 和 WebSocket API

### Step 4.1: 路由定义 (1 小时)
- [ ] 创建 `src/api/mod.rs`
- [ ] 创建 `src/api/router.rs`
  - [ ] 定义 `create_router()`
  - [ ] 配置路由和中间件
  - [ ] 配置状态管理

### Step 4.2: 中间件 (2 小时)
- [ ] 创建 `src/api/middleware.rs`
  - [ ] 实现 `auth_middleware`
  - [ ] 实现 `rate_limit_middleware`
  - [ ] 实现 `request_id_middleware`

### Step 4.3: REST Handlers (1.5 小时)
- [ ] 创建 `src/api/handlers/mod.rs`
- [ ] 创建 `src/api/handlers/rest.rs`
  - [ ] 健康检查
  - [ ] 用户注册/登录
  - [ ] 创建会话
  - [ ] 获取会话列表

### Step 4.4: WebSocket Handler (1.5 小时)
- [ ] 创建 `src/api/handlers/websocket.rs`
  - [ ] 实现 WebSocket 升级
  - [ ] 定义消息类型
  - [ ] 实现消息处理循环

**验收**: WebSocket 能双向通信

---

## 🔧 Phase 5: 集成与测试 (预计 4 小时)

**目标**: 整合并测试完整流程

### Step 5.1: 主入口 (1 小时)
- [ ] 更新 `src/main.rs`
  - [ ] 加载配置
  - [ ] 初始化日志
  - [ ] 初始化组件
  - [ ] 启动服务器

### Step 5.2: 集成测试 (2 小时)
- [ ] 创建 `tests/integration.rs`
  - [ ] WebSocket 连接测试
  - [ ] 消息发送接收测试
  - [ ] 会话生命周期测试

### Step 5.3: 端到端测试 (1 小时)
- [ ] 手动测试完整流程
- [ ] 性能基准测试
- [ ] 错误场景测试

**验收**: 完整流程可用

---

## 📝 每日检查清单

### 开始开发时
- [ ] 拉取最新代码 (`git pull`)
- [ ] 检查 `cargo build` 是否通过
- [ ] 查看当前任务优先级

### 提交代码时
- [ ] `cargo fmt` 格式化
- [ ] `cargo clippy` 无警告
- [ ] `cargo test` 测试通过
- [ ] 更新相关文档

### 完成阶段时
- [ ] 更新 `todo.md`
- [ ] 更新 `EXECUTION_PLAN.md`
- [ ] 提交代码
- [ ] 创建 PR (如有需要)

---

## 🎯 MVP 最小目标

### 必须实现
- [x] 应用能启动
- [ ] WebSocket 连接成功
- [ ] 发送消息得到回复
- [ ] 错误正确处理

### 应该实现
- [ ] 用户认证
- [ ] 会话管理
- [ ] 日志记录

### 可以实现
- [ ] 速率限制
- [ ] 数据持久化
- [ ] 监控指标

---

## 📊 进度追踪

| Phase | 任务 | 预计 | 实际 | 状态 |
|-------|------|------|------|------|
| Phase 0 | 基础设施 | 2h | ___ | 🔲 |
| Phase 1 | 配置错误 | 2h | ___ | 🔲 |
| Phase 2 | 数据访问 | 4h | ___ | 🔲 |
| Phase 3 | Agent 系统 | 6h | ___ | 🔲 |
| Phase 4 | API 层 | 6h | ___ | 🔲 |
| Phase 5 | 集成测试 | 4h | ___ | 🔲 |
| **总计** | **MVP** | **24h** | ___ | 🔲 |

---

## 🚀 快速开始

```bash
# 1. 更新代码
git pull

# 2. 启动服务
make docker-up
make db-migrate

# 3. 运行开发服务器
make dev

# 4. 测试 WebSocket
wscat -c ws://localhost:8080/api/v1/ws/session
```

---

**当前阶段**: Phase 0
**当前任务**: 创建目录结构
**开始时间**: ___________
