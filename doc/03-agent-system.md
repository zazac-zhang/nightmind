# Agent 系统设计

## 1. 系统概述

NightMind 使用 [Rig](https://rig.rs/) 作为 Agent 框架，构建具备记忆、工具调用、RAG 能力的智能伴学 Agent。

---

## 2. 核心组件

### 2.1 NightMindAgent

Agent 的核心封装，负责：
- LLM 交互（调用、流式响应）
- 上下文管理（动态检索）
- 工具编排（静态 + 动态工具）

```rust
pub struct NightMindAgent {
    agent: Agent<CompletionModel, VectorStoreIndex>,
    embedding_model: EmbeddingModel,
}

impl NightMindAgent {
    /// 创建新 Agent 实例
    pub async fn new(
        client: &openai::Client,
        vector_store: &VectorStoreService,
    ) -> Result<Self>;

    /// 单次提示（非流式）
    pub async fn prompt(&self, message: &str) -> Result<String>;

    /// 流式提示
    pub async fn prompt_stream(&self, message: &str)
        -> Result<impl Stream<Item = String>>;

    /// 带状态提示
    pub async fn prompt_with_state(
        &self,
        message: &str,
        state: &SessionState,
    ) -> Result<String>;
}
```

### 2.2 Agent Builder

使用 Rig 的流式 API 构建 Agent：

```rust
let agent = openai_client
    .agent("gpt-4-turbo-preview")
    // 系统提示词
    .preamble(SELF_PREAMBLE)
    // 动态上下文（RAG）
    .dynamic_context(3, context_index)
    // 动态工具
    .dynamic_tools(2, tool_index, toolset)
    // 静态工具
    .tool(ContentTransformerTool)
    .tool(FatigueDetectorTool)
    // 参数配置
    .temperature(0.7)
    .max_tokens(1000)
    .build();
```

---

## 3. Prompt 设计

### 3.1 系统 Preamble

```
你是 NightMind，一位专业的睡前认知巩固导师。

## 你的使命
帮助用户在睡前 30-60 分钟的黄金窗口期，通过深度对话巩固今日所学，
利用睡眠时间进行"后台加工"。

## 核心原则
1. 零交互原则：所有内容通过语音传递，绝不要求用户阅读
2. 认知优先原则：用户脑力用于思考，你负责所有技术细节
3. 意象优于定义：拒绝朗读定义，使用比喻、类比、故事

## 内容转化规则
- ❌ 不要朗读代码/公式，转为解释设计意图
- ✅ 使用"就像..."的比喻方式解释概念
- ✅ 每段控制在 15 秒内朗读完
- ✅ 留出思考间隙（3-5 秒静默）

## 会话节奏（60 分钟）
- 00-05m: 暖场校准（确认状态）
- 05-25m: 核心深度区（苏格拉底式追问）
- 25-45m: 关联巩固区（间隔复习、知识网络）
- 45-55m: 种子沉淀区（潜意识问题）
- 55-60m: 温柔收尾（肯定、放下）

## 睡眠保护
- 检测用户疲劳度，超过阈值主动建议休息
- 会话结束必须以心理放松为终点
- 用户说"晚安"时立即切换为助眠模式
```

### 3.2 阶段性 Prompt

| 阶段 | Prompt 特点 |
|------|-------------|
| **Warmup** | 简短问题、确认状态、今日概览 |
| **DeepDive** | 苏格拉底式追问、深挖概念、建立直觉 |
| **Review** | 关联已有知识、寻找网络连接 |
| **Seed** | 开放性问题、激发潜意识思考 |
| **Closing** | 肯定总结、放下焦虑、白噪音 |

### 3.3 动态 Prompt 模板

```rust
pub struct PromptTemplate {
    pub base: String,
    pub phase_modifiers: HashMap<SessionState, String>,
}

impl PromptTemplate {
    pub fn render(&self, state: &SessionState, context: &Context)
        -> String
    {
        let mut prompt = self.base.clone();

        // 添加阶段修饰
        if let Some(modifier) = self.phase_modifiers.get(state) {
            prompt.push_str(modifier);
        }

        // 添加动态上下文
        for doc in &context.relevant_docs {
            prompt.push_str(&format!("\n参考: {}\n", doc.content));
        }

        prompt
    }
}
```

---

## 4. 工具系统

### 4.1 工具分类

| 类型 | 工具 | 功能 |
|------|------|------|
| **静态工具** | ContentTransformer | 内容格式转换 |
| | FatigueDetector | 疲劳度评估 |
| | KnowledgeRetriever | 知识检索 |
| **动态工具** | AnkiIntegration | Anki 同步 |
| | ObsidianSync | Obsidian 同步 |
| | NotionSync | Notion 同步 |

### 4.2 工具定义接口

```rust
pub trait NightMindTool: Tool + Send + Sync {
    /// 工具名称
    fn name(&self) -> &str;

    /// 工具描述（用于 LLM 理解）
    fn description(&self) -> &str;

    /// 执行工具
    async fn execute(&self, input: &ToolInput) -> Result<ToolOutput>;

    /// 是否适合当前上下文
    fn is_applicable(&self, context: &SessionContext) -> bool;
}
```

### 4.3 示例工具

#### ContentTransformerTool

```rust
pub struct ContentTransformerTool {
    llm_client: Arc<openai::Client>,
}

impl Tool for ContentTransformerTool {
    const NAME: &'static str = "content_transformer";
    type Error = anyhow::Error;

    async fn call(&self, input: &str) -> Result<String> {
        // 检查输入
        self.validate(input)?;

        // 使用 LLM 转换
        let prompt = format!(
            "将以下内容转换为适合语音朗读的形式：\n{}",
            input
        );

        let response = self.llm_client
            .completion_model("gpt-4")
            .prompt(&prompt)
            .await?;

        Ok(response
    }
}

impl ContentTransformerTool {
    fn validate(&self, content: &str) -> Result<()> {
        // 检查是否包含代码、公式
        if content.contains("```") {
            return Err(anyhow!("包含代码块，需要转换"));
        }
        Ok(())
    }
}
```

#### FatigueDetectorTool

```rust
pub struct FatigueDetectorTool {
    response_time_threshold: Duration,
    silence_threshold: Duration,
}

