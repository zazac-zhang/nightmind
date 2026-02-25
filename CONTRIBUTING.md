# 贡献指南

感谢你对 NightMind 项目的关注！我们欢迎任何形式的贡献。

---

## 开发环境设置

### 1. Fork 并克隆仓库

```bash
git clone https://github.com/yourusername/nightmind.git
cd nightmind
```

### 2. 安装依赖

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装 Docker (用于本地服务)
# https://docs.docker.com/get-docker/
```

### 3. 启动开发服务

```bash
make setup
```

### 4. 运行开发服务器

```bash
make dev
```

---

## 代码规范

### Rust 代码

- 使用 `cargo fmt` 格式化代码
- 通过 `cargo clippy` 检查
- 编写单元测试

```bash
make fmt
make check
make test
```

### 提交信息

遵循 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

```
<type>(<scope>): <subject>

<body>

<footer>
```

**类型 (type)**:
- `feat`: 新功能
- `fix`: Bug 修复
- `chore`: 构建/工具变更
- `docs`: 文档更新
- `style`: 代码格式
- `refactor`: 重构
- `perf`: 性能优化
- `test`: 测试相关

**示例**:
```
feat(agent): implement dynamic tool loading

Add support for loading tools from vector store based on
user query context. This allows more flexible agent behavior.

Closes #123
```

---

## Pull Request 流程

### 1. 创建分支

```bash
git checkout -b feat/your-feature-name
# 或
git checkout -b fix/your-bug-fix
```

### 2. 提交变更

```bash
git add .
git commit -m "feat(scope): description"
```

### 3. 推送到 Fork

```bash
git push origin feat/your-feature-name
```

### 4. 创建 Pull Request

在 GitHub 上创建 PR，使用提供的模板。

---

## 开发工作流

### 功能开发

1. 在 [todo.md](todo.md) 中找到或创建任务
2. 创建功能分支
3. 编写代码和测试
4. 运行 `make check` 确保代码质量
5. 提交 PR

### Bug 修复

1. 在 Issues 中报告 Bug
2. 等待确认后创建修复分支
3. 编写测试用例覆盖 Bug
4. 修复并通过测试
5. 提交 PR

---

## 项目结构

```
nightmind/
├── src/              # 源代码
├── doc/              # 架构文档
├── migrations/       # 数据库迁移
├── tests/            # 集成测试
├── docker/           # Docker 配置
└── Makefile          # 开发命令
```

---

## 测试指南

### 单元测试

```bash
# 运行所有测试
make test

# 运行特定测试
cargo test test_name

# 运行测试并显示输出
cargo test -- --nocapture

# 生成覆盖率报告
make test-coverage
```

### 集成测试

```bash
# 启动测试环境
docker-compose -f docker-compose.test.yml up -d

# 运行集成测试
cargo test --test integration '*'
```

---

## 文档贡献

### 架构文档

架构文档位于 `doc/` 目录，遵循以下约定：
- 使用 Markdown 格式
- 代码使用 rust 语法高亮
- 包含图表和示例

### API 文档

```bash
# 生成并打开 API 文档
make docs
```

---

## 发布流程

1. 更新 `CHANGELOG.md`
2. 创建版本分支 `release/vX.X.X`
3. 更新版本号
4. 创建 Git tag
5. GitHub Actions 自动构建发布

---

## 获取帮助

- 查看 [README.md](README.md)
- 查看 [doc/](doc/) 目录下的架构文档
- 在 Issues 中提问

---

## 行为准则

- 尊重不同观点
- 建设性地反馈
- 关注项目本身而非个人

---

## 许可证

贡献的代码将采用 MIT 许可证发布。
