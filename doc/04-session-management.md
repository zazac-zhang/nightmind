# 会话管理设计

## 1. 概述

会话管理是 NightMind 的核心，负责：
- 会话生命周期管理
- 状态机驱动
- 快照与恢复
- 话题栈管理
- 事件广播

---

## 2. 会话状态机

### 2.1 状态定义

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    /// 00-05m: 暖场校准
    Warmup,

    /// 05-25m: 核心深度区
    DeepDive,

    /// 25-45m: 关联巩固区
    Review,

    /// 45-55m: 种子沉淀区
    Seed,

    /// 55-60m: 温柔收尾
    Closing,

    /// 会话结束
    Closed,
}

impl SessionState {
    pub fn phase_name(&self) -> &'static str {
        match self {
            Self::Warmup => "暖场校准",
            Self::DeepDive => "核心深度",
            Self::Review => "关联巩固",
            Self::Seed => "种子沉淀",
            Self::Closing => "温柔收尾",
            Self::Closed => "已关闭",
        }
    }

    pub fn duration_hint(&self) -> Duration {
        match self {
            Self::Warmup => Duration::from_secs(300),      // 5min
            Self::DeepDive => Duration::from_secs(1200),   // 20min
            Self::Review => Duration::from_secs(1200),     // 20min
            Self::Seed => Duration::from_secs(600),        // 10min
            Self::Closing => Duration::from_secs(300),     // 5min
            Self::Closed => Duration::ZERO,
        }
    }

    pub fn cognitive_load(&self) -> CognitiveLoad {
        match self {
            Self::Warmup => CognitiveLoad::Low,
            Self::DeepDive => CognitiveLoad::High,
            Self::Review => CognitiveLoad::Medium,
            Self::Seed => CognitiveLoad::Low,
            Self::Closing => CognitiveLoad::VeryLow,
            Self::Closed => CognitiveLoad::None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CognitiveLoad {
    None,
    VeryLow,
    Low,
    Medium,
    High,
}
```

### 2.2 状态转换

```rust
impl SessionState {
    pub fn can_transition_to(&self, next: &SessionState) -> bool {
        match (self, next) {
            // 正常流程
            (Self::Warmup, Self::DeepDive) => true,
            (Self::DeepDive, Self::Review) => true,
            (Self::Review, Self::Seed) => true,
            (Self::Seed, Self::Closing) => true,
            (Self::Closing, Self::Closed) => true,

            // 提前结束
            (_, Self::Closing) => true,
            (_, Self::Closed) => true,

            // 禁止其他转换
            _ => false,
        }
    }

    pub fn transition(&self, next: &SessionState) -> Result<SessionState> {
        if self.can_transition_to(next) {
            Ok(*next)
        } else {
            Err(anyhow::anyhow!(
                "Invalid state transition: {:?} -> {:?}",
                self, next
            ))
        }
    }
}
```

### 2.3 状态转换触发器

```rust
pub enum TransitionTrigger {
    /// 时间到达
    TimeElapsed,

    /// 用户疲劳度过高
    FatigueDetected(u8),

    /// 用户主动结束
    UserInitiated,

    /// 无响应超时
    NoResponseTimeout,

    /// 检测到入睡
    SleepDetected,
}

impl SessionState {
    pub fn should_transition(
        &self,
        trigger: &TransitionTrigger,
        session: &Session,
    ) -> Option<SessionState> {
        match (self, trigger) {
            // 疲劳检测：任何阶段都可能提前进入收尾
            (_, TransitionTrigger::FatigueDetected(score)) if *score > 80 => {
                Some(SessionState::Closing)
            }

            // 用户主动结束
            (_, TransitionTrigger::UserInitiated) => Some(SessionState::Closing),

            // 检测到入睡
            (_, TransitionTrigger::SleepDetected) => Some(SessionState::Closed),

            // 正常时间推进
            (Self::Warmup, TransitionTrigger::TimeElapsed)
                if session.elapsed() >= Self::Warmup.duration_hint()
                => Some(SessionState::DeepDive),

            (Self::DeepDive, TransitionTrigger::TimeElapsed)
                if session.elapsed() >= Self::Warmup.duration_hint()
                    + Self::DeepDive.duration_hint()
                => Some(SessionState::Review),

            // ... 其他阶段

            _ => None,
        }
    }
}
```

---

## 3. 会话管理器

### 3.1 核心接口

```rust
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<Uuid, Session>>>,
    agent: Arc<NightMindAgent>,
    snapshot_store: Arc<SnapshotStore>,
    rhythm_controller: Arc<RhythmController>,
}

impl SessionManager {
    pub fn new(
        agent: Arc<NightMindAgent>,
        snapshot_store: Arc<SnapshotStore>,
    ) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            agent,
            snapshot_store,
            rhythm_controller: Arc::new(RhythmController::default()),
        }
    }

    /// 创建新会话
    pub async fn create_session(
        &self,
        user_id: Uuid,
        event_tx: broadcast::Sender<SessionEvent>,
    ) -> Result<Uuid> {
        let session_id = Uuid::new_v4();
        let session = Session::new(session_id, user_id, event_tx);

        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id, session);
        }

        // 发送欢迎消息
        self.send_event(session_id, SessionEvent::TextResponse(
            "晚上好，准备好回顾今天了吗？".to_string()
        )).await?;

        Ok(session_id)
    }

    /// 处理用户输入
    pub async fn handle_transcript(
        &self,
        session_id: Uuid,
        transcript: String,
    ) -> Result<()> {
        // 检查会话是否存在
        let session = self.get_session(session_id).await?;

        // 更新活动时间
        session.update_activity();

        // 保存快照
        self.save_snapshot(session_id).await?;

        // 处理特殊命令
        match transcript.trim() {
            "晚安" | "睡觉了" | "晚安 NightMind" => {
                self.transition_to(session_id, SessionState::Closing).await?;
                return Ok(());
            }
            "暂停" | "等一下" => {
                self.pause_session(session_id).await?;
                return Ok(());
            }
            "继续" => {
                self.resume_session(session_id).await?;
                return Ok(());
            }
            _ => {}
        }

        // 发送给 Agent 处理
        let agent = self.agent.clone();
        let event_tx = session.event_tx.clone();
        let session_id = session_id;

        tokio::spawn(async move {
            match agent.prompt_stream(&transcript).await {
                Ok(stream) => {
                    Self::stream_response(session_id, stream, event_tx).await;
                }
                Err(e) => {
                    tracing::error!("Agent error: {:?}", e);
                    let _ = event_tx.send(SessionEvent::Error(
                        "抱歉，我遇到了一些问题。".to_string()
                    ));
                }
            }
        });

        // 检查状态转换
        self.check_state_transition(session_id).await?;

        Ok(())
    }

    /// 状态转换
    pub async fn transition_to(
        &self,
        session_id: Uuid,
        new_state: SessionState,
    ) -> Result<()> {
        let mut session = self.get_session_mut(session_id).await?;

        let old_state = session.state;
        session.state = old_state.transition(&new_state)?;

        // 发送状态变更事件
        let _ = session.event_tx.send(SessionEvent::StateChanged {
            old: old_state,
            new: new_state,
        });

        // 根据新状态发送提示
        match new_state {
            SessionState::Closing => {
                let _ = session.event_tx.send(SessionEvent::TextResponse(
                    "时间过得很快，今晚的对话就到这里吧。愿你带着这些新的思考，安然入梦。".to_string()
                ));
            }
            SessionState::Closed => {
                let _ = session.event_tx.send(SessionEvent::Closing);
            }
            _ => {}
        }

        Ok(())
    }

    /// 恢复会话（从中断）
    pub async fn restore_session(&self, session_id: Uuid)
        -> Result<RestoreResult>
    {
        // 加载快照
        let snapshot = self.snapshot_store.load(session_id).await?
            .ok_or_else(|| anyhow!("No snapshot found"))?;

        // 恢复会话状态
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            session.restore_from_snapshot(&snapshot)?;
        }

        // 构建恢复提示
        let recovery_prompt = snapshot.topic_stack.recovery_prompt();

        Ok(RestoreResult {
            can_restore: true,
            recovery_prompt,
            elapsed: snapshot.elapsed,
        })
    }

    /// 关闭会话
    pub async fn close_session(&self, session_id: Uuid) -> Result<()> {
        let mut sessions = self.sessions.write().await;

        if let Some(mut session) sessions.remove(&session_id) {
            // 保存最终快照
            self.snapshot_store.save(session.final_snapshot()).await?;

            // 发送关闭事件
            let _ = session.event_tx.send(SessionEvent::Closing);

            // 同步到外部服务
            self.sync_session_data(&session).await?;
        }

        Ok(())
    }
}
```

### 3.2 会话结构

```rust
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub state: SessionState,
    pub topic_stack: TopicStack,
    pub start_time: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub event_tx: broadcast::Sender<SessionEvent>,
    pub metrics: SessionMetrics,
    pub context: SessionContext,
}