impl Tool for FatigueDetectorTool {
    const NAME: &'static str = "fatigue_detector";
    type Error = anyhow::Error;

    async fn call(&self, input: &str) -> Result<String> {
        let metrics: SessionMetrics = serde_json::from_str(input)?;

        let fatigue_score = self.calculate_fatigue(&metrics);

        if fatigue_score > 80 {
            Ok("用户疲劳度过高，建议结束会话".to_string())
        } else {
            Ok(format!("用户疲劳度: {}/100", fatigue_score))
        }
    }
}

impl FatigueDetectorTool {
    fn calculate_fatigue(&self, metrics: &SessionMetrics) -> u8 {
        let mut score = 0u8;

        // 响应时间
        if metrics.avg_response_time > self.response_time_threshold {
            score += 30;
        }

        // 沉默时长
        if metrics.silence_duration > self.silence_threshold {
            score += 20;
        }

        // 回答质量下降
        if metrics.response_quality < 0.5 {
            score += 30;
        }

        // 会话时长
        if metrics.session_duration > Duration::from_secs(3600) {
            score += 20;
        }

        score.min(100)
    }
}
```

### 4.4 动态工具加载

```rust
// 从向量存储加载工具
let tool_index = vector_store.index(embedding_model).await?;

let agent = openai_client
    .agent("gpt-4")
    .dynamic_tools(
        2,              // 最多加载 2 个动态工具
        tool_index,     // 工具向量索引
        toolset,        // 可用工具集
    )
    .build();
```

---

## 5. 记忆系统

### 5.1 记忆类型

| 类型 | 存储 | 用途 | 生命周期 |
|------|------|------|----------|
| **短期记忆** | Session State | 当前会话上下文 | 会话结束 |
| **中期记忆** | Redis | 最近几次会话摘要 | 7 天 |
| **长期记忆** | PostgreSQL + Vector Store | 知识图谱、用户偏好 | 永久 |

### 5.2 记忆写入

```rust
pub struct MemoryWriter {
    db: PgPool,
    vector_store: VectorStoreService,
}

impl MemoryWriter {
    pub async fn write_knowledge(
        &self,
        session_id: Uuid,
        content: &str,
        source: KnowledgeSource,
    ) -> Result<()> {
        // 1. 生成 embedding
        let embedding = self.vector_store
            .embed(content)
            .await?;

        // 2. 保存到数据库
        sqlx::query!(
            "INSERT INTO knowledge_points (session_id, content, source, embedding)
             VALUES ($1, $2, $3, $4)",
            session_id,
            content,
            source as KnowledgeSource,
            embedding as Vector,
        )
        .execute(&self.db)
        .await?;

        // 3. 添加到向量索引
        self.vector_store.add(Document {
            id: Uuid::new_v4(),
            content: content.to_string(),
            embedding,
        }).await?;

        Ok(())
    }
}
```

### 5.3 记忆检索

```rust
pub struct MemoryRetriever {
    vector_store: VectorStoreService,
}

