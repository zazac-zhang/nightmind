# 内容处理设计

## 1. 概述

内容处理模块负责将原始知识转换为"语音友好"的形式，并控制会话节奏以适配用户的认知负荷。

---

## 2. 内容转换器

### 2.1 核心职责

- 代码/公式 → 自然语言解释
- 技术文档 → 比喻/类比
- 列表/对比 → 分组隐喻
- 长文本 → 分段处理

### 2.2 转换器接口

```rust
pub struct ContentTransformer {
    llm_client: Arc<openai::Client>,
    rules: Vec<TransformRule>,
}

#[async_trait]
pub trait Transformer: Send + Sync {
    async fn transform(&self, content: &str, context: &TransformContext)
        -> Result<TransformedContent>;

    fn can_handle(&self, content: &str) -> bool;
}

pub struct TransformedContent {
    pub text: String,
    pub estimated_duration: Duration,
    pub has_visual_element: bool,
    pub意象描述: Option<String>,
}

pub struct TransformContext {
    pub user_knowledge_level: KnowledgeLevel,
    pub current_cognitive_load: CognitiveLoad,
    pub time_constraint: Option<Duration>,
}

#[derive(Debug, Clone, Copy)]
pub enum KnowledgeLevel {
    Beginner,
    Intermediate,
    Advanced,
}
```

### 2.3 转换规则

```rust
pub enum TransformRule {
    /// 代码转比喻
    CodeToMetaphor,

    /// 公式转解释
    FormulaToExplanation,

    /// 列表转故事
    ListToStory,

    /// 长句分段
    LongSentenceSplit,

    /// 技术术语简化
    TermSimplification,
}

impl ContentTransformer {
    pub async fn apply_rules(
        &self,
        content: &str,
        context: &TransformContext,
    ) -> Result<TransformedContent> {
        let mut transformed = content.to_string();

        // 检测内容类型
        let content_type = self.detect_content_type(content);

        // 应用对应规则
        for rule in &self.rules {
            if rule.is_applicable(content_type) {
                transformed = rule.apply(&transformed, context).await?;
            }
        }

        // 验证结果
        self.validate_output(&transformed)?;

        Ok(TransformedContent {
            text: transformed,
            estimated_duration: self.estimate_duration(&transformed),
            has_visual_element: false,
            意象描述: self.extract_metaphor(&transformed),
        })
    }

    fn detect_content_type(&self, content: &str) -> ContentType {
        if content.contains("```") || content.contains("def ") || content.contains("fn ") {
            ContentType::Code
        } else if content.contains('$') || content.contains("\\frac") {
            ContentType::Formula
        } else if content.lines().filter(|l| l.starts_with("- ") || l.starts_with("* ")).count() > 2 {
            ContentType::List
        } else {
            ContentType::Text
        }
    }

    fn validate_output(&self, content: &str) -> Result<()> {
        // 检查是否符合"语音友好化"标准
        let checklist = vec![
            ("15秒内可读完", self.check_reading_time(content)),
            ("无需视觉", !self.requires_visual(content)),
            ("有意象", self.has_metaphor(content)),
            ("有思考间隙", self.has_pause_point(content)),
        ];

        let failed: Vec<_> = checklist.into_iter()
            .filter(|(_, passed)| !passed)
            .map(|(item, _)| item)
            .collect();

        if !failed.is_empty() {
            tracing::warn!("Content validation failed: {:?}", failed);
        }

        Ok(())
    }

    fn check_reading_time(&self, content: &str) -> bool {
        // 平均语速: 200 字/分钟
        let char_count = content.chars().count();
        char_count < 50  // 约 15 秒
    }

    fn requires_visual(&self, content: &str) -> bool {
        let visual_indicators = [
            "如图", "下图", "上图", "见下图", "参考图",
            "→", "⇒", "图表", "表格",
        ];

        !visual_indicators.iter().any(|&ind| content.contains(ind))
    }

    fn has_metaphor(&self, content: &str) -> bool {
        let metaphor_indicators = [
            "就像", "好比", "如同", "相当于",
            "想象", "比喻成", "可以理解为",
        ];

        metaphor_indicators.iter().any(|&ind| content.contains(ind))
    }

    fn has_pause_point(&self, content: &str) -> bool {
        // 检查是否有提问或省略号
        content.contains('?') || content.contains("...") || content.contains("呢")
    }

    fn estimate_duration(&self, content: &str) -> Duration {
        let char_count = content.chars().count();
        Duration::from_secs((char_count as u64 * 60 / 200).max(1))
    }
}
```

### 2.4 具体规则实现

#### CodeToMetaphor

```rust
pub struct CodeToMetaphorRule {
    llm_client: Arc<openai::Client>,
}

