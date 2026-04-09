// ============================================================
// Knowledge Extraction Pipeline
// ============================================================
//! Extracts knowledge points from various external sources.
//!
//! This module provides the pipeline for importing and processing
//! learning content from Anki, Obsidian, Notion, and manual input.

use crate::repository::models::knowledge::{
    KnowledgeContentType, KnowledgePoint, KnowledgeSourceType, UpdateKnowledgePoint,
    CreateKnowledgePoint,
};
use crate::repository::knowledge::KnowledgeRepository;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Knowledge extractor for importing learning content
pub struct KnowledgeExtractor {
    /// Knowledge repository
    repository: Arc<dyn KnowledgeRepository>,
}

impl KnowledgeExtractor {
    /// Creates a new knowledge extractor
    #[must_use]
    pub fn new(repository: Arc<dyn KnowledgeRepository>) -> Self {
        Self { repository }
    }

    /// Extracts knowledge from Anki cards
    ///
    /// # Arguments
    ///
    /// * `user_id` - User ID
    /// * `cards` - Anki cards to extract
    ///
    /// # Errors
    ///
    /// Returns an error if extraction fails
    pub async fn extract_from_anki(
        &self,
        user_id: Uuid,
        cards: Vec<AnkiCard>,
    ) -> Result<Vec<KnowledgePoint>, ExtractionError> {
        let mut knowledge_points = Vec::new();

        for card in cards {
            // Parse content type from card tags
            let content_type = self.classify_content(&card.front, &card.tags);

            let request = CreateKnowledgePoint {
                content: format!("Q: {}\n\nA: {}", card.front, card.back),
                content_type: content_type.as_str().to_string(),
                source_type: KnowledgeSourceType::Anki.as_str().to_string(),
                user_id,
                title: Some(card.front.chars().take(100).collect::<String>()),
                summary: Some(card.back.clone()),
                tags: card.tags.clone(),
                source_id: Some(card.anki_card_id.to_string()),
                parent_id: None,
            };

            let point = self
                .repository
                .create(request)
                .await
                .map_err(|e| ExtractionError::RepositoryError(e.to_string()))?;

            knowledge_points.push(point);
        }

        Ok(knowledge_points)
    }

    /// Extracts knowledge from Obsidian markdown notes
    ///
    /// # Arguments
    ///
    /// * `user_id` - User ID
    /// * `notes` - Obsidian notes to extract
    ///
    /// # Errors
    ///
    /// Returns an error if extraction fails
    pub async fn extract_from_obsidian(
        &self,
        user_id: Uuid,
        notes: Vec<ObsidianNote>,
    ) -> Result<Vec<KnowledgePoint>, ExtractionError> {
        let mut knowledge_points = Vec::new();

        for note in notes {
            // Extract content type from frontmatter or content
            let content_type = self.classify_content(&note.content, &note.tags);

            // Extract title from filename or first heading
            let title = note
                .frontmatter
                .as_ref()
                .and_then(|f| f.title.as_ref())
                .cloned()
                .unwrap_or_else(|| note.filename.clone());

            let request = CreateKnowledgePoint {
                content: note.content.clone(),
                content_type: content_type.as_str().to_string(),
                source_type: KnowledgeSourceType::Obsidian.as_str().to_string(),
                user_id,
                title: Some(title),
                summary: note.frontmatter.as_ref().and_then(|f| f.description.clone()),
                tags: note.tags.clone(),
                source_id: Some(note.filepath.clone()),
                parent_id: None,
            };

            let point = self
                .repository
                .create(request)
                .await
                .map_err(|e| ExtractionError::RepositoryError(e.to_string()))?;

            knowledge_points.push(point);
        }

        Ok(knowledge_points)
    }

