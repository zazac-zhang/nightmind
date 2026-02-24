# 外部集成设计

## 1. 概述

NightMind 作为"智能增强层"，需要与现有学习工具无缝集成。本文档定义各集成的接口与协议。

---

## 2. Anki 集成

### 2.1 集成方式

通过 [AnkiConnect](https://foosoft.net/projects/anki-connect/) API 进行双向通信。

### 2.2 核心接口

```rust
pub struct AnkiService {
    connect_url: String,
    client: reqwest::Client,
}

#[async_trait]
pub trait AnkiIntegration: Send + Sync {
    /// 获取今日待复习卡片
    async fn get_due_cards(&self, deck_name: Option<String>)
        -> Result<Vec<AnkiCard>>;

    /// 提交复习结果
    async fn submit_review(
        &self,
        card_id: u64,
        rating: ReviewRating,
    ) -> Result<()>;

    /// 创建新卡片
    async fn create_card(
        &self,
        deck: String,
        front: String,
        back: String,
        tags: Vec<String>,
    ) -> Result<u64>;

    /// 获取卡片统计数据
    async fn get_card_stats(&self, card_id: u64)
        -> Result<CardStats>;
}

#[derive(Debug, Clone)]
pub struct AnkiCard {
    pub id: u64,
    pub note_id: u64,
    pub deck: String,
    pub front: String,
    pub back: String,
    pub tags: Vec<String>,
    pub due: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy)]
pub enum ReviewRating {
    Again,   // < 2.5
    Hard,    // 2.5 - 3.5
    Good,    // 3.5 - 4.5
    Easy,    // > 4.5
}

#[derive(Debug, Clone)]
pub struct CardStats {
    pub interval: u32,      // 当前间隔（天）
    pub ease_factor: f32,   // 难度因子
    pub reviews: u32,       // 复习次数
    pub lapses: u32,        // 遗忘次数
}
```

### 2.3 会话集成流程

```
1. 会话开始时
   └─> get_due_cards() → 获取今日复习列表
   └─> 按优先级排序（ overdue > due_date）

2. 对话过程中
   └─> Agent 将卡片内容转换为对话问题
   └─> 用户回答后，调用 submit_review()

3. 会话结束时
   └─> 生成的新理解 → create_card()
   └─> 写入用户日志牌组
```

### 2.4 智能复习问题生成

```rust
pub struct CardToQuestionConverter {
    llm_client: Arc<openai::Client>,
}

impl CardToQuestionConverter {
    pub async fn convert(&self, card: &AnkiCard, context: &ReviewContext)
        -> Result<ReviewQuestion>
    {
        let prompt = format!(
            "将以下 Anki 卡片转换为适合睡前语音对话的问题。

            卡片正面：{}
            卡片背面：{}

            要求：
            1. 不要直接朗读背面内容
            2. 使用引导式提问（'你还记得...''能不能说说...'）
            3. 给用户留出思考空间
            4. 如果有代码/公式，使用比喻
            ",
            card.front, card.back
        );

        let question_text = self.llm_client
            .completion_model("gpt-4")
            .prompt(&prompt)
            .await?;

        Ok(ReviewQuestion {
            card_id: card.id,
            question: question_text,
            expected_keywords: self.extract_keywords(&card.back),
            hint: self.generate_hint(&card.back),
        })
    }
}
```

---

## 3. Obsidian 集成

### 3.1 集成方式

文件系统监听 + Markdown 解析。

### 3.2 核心接口

```rust
pub struct ObsidianService {
    vault_path: PathBuf,
    watcher: RecommendedWatcher,
    index: Arc<RwLock<ObsidianIndex>>,
}

#[async_trait]
pub trait ObsidianIntegration: Send + Sync {
    /// 启动文件监听
    async fn start_watching(&self) -> Result<()>;

    /// 获取最近修改的笔记
    async fn get_recent_notes(&self, since: DateTime<Utc>)
        -> Result<Vec<ObsidianNote>>;

    /// 搜索笔记
    async fn search_notes(&self, query: &str)
        -> Result<Vec<ObsidianNote>>;

    /// 写入每日日志
    async fn write_daily_note(&self, date: NaiveDate, content: &str)
        -> Result<()>;

    /// 发现笔记间关联
    async fn find_connections(&self, note_id: &str)
        -> Result<Vec<NoteConnection>>;
}

#[derive(Debug, Clone)]
pub struct ObsidianNote {
    pub id: String,              // 文件名
    pub path: PathBuf,
    pub content: String,
    pub frontmatter: FrontMatter,
    pub links: Vec<String>,      // [[链接]]
    pub tags: Vec<String>,
    pub modified: DateTime<Utc>,
    pub created: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NoteConnection {
    pub from_note: String,
    pub to_note: String,
    pub connection_type: ConnectionType,
    pub context: String,
}

#[derive(Debug, Clone)]
pub enum ConnectionType {
    DirectLink,      // [[直接链接]]
    TagReference,    // 共同标签
    SemanticSimilar, // 语义相似
}
```

### 3.3 双向同步流程

#### 被动摄入

```rust
impl ObsidianService {
    pub async fn on_file_change(&self, event: NotifyEvent) {
        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) => {
                if let Ok(path) = event.path.strip_prefix(&self.vault_path) {
                    if let Some(note) = self.parse_note(path).await {
                        // 添加到向量存储
                        self.vector_store.add_note(&note).await;

                        // 更新索引
                        self.index.write().await.insert(note.id.clone(), note);
                    }
                }
            }
            _ => {}
        }
    }

    async fn parse_note(&self, path: &Path) -> Result<ObsidianNote> {
        let full_path = self.vault_path.join(path);
        let content = tokio::fs::read_to_string(&full_path).await?;

        Ok(ObsidianNote {
            id: path.file_stem()?.to_str()?.to_string(),
            path: full_path,
            content: content.clone(),
            frontmatter: Self::parse_frontmatter(&content)?,
            links: Self::extract_links(&content),
            tags: Self::extract_tags(&content),
            modified: self.get_modified_time(&full_path).await?,
            created: self.get_created_time(&full_path).await?,
        })
    }
}
```

#### 主动回写

```rust
impl ObsidianService {
    pub async fn write_daily_note(
        &self,
        date: NaiveDate,
        session_summary: &SessionSummary,
    ) -> Result<()> {
        let filename = format!("{}.md", date.format("%Y-%m-%d"));
        let path = self.vault_path.join(filename);

        let content = format!(
            "---\n\
            date: {}\n\
            type: nightmind-session\n\
            ---\n\n\
            # NightMind 会话记录\n\n\
            ## 今晚讨论\n\n\
            {}\n\n\
            ## 新的理解\n\n\
            {}\n\n\
            ## 相关笔记\n\n\
            {}\n\n\
            ---\n\
            _Generated by [NightMind](https://nightmind.dev)_\n\
            ",
            date,
            session_summary.topics.join("\n- "),
            session_summary.insights.join("\n- "),
            session_summary.related_notes.join("\n- ")
        );

        tokio::fs::write(path, content).await?;
        Ok(())
    }
}
```

---

## 4. Notion 集成

### 4.1 集成方式

通过 [Notion API](https://developers.notion.com/)。

### 4.2 核心接口

```rust
pub struct NotionService {
    api_key: String,
    database_id: String,
    client: reqwest::Client,
}

#[async_trait]
pub trait NotionIntegration: Send + Sync {
    /// 查询数据库
    async fn query_database(
        &self,
        filter: Option<NotionFilter>,
    ) -> Result<Vec<NotionPage>>;

    /// 创建页面
    async fn create_page(&self, parent: DatabaseParent, properties: PageProperties)
        -> Result<NotionPage>;

    /// 追加块内容
    async fn append_blocks(&self, block_id: &str, blocks: Vec<Block>)
        -> Result<()>;

    /// 搜索
    async fn search(&self, query: &str)
        -> Result<Vec<NotionPage>>;
}

#[derive(Debug, Clone)]
pub struct NotionPage {
    pub id: String,
    pub title: String,
    pub content: String,
    pub properties: HashMap<String, PropertyValue>,
    pub created_time: DateTime<Utc>,
    pub last_edited_time: DateTime<Utc>,
}
```

### 4.3 数据库映射

```rust
pub struct NotionDatabaseConfig {
    pub knowledge_base_id: String,
    pub session_log_id: String,
    pub review_queue_id: String,
}

// 知识库数据库结构
// - title: 标题
// - content: 内容
// - source: 来源 (手动/Anki/Obsidian)
// - tags: 标签
// - last_reviewed: 最后复习时间
// - review_interval: 复习间隔
```

---

## 5. Readwise 集成

### 5.1 集成方式

通过 [Readwise API](https://readwise.io/api_deployment)。

### 5.2 核心接口

```rust
pub struct ReadwiseService {
    api_key: String,
    client: reqwest::Client,
}

#[async_trait]
pub trait ReadwiseIntegration: Send + Sync {
    /// 获取最近高亮
    async fn get_recent_highlights(&self, updated_after: Option<DateTime<Utc>>)
        -> Result<Vec<Highlight>>;

    /// 更新高亮状态
    async fn update_highlight(
        &self,
        highlight_id: &str,
        notes: Option<&str>,
    ) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct Highlight {
    pub id: String,
    pub text: String,
    pub note: Option<String>,
    pub book_id: String,
    pub book_title: String,
    pub highlighted_at: DateTime<Utc>,
}
```

### 5.3 费曼技巧处理

```rust
impl ReadwiseService {
    pub async fn process_highlights_for_review(
        &self,
        highlights: Vec<Highlight>,
    ) -> Result<Vec<ReviewItem>> {
        let mut items = Vec::new();

        for highlight in highlights {
            // 让用户用自己的话复述高亮内容
            let item = ReviewItem {
                id: highlight.id.clone(),
                content: highlight.text.clone(),
                prompt: format!(
                    "你在 '{}' 中标记了这段话。能不能用自己的话解释一下它的含义？",
                    highlight.book_title
                ),
                source: ReviewSource::Readwise,
            };
            items.push(item);
        }

        Ok(items)
    }
}
```

---

## 6. 集成编排器

### 6.1 统一接口

```rust
pub struct IntegrationOrchestrator {
    anki: Option<Arc<AnkiService>>,
    obsidian: Option<Arc<ObsidianService>>,
    notion: Option<Arc<NotionService>>,
    readwise: Option<Arc<ReadwiseService>>,
}

impl IntegrationOrchestrator {
    /// 会话开始：收集所有来源的内容
    pub async fn gather_session_context(
        &self,
        user_id: Uuid,
    ) -> Result<SessionContext> {
        let mut context = SessionContext::default();

        // 1. Anki: 今日复习卡片
        if let Some(anki) = &self.anki {
            context.due_cards = anki.get_due_cards(None).await?;
        }

        // 2. Obsidian: 最近修改的笔记
        if let Some(obsidian) = &self.obsidian {
            context.recent_notes = obsidian.get_recent_notes(
                Utc::now() - Duration::from_secs(86400 * 7)  // 7天内
            ).await?;
        }

        // 3. Notion: 最近更新的页面
        if let Some(notion) = &self.notion {
            let results = notion.search(
                &format!("user:{}", user_id)
            ).await?;
            context.notion_pages = results;
        }

        // 4. Readwise: 最近高亮
        if let Some(readwise) = &self.readwise {
            context.highlights = readwise.get_recent_highlights(None).await?;
        }

        Ok(context)
    }

    /// 会话结束：同步新生成的内容
    pub async fn sync_session_results(
        &self,
        session: &Session,
        summary: &SessionSummary,
    ) -> Result<SyncResult> {
        let mut result = SyncResult::default();

        // 1. 写入 Obsidian 每日日志
        if let Some(obsidian) = &self.obsidian {
            if let Err(e) = obsidian.write_daily_note(
                session.start_time.date_naive(),
                summary,
            ).await {
                tracing::warn!("Failed to write daily note: {:?}", e);
            } else {
                result.obsidian_synced = true;
            }
        }

        // 2. 创建 Anki 卡片
        if let Some(anki) = &self.anki {
            for insight in &summary.new_insights {
                match anki.create_card(
                    "NightMind".to_string(),
                    insight.question.clone(),
                    insight.answer.clone(),
                    vec!["nightmind".to_string()],
                ).await {
                    Ok(_) => result.anki_cards_created += 1,
                    Err(e) => tracing::warn!("Failed to create Anki card: {:?}", e),
                }
            }
        }

        // 3. 更新 Notion 数据库
        if let Some(notion) = &self.notion {
            // 实现同步逻辑
        }

        Ok(result)
    }
}
```

### 6.2 同步状态跟踪

```rust
#[derive(Debug, Clone, Default)]
pub struct SyncResult {
    pub obsidian_synced: bool,
    pub anki_cards_created: u32,
    pub notion_pages_created: u32,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SyncStatus {
    pub last_sync: DateTime<Utc>,
    pub obsidian_enabled: bool,
    pub anki_enabled: bool,
    pub notion_enabled: bool,
    pub readwise_enabled: bool,
}
```

---

## 7. 集成配置

### 7.1 配置结构

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct IntegrationConfig {
    pub anki: Option<AnkiConfig>,
    pub obsidian: Option<ObsidianConfig>,
    pub notion: Option<NotionConfig>,
    pub readwise: Option<ReadwiseConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnkiConfig {
    pub connect_url: String,
    pub default_deck: String,
    pub sync_on_session_end: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ObsidianConfig {
    pub vault_path: PathBuf,
    pub daily_note_template: String,
    pub auto_sync: bool,
    pub watch_folders: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NotionConfig {
    pub api_key: String,
    pub knowledge_base_id: String,
    pub session_log_id: String,
}
```

### 7.2 配置加载

```rust
impl IntegrationConfig {
    pub fn from_env() -> Result<Self> {
        let config = config::Config::builder()
            .add_source(config::File::with_name("config/integrations"))
            .add_source(config::Environment::with_prefix("NIGHTMIND"))
            .build()?;

        Ok(config.try_deserialize()?)
    }
}
```

---

## 8. 错误处理

### 8.1 集成错误类型

```rust
#[derive(Debug, thiserror::Error)]
pub enum IntegrationError {
    #[error("AnkiConnect unavailable: {0}")]
    AnkiConnectUnavailable(String),

    #[error("Obsidian vault not found: {0}")]
    VaultNotFound(PathBuf),

    #[error("Notion API error: {0}")]
    NotionApiError(String),

    #[error("Readwise API error: {0}")]
    ReadwiseApiError(String),

    #[error("Sync failed for {service}: {reason}")]
    SyncFailed { service: String, reason: String },
}
```

### 8.2 降级策略

```rust
impl IntegrationOrchestrator {
    pub async fn sync_with_fallback(
        &self,
        session: &Session,
        summary: &SessionSummary,
    ) -> SyncResult {
        let mut result = SyncResult::default();

        // 尝试所有集成，记录失败但不中断
        if let Some(anki) = &self.anki {
            match self.sync_to_anki(anki, summary).await {
                Ok(r) => result.anki_cards_created = r,
                Err(e) => result.errors.push(format!("Anki: {}", e)),
            }
        }

        // ... 其他集成

        result
    }
}
```