#[async_trait]
impl Transformer for CodeToMetaphorRule {
    async fn transform(&self, content: &str, context: &TransformContext)
        -> Result<TransformedContent>
    {
        let prompt = format!(
            "将以下代码片段转换为适合语音朗读的比喻解释。
            不要朗读代码本身，而是解释其核心思想。
            使用'就像...'的比喻方式。

            代码：
            {}

            要求：
            1. 使用生活化的比喻
            2. 15秒内可读完
            3. 保留核心逻辑
            ",
            content
        );

        let response = self.llm_client
            .completion_model("gpt-4")
            .prompt(&prompt)
            .await?;

        Ok(TransformedContent {
            text: response,
            estimated_duration: Duration::from_secs(15),
            has_visual_element: false,
            意象描述: None,
        })
    }

    fn can_handle(&self, content: &str) -> bool {
        content.contains("```")
            || content.contains("def ")
            || content.contains("fn ")
            || content.contains("class ")
    }
}
```

#### ListToStory

```rust
pub struct ListToStoryRule {
    llm_client: Arc<openai::Client>,
}

#[async_trait]
impl Transformer for ListToStoryRule {
    async fn transform(&self, content: &str, context: &TransformContext)
        -> Result<TransformedContent>
    {
        let items: Vec<_> = content
            .lines()
            .filter_map(|line| {
                let cleaned = line.trim_start_matches("- ")
                    .trim_start_matches("* ")
                    .trim_start_matches(&['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'][..])
                    .trim_start_matches(". ")
                    .trim();
                if !cleaned.is_empty() {
                    Some(cleaned.to_string())
                } else {
                    None
                }
            })
            .collect();

        let prompt = format!(
            "将以下要点转换为一个连贯的故事或类比，便于语音记忆。

            要点：
            {}

            要求：
            1. 使用一个统一的比喻（如'种子生长''建造房子'等）
            2. 将要点自然串联
            3. 30秒内可读完
            ",
            items.join("\n")
        );

        let response = self.llm_client
            .completion_model("gpt-4")
            .prompt(&prompt)
            .await?;

        Ok(TransformedContent {
            text: response,
            estimated_duration: Duration::from_secs(30),
            has_visual_element: false,
            意象描述: Some("故事化记忆".to_string()),
        })
    }