    /// Extracts knowledge from manual input
    ///
    /// # Arguments
    ///
    /// * `user_id` - User ID
    /// * `input` - Manual knowledge input
    ///
    /// # Errors
    ///
    /// Returns an error if extraction fails
    pub async fn extract_from_manual(
        &self,
        user_id: Uuid,
        input: ManualKnowledgeInput,
    ) -> Result<KnowledgePoint, ExtractionError> {
        let tags = input.tags.clone().unwrap_or_default();
        let tags_ref = tags.as_slice();

        let content_type = self
            .classify_content(&input.content, tags_ref)
            .as_str()
            .to_string();

        let request = CreateKnowledgePoint {
            content: input.content.clone(),
            content_type,
            source_type: KnowledgeSourceType::Manual.as_str().to_string(),
            user_id,
            title: input.title.clone(),
            summary: input.summary.clone(),
            tags,
            source_id: None,
            parent_id: input.parent_id,
        };

        let point = self
            .repository
            .create(request)
            .await
            .map_err(|e| ExtractionError::RepositoryError(e.to_string()))?;

        Ok(point)
    }

    /// Batch extracts knowledge from multiple sources
    ///
    /// # Arguments
    ///
    /// * `user_id` - User ID
    /// * `sources` - Multiple knowledge sources
    ///
    /// # Errors
    ///
    /// Returns an error if extraction fails
    pub async fn extract_batch(
        &self,
        user_id: Uuid,
        sources: Vec<KnowledgeSource>,
    ) -> Result<Vec<KnowledgePoint>, ExtractionError> {
        let mut all_points = Vec::new();

        for source in sources {
            let points = match source {
                KnowledgeSource::Anki(cards) => {
                    self.extract_from_anki(user_id, cards).await?
                },
                KnowledgeSource::Obsidian(notes) => {
                    self.extract_from_obsidian(user_id, notes).await?
                },
                KnowledgeSource::Manual(input) => {
                    vec![self.extract_from_manual(user_id, input).await?]
                },
            };

            all_points.extend(points);
        }

        Ok(all_points)
    }

    /// Classifies content type based on content and tags
    fn classify_content(&self, content: &str, tags: &[String]) -> KnowledgeContentType {
        let content_lower = content.to_lowercase();

        // Check tags first (more reliable)
        for tag in tags {
            match tag.to_lowercase().as_str() {
                "concept" | "theory" | "principle" => {
                    return KnowledgeContentType::Concept;
                },
                "fact" | "data" | "definition" => return KnowledgeContentType::Fact,
                "procedure" | "how-to" | "tutorial" | "steps" => {
                    return KnowledgeContentType::Procedure;
                },
                "story" | "example" | "narrative" => return KnowledgeContentType::Story,
                _ => {},
            }
        }

        // Analyze content structure
        if content_lower.contains("step") || content_lower.contains("how to") {
            KnowledgeContentType::Procedure
        } else if content_lower.contains("example") || content_lower.contains("story") {
            KnowledgeContentType::Story
        } else if content_lower.contains("definition") || content_lower.contains("means that") {
            KnowledgeContentType::Concept
        } else if content_lower.contains("is a") || content_lower.contains("refers to") {
            KnowledgeContentType::Fact
        } else {
            // Default to concept for unknown content
            KnowledgeContentType::Concept
        }
    }
}

/// Anki card data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnkiCard {
    /// Anki card ID
    pub anki_card_id: i64,

    /// Card front (question)
    pub front: String,

    /// Card back (answer)
    pub back: String,

    /// Card tags
    pub tags: Vec<String>,

    /// Anki deck name
    pub deck: String,

    /// Note ID
    pub note_id: Option<i64>,
}

/// Obsidian note frontmatter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsidianFrontmatter {
    /// Note title
    pub title: Option<String>,

    /// Note description
    pub description: Option<String>,

    /// Tags
    pub tags: Option<Vec<String>>,

    /// Custom properties
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

/// Obsidian note data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObsidianNote {
    /// File path
    pub filepath: String,

    /// File name
    pub filename: String,

    /// Note content (markdown)
    pub content: String,

    /// Frontmatter metadata
    pub frontmatter: Option<ObsidianFrontmatter>,

    /// Extracted tags
    pub tags: Vec<String>,

    /// Note created/modified date
    pub modified: Option<chrono::DateTime<chrono::Utc>>,
}

