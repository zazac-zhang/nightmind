// ============================================================
// Knowledge Point Model
// ============================================================
//! Knowledge point entity model.
//!
//! This module defines the knowledge point structure based on the
//! knowledge_points database table.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Knowledge point entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgePoint {
    /// Unique knowledge point identifier
    pub id: Uuid,

    /// Associated user ID
    pub user_id: Uuid,

    /// Knowledge content
    pub content: String,

    /// Content type (concept, fact, procedure, story)
    pub content_type: String,

    /// Source type (anki, obsidian, notion, readwise, manual)
    pub source_type: String,

    /// Original source ID
    pub source_id: Option<String>,

    /// Knowledge title
    pub title: Option<String>,

    /// Content summary
    pub summary: Option<String>,

    /// Tags for categorization
    pub tags: Vec<String>,

    /// Parent knowledge point ID
    pub parent_id: Option<Uuid>,

    /// Related knowledge point IDs
    pub related_ids: Vec<Uuid>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Last reviewed timestamp
    pub last_reviewed_at: Option<DateTime<Utc>>,
}

/// Knowledge point content type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum KnowledgeContentType {
    /// Abstract concept or principle
    Concept,

    /// Factual information
    Fact,

    /// Step-by-step procedure
    Procedure,

    /// Narrative or story-based content
    Story,
}

impl KnowledgeContentType {
    /// Converts from string to KnowledgeContentType
    #[must_use]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "concept" => Some(KnowledgeContentType::Concept),
            "fact" => Some(KnowledgeContentType::Fact),
            "procedure" => Some(KnowledgeContentType::Procedure),
            "story" => Some(KnowledgeContentType::Story),
            _ => None,
        }
    }

    /// Converts to string
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            KnowledgeContentType::Concept => "concept",
            KnowledgeContentType::Fact => "fact",
            KnowledgeContentType::Procedure => "procedure",
            KnowledgeContentType::Story => "story",
        }
    }
}

/// Knowledge point source type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum KnowledgeSourceType {
    /// From Anki spaced repetition system
    Anki,

    /// From Obsidian markdown notes
    Obsidian,

    /// From Notion database
    Notion,

    /// From Readwise highlights
    Readwise,

    /// Manually created
    Manual,
}

impl KnowledgeSourceType {
    /// Converts from string to KnowledgeSourceType
    #[must_use]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "anki" => Some(KnowledgeSourceType::Anki),
            "obsidian" => Some(KnowledgeSourceType::Obsidian),
            "notion" => Some(KnowledgeSourceType::Notion),
            "readwise" => Some(KnowledgeSourceType::Readwise),
            "manual" => Some(KnowledgeSourceType::Manual),
            _ => None,
        }
    }

    /// Converts to string
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            KnowledgeSourceType::Anki => "anki",
            KnowledgeSourceType::Obsidian => "obsidian",
            KnowledgeSourceType::Notion => "notion",
            KnowledgeSourceType::Readwise => "readwise",
            KnowledgeSourceType::Manual => "manual",
        }
    }
}

/// Create knowledge point request
#[derive(Debug, Clone)]
pub struct CreateKnowledgePoint {
    /// Knowledge content
    pub content: String,

    /// Content type
    pub content_type: String,

    /// Source type
    pub source_type: String,

    /// User ID creating the knowledge
    pub user_id: Uuid,

    /// Optional title
    pub title: Option<String>,

    /// Optional summary
    pub summary: Option<String>,

    /// Optional tags
    pub tags: Vec<String>,

    /// Optional source ID
    pub source_id: Option<String>,

    /// Optional parent ID
    pub parent_id: Option<Uuid>,
}

/// Update knowledge point request
#[derive(Debug, Clone)]
pub struct UpdateKnowledgePoint {
    /// Knowledge point ID to update
    pub id: Uuid,

    /// New content (optional)
    pub content: Option<String>,

    /// New content type (optional)
    pub content_type: Option<String>,

    /// New title (optional)
    pub title: Option<String>,

    /// New summary (optional)
    pub summary: Option<String>,

    /// New tags (optional)
    pub tags: Option<Vec<String>>,

    /// New related IDs (optional)
    pub related_ids: Option<Vec<Uuid>>,
}

impl KnowledgePoint {
    /// Calculates complexity score based on content length and type
    #[must_use]
    pub fn complexity_score(&self) -> f32 {
        // Base complexity from content length (0-1 scale)
        let char_count = self.content.chars().count() as f32;
        let length_complexity = (char_count / 1000.0).min(1.0) * 0.5;

        // Content type multiplier
        let type_multiplier = match self.content_type.as_str() {
            "concept" => 0.7,
            "procedure" => 0.9,
            "fact" => 0.3,
            "story" => 0.5,
            _ => 0.5,
        };

        length_complexity * type_multiplier
    }

    /// Gets the knowledge point content length
    #[must_use]
    pub fn content_length(&self) -> usize {
        self.content.chars().count()
    }

    /// Checks if the knowledge point is due for review
    #[must_use]
    pub fn is_due_for_review(&self, next_review_date: Option<chrono::NaiveDate>) -> bool {
        match next_review_date {
            Some(date) => date <= chrono::Utc::now().date_naive(),
            None => true, // No review scheduled, should review
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_content_type_conversion() {
        assert_eq!(
            KnowledgeContentType::from_str("concept"),
            Some(KnowledgeContentType::Concept)
        );
        assert_eq!(
            KnowledgeContentType::from_str("Concept"),
            Some(KnowledgeContentType::Concept)
        );
        assert_eq!(KnowledgeContentType::from_str("invalid"), None);
    }

    #[test]
    fn test_knowledge_source_type_conversion() {
        assert_eq!(
            KnowledgeSourceType::from_str("anki"),
            Some(KnowledgeSourceType::Anki)
        );
        assert_eq!(
            KnowledgeSourceType::from_str("Anki"),
            Some(KnowledgeSourceType::Anki)
        );
        assert_eq!(KnowledgeSourceType::from_str("invalid"), None);
    }

    #[test]
    fn test_complexity_score() {
        let point = KnowledgePoint {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            content: "A".repeat(500),
            content_type: "concept".to_string(),
            source_type: "manual".to_string(),
            source_id: None,
            title: Some("Test".to_string()),
            summary: None,
            tags: vec![],
            parent_id: None,
            related_ids: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_reviewed_at: None,
        };

        let score = point.complexity_score();
        assert!(score > 0.0 && score <= 1.0);
    }
}