    fn can_handle(&self, content: &str) -> bool {
        let list_markers = ["- ", "* ", "1. ", "2. ", "3. "];
        let line_count = content.lines().count();
        let has_markers = list_markers.iter().any(|&m| content.contains(m));

        has_markers && line_count >= 3
    }
}
```

---

## 3. 节奏控制器

### 3.1 核心职责

- 阶段时间管理
- 认知负荷自适应
- 疲劳度检测
- 内容密度调整

### 3.2 控制器接口

```rust
pub struct RhythmController {
    phase_configs: HashMap<SessionState, PhaseConfig>,
    fatigue_threshold: u8,
    adaptive_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct PhaseConfig {
    pub duration: Duration,
    pub cognitive_load: CognitiveLoad,
    pub content_density: DensityLevel,
    pub interaction_frequency: InteractionFrequency,
    pub transition_prompt: String,
}

#[derive(Debug, Clone, Copy)]
pub enum DensityLevel {
    Minimal,    // 仅核心结论
    Low,        // 简要说明
    Medium,     // 标准解释
    High,       // 深度探讨
}

#[derive(Debug, Clone, Copy)]
pub enum InteractionFrequency {
    Sparse,     // 2-3 分钟一次
    Normal,     // 1-2 分钟一次
    Frequent,   // 30-60 秒一次
}
```

### 3.3 阶段配置

```rust
impl Default for RhythmController {
    fn default() -> Self {
        let mut phase_configs = HashMap::new();

        phase_configs.insert(
            SessionState::Warmup,
            PhaseConfig {
                duration: Duration::from_secs(300),
                cognitive_load: CognitiveLoad::Low,
                content_density: DensityLevel::Low,
                interaction_frequency: InteractionFrequency::Normal,
                transition_prompt: "今天感觉怎么样？有什么特别想聊的吗？".to_string(),
            },
        );

        phase_configs.insert(
            SessionState::DeepDive,
            PhaseConfig {
                duration: Duration::from_secs(1200),
                cognitive_load: CognitiveLoad::High,
                content_density: DensityLevel::High,
                interaction_frequency: InteractionFrequency::Frequent,
                transition_prompt: "让我们深入一点...".to_string(),
            },
        );

        phase_configs.insert(
            SessionState::Review,
            PhaseConfig {
                duration: Duration::from_secs(1200),
                cognitive_load: CognitiveLoad::Medium,
                content_density: DensityLevel::Medium,
                interaction_frequency: InteractionFrequency::Normal,
                transition_prompt: "我们来回顾一下之前学过的...".to_string(),
            },
        );

        phase_configs.insert(
            SessionState::Seed,
            PhaseConfig {
                duration: Duration::from_secs(600),
                cognitive_load: CognitiveLoad::Low,
                content_density: DensityLevel::Minimal,
                interaction_frequency: InteractionFrequency::Sparse,
                transition_prompt: "最后，留给你一个思考...".to_string(),
            },
        );

        phase_configs.insert(
            SessionState::Closing,
            PhaseConfig {
                duration: Duration::from_secs(300),
                cognitive_load: CognitiveLoad::VeryLow,
                content_density: DensityLevel::Minimal,
                interaction_frequency: InteractionFrequency::Sparse,
                transition_prompt: "今晚就到这里吧，晚安...".to_string(),
            },
        );

        Self {
            phase_configs,
            fatigue_threshold: 70,
            adaptive_enabled: true,
        }
    }
}
```

### 3.4 节奏控制逻辑

```rust
impl RhythmController {
    /// 获取当前阶段配置
    pub fn get_phase_config(&self, state: SessionState) -> &PhaseConfig {
        self.phase_configs.get(&state)
            .unwrap_or_else(|| self.phase_configs.get(&SessionState::Warmup).unwrap())
    }

    /// 检查是否应该转换阶段
    pub fn should_transition(
        &self,
        session: &Session,
        metrics: &SessionMetrics,
    ) -> Option<SessionState> {
        let current_config = self.get_phase_config(session.state);
        let elapsed = session.elapsed();

        // 时间检查
        if elapsed >= current_config.duration {
            return Some(session.state.next().unwrap_or(SessionState::Closed));
        }

        // 疲劳度检查（自适应）
        if self.adaptive_enabled {
            let fatigue_score = metrics.fatigue_score();
            if fatigue_score > self.fatigue_threshold {
                tracing::info!(
                    "Fatigue threshold exceeded: {}/{}",
                    fatigue_score, self.fatigue_threshold
                );
                return Some(SessionState::Closing);
            }
        }

        None
    }

    /// 根据疲劳度调整内容密度
    pub fn adjust_density(
        &self,
        base_density: DensityLevel,
        fatigue_score: u8,
    ) -> DensityLevel {
        match (base_density, fatigue_score) {
            (DensityLevel::High, score) if score > 60 => DensityLevel::Medium,
            (DensityLevel::Medium, score) if score > 70 => DensityLevel::Low,
            (DensityLevel::Low, score) if score > 80 => DensityLevel::Minimal,
            _ => base_density,
        }
    }

    /// 生成阶段转换提示
    pub fn transition_prompt(&self, from: SessionState, to: SessionState)
        -> Option<String>
    {
        self.phase_configs.get(&to)
            .map(|config| config.transition_prompt.clone())
    }