impl Session {
    pub fn new(
        id: Uuid,
        user_id: Uuid,
        event_tx: broadcast::Sender<SessionEvent>,
    ) -> Self {
        Self {
            id,
            user_id,
            state: SessionState::Warmup,
            topic_stack: TopicStack::new(3),
            start_time: Utc::now(),
            last_activity: Utc::now(),
            event_tx,
            metrics: SessionMetrics::default(),
            context: SessionContext::default(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        Utc::now() - self.start_time
    }

    pub fn update_activity(&mut self) {
        self.last_activity = Utc::now();
    }

    pub fn restore_from_snapshot(&mut self, snapshot: &Snapshot) -> Result<()> {
        self.state = snapshot.state;
        self.topic_stack = snapshot.topic_stack.clone();
        self.context = snapshot.context.clone();
        Ok(())
    }

    pub fn final_snapshot(&self) -> Snapshot {
        Snapshot {
            session_id: self.id,
            timestamp: Utc::now(),
            state: self.state,
            topic_stack: self.topic_stack.clone(),
            context: self.context.clone(),
            elapsed: self.elapsed(),
            metrics: self.metrics.clone(),
        }
    }
}
```

---

## 4. 话题栈管理

### 4.1 话题栈结构

```rust
#[derive(Debug, Clone)]
pub struct TopicStack {
    topics: Vec<Topic>,
    max_depth: usize,
}

#[derive(Debug, Clone)]
pub struct Topic {
    pub id: String,
    pub title: String,
    pub context: String,          // 当前讨论位置
    pub anchor: String,           // 用于恢复的记忆锚点
    pub created_at: DateTime<Utc>,
    pub child_count: u32,         // 子话题数量
}

impl TopicStack {
    pub fn new(max_depth: usize) -> Self {
        Self {
            topics: Vec::new(),
            max_depth,
        }
    }

    /// 进入新话题
    pub fn push(&mut self, topic: Topic) -> Result<()> {
        if self.topics.len() >= self.max_depth {
            anyhow::bail!("Topic stack depth exceeded (max: {})", self.max_depth);
        }

        // 增加父话题的子计数
        if let Some(parent) = self.topics.last_mut() {
            parent.child_count += 1;
        }

        self.topics.push(topic);
        Ok(())
    }

    /// 返回上一话题
    pub fn pop(&mut self) -> Option<Topic> {
        self.topics.pop()
    }

    /// 获取当前话题
    pub fn current(&self) -> Option<&Topic> {
        self.topics.last()
    }

    /// 更新当前话题的上下文
    pub fn update_current_context(&mut self, context: String) {
        if let Some(topic) = self.topics.last_mut() {
            topic.context = context;
        }
    }

    /// 生成恢复提示
    pub fn recovery_prompt(&self) -> String {
        match self.current() {
            Some(topic) => {
                format!(
                    "我们刚才聊到「{}」（锚点），正想请你{}。要继续这个话题吗？",
                    topic.title,
                    topic.context
                )
            }
            None => "准备好继续我们之前的对话吗？".to_string(),
        }
    }

    /// 获取话题路径（用于日志）
    pub fn path(&self) -> String {
        self.topics
            .iter()
            .map(|t| t.title.as_str())
            .collect::<Vec<_>>()
            .join(" → ")
    }
}
```

### 4.2 话题栈使用示例

```rust
// 用户问："什么是装饰器？"
session.topic_stack.push(Topic {
    id: "topic-1".to_string(),
    title: "装饰器概念".to_string(),
    context: "理解装饰器的作用".to_string(),
    anchor: "装饰器是函数包装器".to_string(),
    created_at: Utc::now(),
    child_count: 0,
});

// 用户追问："装饰器可以嵌套吗？"
session.topic_stack.push(Topic {
    id: "topic-2".to_string(),
    title: "装饰器嵌套".to_string(),
    context: "理解嵌套装饰器的执行顺序".to_string(),
    anchor: "装饰器从下往上执行".to_string(),
    created_at: Utc::now(),
    child_count: 0,
});

// 用户说："等等，回到上一个话题"
session.topic_stack.pop();
// 当前话题回到 "装饰器概念"

// 生成恢复提示
let prompt = session.topic_stack.recovery_prompt();
// "我们刚才聊到「装饰器概念」（锚点），正想请你理解装饰器的作用。要继续这个话题吗？"
```

---

## 5. 快照与恢复

### 5.1 快照结构

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub session_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub state: SessionState,
    pub topic_stack: TopicStack,
    pub context: SessionContext,
    pub elapsed: Duration,
    pub metrics: SessionMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    pub user_mood: Option<MoodState>,
    pub fatigue_score: u8,
    pub recent_topics: Vec<String>,
    pub last_agent_response: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MoodState {
    Energetic,
    Focused,
    Tired,
    Distracted,
    Anxious,
}
```

### 5.2 快照存储

```rust
pub struct SnapshotStore {
    db: PgPool,
    redis: redis::Client,
}

impl SnapshotStore {
    /// 保存快照
    pub async fn save(&self, snapshot: Snapshot) -> Result<()> {
        let key = format!("snapshot:{}", snapshot.session_id);

        // 1. 保存到 Redis（快速访问）
        let mut conn = self.redis.get_async_connection().await?;
        conn.set_ex(
            &key,
            serde_json::to_string(&snapshot)?,
            3600  // 1 小时过期
        ).await?;

        // 2. 持久化到 PostgreSQL
        sqlx::query!(
            "INSERT INTO snapshots (session_id, timestamp, data)
             VALUES ($1, $2, $3)",
            snapshot.session_id,
            snapshot.timestamp,
            serde_json::to_value(&snapshot)? as Json<Value>
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// 加载快照
    pub async fn load(&self, session_id: Uuid) -> Result<Option<Snapshot>> {
        let key = format!("snapshot:{}", session_id);

        // 1. 先从 Redis 读取
        let mut conn = self.redis.get_async_connection().await?;
        if let Ok(data) = conn.get::<_, String>(&key).await {
            return Ok(Some(serde_json::from_str(&data)?));
        }

        // 2. Redis 未命中，从 PostgreSQL 读取
        let row = sqlx::query!(
            "SELECT data FROM snapshots
             WHERE session_id = $1
             ORDER BY timestamp DESC
             LIMIT 1",
            session_id
        )
        .fetch_optional(&self.db)
        .await?;

        if let Some(row) = row {
            let snapshot: Snapshot = serde_json::from_value(row.data)?;
            Ok(Some(snapshot))
        } else {
            Ok(None)
        }
    }

    /// 自动清理过期快照
    pub async fn cleanup_expired(&self, older_than: Duration) -> Result<u64> {
        let cutoff = Utc::now() - older_than;

        let result = sqlx::query!(
            "DELETE FROM snapshots WHERE timestamp < $1",
            cutoff
        )
        .execute(&self.db)
        .await?;

        Ok(result.rows_affected())
    }
}
```

### 5.3 自动快照策略

```rust
pub enum SnapshotTrigger {
    /// 每隔固定时间
    TimeInterval(Duration),

    /// 每个语义单元结束后
    SemanticUnitComplete,

    /// 话题切换时
    TopicChange,

    /// 状态转换时
    StateTransition,
}

pub struct SnapshotScheduler {
    triggers: Vec<SnapshotTrigger>,
    last_snapshot: Arc<Mutex<HashMap<Uuid, DateTime<Utc>>>>,
}

impl SnapshotScheduler {
    pub fn should_snapshot(&self, session: &Session, event: &SessionEvent)
        -> bool
    {
        for trigger in &self.triggers {
            match trigger {
                SnapshotTrigger::TimeInterval(interval) => {
                    if let Ok(last) = self.last_snapshot.lock() {
                        if let Some(&last_time) = last.get(&session.id) {
                            if Utc::now() - last_time > *interval {
                                return true;
                            }
                        }
                    }
                }
                SnapshotTrigger::SemanticUnitComplete => {
                    matches!(event, SessionEvent::SemanticUnitComplete)
                }
                SnapshotTrigger::TopicChange => {
                    matches!(event, SessionEvent::TopicChanged { .. })
                }
                SnapshotTrigger::StateTransition => {
                    matches!(event, SessionEvent::StateChanged { .. })
                }
            }
        }
        false
    }
}
```

---

## 6. 事件系统

### 6.1 事件定义

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionEvent {
    /// 文本响应
    TextResponse(String),

    /// TTS 音频数据
    AudioTTS(Vec<u8>),

    /// 触觉反馈
    HapticFeedback(HapticPattern),

    /// 状态变更
    StateChanged {
        old: SessionState,
        new: SessionState,
    },

    /// 话题变更
    TopicChanged {
        from: Option<String>,
        to: String,
    },

    /// 语义单元完成
    SemanticUnitComplete,

    /// 错误
    Error(String),

    /// 会话关闭
    Closing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HapticPattern {
    Short1,           // 肯定
    Short2,           // 纠正
    Long1,            // 话题切换
    LightVibrate,     // 思考中
}
```

### 6.2 事件广播

```rust
impl SessionManager {
    async fn send_event(&self, session_id: Uuid, event: SessionEvent)
        -> Result<()>
    {
        let sessions = self.sessions.read().await;
        if let Some(session) = sessions.get(&session_id) {
            session.event_tx.send(event)
                .map_err(|_| anyhow!("Failed to send event"))?;
        }
        Ok(())
    }
}
```

---

## 7. 会话指标

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionMetrics {
    /// 平均响应时间
    pub avg_response_time: Duration,

    /// 沉默总时长
    pub total_silence: Duration,

    /// 用户发言次数
    pub user_turns: u32,

    /// Agent 发言次数
    pub agent_turns: u32,

    /// 回答质量评分 (0-1)
    pub response_quality: f32,

    /// 话题切换次数
    pub topic_switches: u32,

    /// 中断次数
    pub interruptions: u32,
}

impl SessionMetrics {
    pub fn fatigue_score(&self) -> u8 {
        let mut score = 0u8;

        // 响应时间
        if self.avg_response_time > Duration::from_secs(10) {
            score += 30;
        }

        // 沉默比例
        let silence_ratio = self.total_silence.as_secs_f64()
            / (self.avg_response_time.as_secs_f64() * self.agent_turns as f64);
        if silence_ratio > 0.5 {
            score += 20;
        }

        // 回答质量
        if self.response_quality < 0.5 {
            score += 30;
        }

        // 频繁中断
        if self.interruptions > 5 {
            score += 20;
        }

        score.min(100)
    }
}
```