/// Manual knowledge input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualKnowledgeInput {
    /// Knowledge content
    pub content: String,

    /// Optional title
    pub title: Option<String>,

    /// Optional summary
    pub summary: Option<String>,

    /// Optional tags
    pub tags: Option<Vec<String>>,

    /// Optional parent knowledge point
    pub parent_id: Option<Uuid>,
}

/// Knowledge source for batch extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KnowledgeSource {
    /// Anki cards
    Anki(Vec<AnkiCard>),

    /// Obsidian notes
    Obsidian(Vec<ObsidianNote>),

    /// Manual input
    Manual(ManualKnowledgeInput),
}

/// Extraction errors
#[derive(Debug, thiserror::Error)]
pub enum ExtractionError {
    /// Repository error
    #[error("Repository error: {0}")]
    RepositoryError(String),

    /// Parse error
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),
}

/// Daily learning summary for knowledge extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyLearningSummary {
    /// User ID
    pub user_id: Uuid,

    /// Date of learning
    pub date: chrono::NaiveDate,

    /// Topics learned
    pub topics: Vec<String>,

    /// Anki cards reviewed
    pub anki_cards_reviewed: i32,

    /// Notes created
    pub notes_created: i32,

    /// Learning session duration (minutes)
    pub session_duration_minutes: i32,

    /// Overall comprehension (0.0-1.0)
    pub comprehension_score: f32,
}