    /// 计算下次交互时间
    pub fn next_interaction_delay(&self, state: SessionState)
        -> Duration
    {
        let config = self.get_phase_config(state);
        match config.interaction_frequency {
            InteractionFrequency::Sparse => Duration::from_secs(120),
            InteractionFrequency::Normal => Duration::from_secs(60),
            InteractionFrequency::Frequent => Duration::from_secs(30),
        }
    }
}
```

---

## 4. 疲劳检测器

### 4.1 检测维度

```rust
pub struct FatigueDetector {
    thresholds: FatigueThresholds,
}

#[derive(Debug, Clone)]
pub struct FatigueThresholds {
    pub response_time: Duration,
    pub silence_ratio: f32,
    pub response_quality_min: f32,
    pub session_max_duration: Duration,
}

#[derive(Debug, Clone)]
pub struct FatigueSignals {
    pub response_time_slow: bool,
    pub silence_excessive: bool,
    pub quality_declined: bool,
    pub session_too_long: bool,
    pub frequent_interruptions: bool,
}

impl FatigueDetector {
    pub fn detect(&self, metrics: &SessionMetrics) -> (u8, FatigueSignals) {
        let mut score = 0u8;
        let mut signals = FatigueSignals {
            response_time_slow: false,
            silence_excessive: false,
            quality_declined: false,
            session_too_long: false,
            frequent_interruptions: false,
        };

        // 响应时间
        if metrics.avg_response_time > self.thresholds.response_time {
            score += 25;
            signals.response_time_slow = true;
        }

        // 沉默比例
        let silence_ratio = metrics.total_silence.as_secs_f64()
            / (metrics.avg_response_time.as_secs_f64() * metrics.agent_turns as f64);
        if silence_ratio > self.thresholds.silence_ratio {
            score += 20;
            signals.silence_excessive = true;
        }

        // 回答质量
        if metrics.response_quality < self.thresholds.response_quality_min {
            score += 25;
            signals.quality_declined = true;
        }

        // 会话时长
        if metrics.total_duration > self.thresholds.session_max_duration {
            score += 15;
            signals.session_too_long = true;
        }

        // 频繁中断
        if metrics.interruptions > 5 {
            score += 15;
            signals.frequent_interruptions = true;
        }

        (score.min(100), signals)
    }

    pub fn get_suggestion(&self, signals: &FatigueSignals) -> String {
        match signals {
            FatigueSignals {
                response_time_slow: true,
                silence_excessive: true,
                ..
            } => "我感觉到你可能有点累了，要不我们今天就到这里？".to_string(),

            FatigueSignals {
                quality_declined: true,
                ..
            } => "我们休息一下，换个轻松的话题？".to_string(),

            FatigueSignals {
                session_too_long: true,
                ..
            } => "时间不早了，我们来做最后的总结吧。".to_string(),

            _ => "还好吗？".to_string(),
        }
    }
}
```

---

## 5. 内容质量评估

### 5.1 评估维度

```rust
pub struct ContentEvaluator;

impl ContentEvaluator {
    pub fn evaluate(&self, content: &TransformedContent) -> QualityScore {
        let mut score = QualityScore::default();

        // 长度适中 (10-50字)
        let len = content.text.chars().count();
        score.length_score = if len >= 10 && len <= 50 {
            1.0
        } else {
            1.0 - (len as f32 - 30.0).abs() / 30.0
        };

        // 无视觉依赖
        score.visual_score = if !content.has_visual_element { 1.0 } else { 0.0 };

        // 有意象
        score.metaphor_score = if content.意象描述.is_some() { 1.0 } else { 0.5 };

        // 时长合理 (5-30秒)
        let secs = content.estimated_duration.as_secs();
        score.duration_score = if secs >= 5 && secs <= 30 {
            1.0
        } else {
            1.0 - (secs as f32 - 15.0).abs() / 15.0
        };

        score
    }
}

#[derive(Debug, Clone, Default)]
pub struct QualityScore {
    pub length_score: f32,
    pub visual_score: f32,
    pub metaphor_score: f32,
    pub duration_score: f32,
}

impl QualityScore {
    pub fn overall(&self) -> f32 {
        (self.length_score * 0.2
            + self.visual_score * 0.3
            + self.metaphor_score * 0.3
            + self.duration_score * 0.2)
    }
}
```