impl MemoryRetriever {
    pub async fn retrieve_context(
        &self,
        query: &str,
        user_id: Uuid,
        limit: usize,
    ) -> Result<Vec<Document>> {
        self.vector_store
            .search(query, limit)
            .await?
            .into_iter()
            .filter(|doc| doc.user_id == user_id)
            .collect()
    }
}
```

---

## 6. RAG (检索增强生成)

### 6.1 向量存储架构

```
┌─────────────────────────────────────────────────────────┐
│                    Vector Store                         │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐    │
│  │  Knowledge  │  │   Tools     │  │ Prompts     │    │
│  │  Index      │  │   Index     │  │  Index      │    │
│  └─────────────┘  └─────────────┘  └─────────────┘    │
│                                                         │
│  ┌─────────────────────────────────────────────────┐   │
│  │              Qdrant Collection                  │   │
│  │  - documents: 1536-dim embeddings               │   │
│  │  - metadata: user_id, source, created_at        │   │
│  │  - payload: original content                    │   │
│  └─────────────────────────────────────────────────┘   │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### 6.2 动态上下文注入

```rust
pub struct DynamicContextInjector {
    vector_store: VectorStoreService,
}

impl DynamicContextInjector {
    pub async fn inject(
        &self,
        agent: &mut Agent,
        query: &str,
        num_docs: usize,
    ) -> Result<()> {
        let docs = self.vector_store
            .search(query, num_docs)
            .await?;

        agent.set_context(docs)?;
        Ok(())
    }
}
```

---

## 7. 流式处理

### 7.1 流式响应流程

```
User Input
    ↓
Agent.prompt_stream()
    ↓
LLM Stream (Token by Token)
    ↓
Chunk Accumulator ( accumulate to sentence )
    ↓
TTS Service
    ↓
Audio Stream to Client
```

### 7.2 流式处理实现

```rust
pub async fn stream_response(
    &self,
    session_id: Uuid,
    message: &str,
    event_tx: broadcast::Sender<SessionEvent>,
) -> Result<()> {
    let mut stream = self.agent.prompt_stream(message).await?;

    let mut buffer = String::new();
    let mut sentence_end = ['.', '。', '!', '！', '?', '？'];

    while let Some(chunk) = stream.next().await {
        let token = chunk?;
        buffer.push_str(&token);

        // 检测句子结束
        if sentence_end.contains(&token.chars().last().unwrap_or(' ')) {
            // 发送 TTS
            let audio = self.tts_service
                .synthesize(&buffer)
                .await?;

            event_tx.send(SessionEvent::AudioTTS(audio))?;
            buffer.clear();
        }
    }

    Ok(())
}
```

---

## 8. 多模型支持

### 8.1 模型配置

```rust
pub enum LlmProvider {
    OpenAI { model: String },
    Anthropic { model: String },
    Gemini { model: String },
    Local { model: String },  // Ollama
}

pub struct AgentConfig {
    pub provider: LlmProvider,
    pub fallback_provider: Option<LlmProvider>,
    pub max_retries: u32,
}
```

### 8.2 降级策略

```rust
impl NightMindAgent {
    pub async fn prompt_with_fallback(&self, message: &str)
        -> Result<String>
    {
        match self.agent.prompt(message).await {
            Ok(response) => Ok(response),
            Err(e) if e.is_transient() => {
                tracing::warn!("Primary provider failed, trying fallback");
                self.fallback_agent.prompt(message).await
            }
            Err(e) => Err(e.into()),
        }
    }
}
```

---

## 9. 工具调用流程

```
User Message
    ↓
Agent Analysis
    ↓
Tool Selection (based on context)
    ↓
Tool Execution (parallel if possible)
    ↓
Result Aggregation
    ↓
Response Generation
```

### 9.1 并行工具调用

```rust
let (anki_result, obsidian_result) = tokio::join!(
    anki_tool.fetch_due_cards(),
    obsidian_tool.sync_recent_notes(),
);
```