impl DailyLearningSummary {
    /// Creates a new daily learning summary
    #[must_use]
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            date: Utc::now().date_naive(),
            topics: Vec::new(),
            anki_cards_reviewed: 0,
            notes_created: 0,
            session_duration_minutes: 0,
            comprehension_score: 0.5,
        }
    }

    /// Extracts knowledge points from the summary
    ///
    /// # Errors
    ///
    /// Returns an error if extraction fails
    pub async fn extract_knowledge(
        &self,
        extractor: &KnowledgeExtractor,
    ) -> Result<Vec<KnowledgePoint>, ExtractionError> {
        let mut all_points = Vec::new();

        // Create a summary knowledge point
        let summary_content = format!(
            "今日学习内容：\n\
             主题：{}\n\
             Anki复习：{}张\n\
             新建笔记：{}篇\n\
             学习时长：{}分钟\n\
             理解程度：{:.0}%",
            self.topics.join(", "),
            self.anki_cards_reviewed,
            self.notes_created,
            self.session_duration_minutes,
            self.comprehension_score * 100.0
        );

        let manual_input = ManualKnowledgeInput {
            content: summary_content,
            title: Some(format!("学习总结 - {}", self.date)),
            summary: Some(format!(
                "复习{}张卡片，创建{}篇笔记",
                self.anki_cards_reviewed, self.notes_created
            )),
            tags: vec!["daily-summary".to_string(), "review".to_string()],
            parent_id: None,
        };

        let summary_point = extractor.extract_from_manual(self.user_id, manual_input).await?;
        all_points.push(summary_point);

        Ok(all_points)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anki_card_creation() {
        let card = AnkiCard {
            anki_card_id: 12345,
            front: "What is Rust ownership?".to_string(),
            back: "Ownership is Rust's memory management system".to_string(),
            tags: vec!["rust".to_string(), "concept".to_string()],
            deck: "Programming".to_string(),
            note_id: Some(999),
        };

        assert_eq!(card.anki_card_id, 12345);
        assert_eq!(card.tags.len(), 2);
    }

    #[test]
    fn test_obsidian_note_creation() {
        let note = ObsidianNote {
            filepath: "/notes/rust.md".to_string(),
            filename: "rust.md".to_string(),
            content: "# Rust Ownership\n\n...".to_string(),
            frontmatter: Some(ObsidianFrontmatter {
                title: Some("Rust Ownership".to_string()),
                description: Some("Memory management in Rust".to_string()),
                tags: Some(vec!["rust".to_string()]),
                extra: serde_json::json!({}),
            }),
            tags: vec!["rust".to_string()],
            modified: Some(Utc::now()),
        };

        assert_eq!(note.filename, "rust.md");
        assert!(note.frontmatter.is_some());
    }

    #[test]
    fn test_manual_knowledge_input() {
        let input = ManualKnowledgeInput {
            content: "Test content".to_string(),
            title: Some("Test Title".to_string()),
            summary: None,
            tags: Some(vec!["test".to_string()]),
            parent_id: None,
        };

        assert_eq!(input.content, "Test content");
        assert_eq!(input.title, Some("Test Title".to_string()));
    }

    #[test]
    fn test_daily_learning_summary() {
        let summary = DailyLearningSummary::new(Uuid::new_v4());

        assert_eq!(summary.anki_cards_reviewed, 0);
        assert_eq!(summary.notes_created, 0);
        assert_eq!(summary.date, Utc::now().date_naive());
    }

    #[test]
    fn test_knowledge_source_classification() {
        let repository = std::sync::Arc::new(MockKnowledgeRepository::new());
        let extractor = KnowledgeExtractor::new(repository);

        // Test concept classification
        let content_type =
            extractor.classify_content("This is a definition of ownership", &[]);
        assert_eq!(content_type, KnowledgeContentType::Fact);

        // Test procedure classification
        let content_type = extractor.classify_content("Step 1: Do this. Step 2: Do that.", &[]);
        assert_eq!(content_type, KnowledgeContentType::Procedure);
    }

    // Mock repository for testing
    struct MockKnowledgeRepository;

    impl MockKnowledgeRepository {
        fn new() -> Self {
            Self
        }
    }

    #[async_trait::async_trait]
    impl KnowledgeRepository for MockKnowledgeRepository {
        async fn create(
            &self,
            request: CreateKnowledgePoint,
        ) -> Result<KnowledgePoint, crate::repository::knowledge::RepositoryError> {
            Ok(KnowledgePoint {
                id: Uuid::new_v4(),
                user_id: request.user_id,
                content: request.content,
                content_type: request.content_type,
                source_type: request.source_type,
                source_id: request.source_id,
                title: request.title,
                summary: request.summary,
                tags: request.tags,
                parent_id: request.parent_id,
                related_ids: vec![],
                created_at: Utc::now(),
                updated_at: Utc::now(),
                last_reviewed_at: None,
            })
        }

        async fn get_by_id(
            &self,
            _id: Uuid,
        ) -> Result<KnowledgePoint, crate::repository::knowledge::RepositoryError> {
            Err(crate::repository::knowledge::RepositoryError::NotFound(
                "not implemented".to_string(),
            ))
        }

        async fn update(
            &self,
            _request: UpdateKnowledgePoint,
        ) -> Result<KnowledgePoint, crate::repository::knowledge::RepositoryError> {
            Err(crate::repository::knowledge::RepositoryError::NotFound(
                "not implemented".to_string(),
            ))
        }

        async fn delete(&self, _id: Uuid) -> Result<(), crate::repository::knowledge::RepositoryError> {
            Ok(())
        }

        async fn list_by_user(
            &self,
            _user_id: Uuid,
            _page: u32,
            _limit: u32,
        ) -> Result<Vec<KnowledgePoint>, crate::repository::knowledge::RepositoryError> {
            Ok(vec![])
        }

        async fn search(
            &self,
            _query: crate::repository::knowledge::KnowledgeSearchQuery,
        ) -> Result<Vec<KnowledgePoint>, crate::repository::knowledge::RepositoryError> {
            Ok(vec![])
        }

        async fn update_embedding(
            &self,
            _id: Uuid,
            _embedding: Vec<f32>,
        ) -> Result<KnowledgePoint, crate::repository::knowledge::RepositoryError> {
            Err(crate::repository::knowledge::RepositoryError::NotFound(
                "not implemented".to_string(),
            ))
        }
    }
}
