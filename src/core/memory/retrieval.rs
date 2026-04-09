// ============================================================
// Knowledge Retrieval System
// ============================================================
//! Context-aware knowledge retrieval with multi-dimensional reranking.
//!
//! This module implements the RAG (Retrieval-Augmented Generation) system
//! for intelligent knowledge retrieval during evening sessions.

use crate::core::memory::EveningFSRS;
use crate::repository::models::knowledge::KnowledgePoint;
use crate::repository::models::SessionState;
use crate::repository::ReviewIntervalRepository;
use crate::services::vector::{VectorSearchResult, VectorService};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Knowledge retriever for RAG
pub struct KnowledgeRetriever {
    /// Vector service for semantic search
    vector_service: Arc<VectorService>,
    /// Review interval repository
    review_repository: Arc<dyn ReviewIntervalRepository>,
    /// FSRS algorithm instance
    fsrs: EveningFSRS,
}

impl KnowledgeRetriever {
    /// Creates a new knowledge retriever
    #[must_use]
    pub fn new(
        vector_service: Arc<VectorService>,
        review_repository: Arc<dyn ReviewIntervalRepository>,
    ) -> Self {
        Self {
            vector_service,
            review_repository,
            fsrs: EveningFSRS::new(),
        }
    }

    /// Creates a new retriever with custom FSRS parameters
    #[must_use]
    pub fn with_fsrs(
        vector_service: Arc<VectorService>,
        review_repository: Arc<dyn ReviewIntervalRepository>,
        fsrs: EveningFSRS,
    ) -> Self {
        Self {
            vector_service,
            review_repository,
            fsrs,
        }
    }

    /// Retrieves knowledge for context generation
    ///
    /// # Arguments
    ///
    /// * `user_id` - User ID
    /// * `current_message` - Current conversation message
    /// * `cognitive_load` - Current cognitive load (0.0-1.0)
    /// * `session_state` - Current session state
    /// * `limit` - Maximum results to return
    ///
    /// # Errors
    ///
    /// Returns an error if retrieval fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use nightmind::core::memory::retrieval::KnowledgeRetriever;
    /// # async fn example(retriever: &KnowledgeRetriever) -> Result<(), Box<dyn std::error::Error>> {
    /// let context = retriever.retrieve_for_context(
    ///     user_id,
    ///     "What did I learn about Rust yesterday?",
    ///     0.5,
    ///     SessionState::Review,
    ///     5,
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn retrieve_for_context(
        &self,
        user_id: Uuid,
        _current_message: &str,
        cognitive_load: f32,
        _session_state: crate::repository::models::SessionState,
        limit: u64,
    ) -> Result<RetrievalContext, RetrievalError> {
        // Determine retrieval count based on cognitive load
        let retrieval_limit = match cognitive_load {
            load if load > 0.7 => 3,  // High fatigue: fewer results
            load if load < 0.3 => 7,  // Low fatigue: more results
            _ => limit,               // Normal: use requested limit
        };

        // TODO: Generate embedding from current_message
        // For now, use a dummy vector
        let dummy_embedding = vec![0.0f32; 1536];

        // Search vector database
        let search_results = self
            .vector_service
            .search(dummy_embedding, retrieval_limit, 0.5, Some(user_id))
            .await
            .map_err(|e| RetrievalError::VectorError(e.to_string()))?;

        // Rerank results based on multiple factors
        let reranked = self.rerank_by_context(search_results, cognitive_load).await;

        // Build context
        let context = self.build_context(&reranked, cognitive_load)?;

        let total_count = reranked.len();

        Ok(RetrievalContext {
            context_text: context,
            knowledge_points: reranked,
            total_count,
            retrieved_at: Utc::now(),
        })
    }

    /// Retrieves due reviews for a user
    ///
    /// # Errors
    ///
    /// Returns an error if retrieval fails
    pub async fn retrieve_due_reviews(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<ReviewItem>, RetrievalError> {
        // Get review intervals from repository
        let intervals = self
            .review_repository
            .list_due_reviews(user_id)
            .await
            .map_err(|e| RetrievalError::DatabaseError(e.to_string()))?;

        // Convert to review items
        let items = intervals
            .into_iter()
            .map(|interval| ReviewItem {
                knowledge_point_id: interval.knowledge_point_id,
                interval_days: interval.interval_days,
                ease_factor: interval.ease_factor,
                total_reviews: interval.total_reviews,
                accuracy_rate: interval.accuracy_rate(),
                days_overdue: interval.days_overdue(),
            })
            .collect();

        Ok(items)
    }

    /// Reranks search results using multi-dimensional scoring
    ///
    /// # Arguments
    ///
    /// * `results` - Initial search results from vector DB
    /// * `cognitive_load` - Current cognitive load for adjustment
    ///
    /// # Errors
    ///
    /// Returns an error if reranking fails
    async fn rerank_by_context(
        &self,
        results: Vec<VectorSearchResult>,
        cognitive_load: f32,
    ) -> Vec<RetrievalItem> {
        let mut reranked = Vec::new();

        for result in results {
            // Get review interval if exists
            let interval = self
                .review_repository
                .get_by_knowledge_point(result.payload.entity_id)
                .await
                .ok()
                .flatten();

            let mut score = result.score;

            // Factor 1: Time decay - recently reviewed items get lower priority
            if let Some(ref interval) = interval {
                if let Some(last_reviewed) = interval.last_review_date {
                    let days_since = (Utc::now().date_naive() - last_reviewed).num_days();
                    let decay_factor = 1.0 / (1.0 + days_since as f32 * 0.1);
                    score *= decay_factor;
                }

                // Factor 2: Extraction difficulty (FSRS)
                let difficulty = self.fsrs.extraction_difficulty(
                    &mock_knowledge_point(result.payload.entity_id),
                    interval.days_overdue(),
                    interval.accuracy_rate(),
                );

                // High fatigue: prioritize easier items
                if cognitive_load > 0.6 {
                    score *= (2.0 - difficulty) * 0.5;
                } else {
                    // Normal: balance difficulty
                    score *= 1.0 + (difficulty - 0.5) * 0.2;
                }

                // Factor 3: Accuracy boost - items with low accuracy get higher priority
                score *= 1.0 + (1.0 - interval.accuracy_rate()) * 0.3;
            }

            reranked.push(RetrievalItem {
                knowledge_point_id: result.payload.entity_id,
                title: Some(result.payload.title.clone()),
                score,
                original_score: result.score,
                retrieval_reason: RetrievalReason::SemanticSimilarity,
            });
        }

        // Sort by final score
        reranked.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        reranked
    }

    /// Builds context text from retrieved items
    ///
    /// # Errors
    ///
    /// Returns an error if context building fails
    fn build_context(
        &self,
        items: &[RetrievalItem],
        cognitive_load: f32,
    ) -> Result<String, RetrievalError> {
        let mut context = String::from("【相关知识库】\n\n");

        for (i, item) in items.iter().enumerate() {
            let title = item.title.as_deref().unwrap_or("未知知识点");

            // Adapt presentation based on cognitive load
            let formatted = if cognitive_load > 0.6 {
                // High load: simplified presentation
                format!("{}. {} (简化版)\n", i + 1, title)
            } else {
                format!("{}. {}\n", i + 1, title)
            };

            context.push_str(&formatted);
        }

        // Add usage guidance
        context.push_str("\n【使用指南】\n");
        context.push_str(&format!("当前认知负荷：{:.1}\n", cognitive_load));
        context.push_str("请根据用户当前理解水平，灵活引用上述知识，\n");
        context.push_str("优先使用主动回忆（提问）而非单向灌输。\n");

        Ok(context)
    }
}

/// Retrieval context with formatted text and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalContext {
    /// Formatted context text for agent consumption
    pub context_text: String,
    /// Retrieved knowledge items
    pub knowledge_points: Vec<RetrievalItem>,
    /// Total number of items retrieved
    pub total_count: usize,
    /// When the retrieval was performed
    pub retrieved_at: DateTime<Utc>,
}

/// Single retrieval item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalItem {
    /// Knowledge point ID
    pub knowledge_point_id: Uuid,
    /// Knowledge point title
    pub title: Option<String>,
    /// Final reranked score
    pub score: f32,
    /// Original similarity score from vector search
    pub original_score: f32,
    /// Why this item was retrieved
    pub retrieval_reason: RetrievalReason,
}

/// Reason for retrieval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetrievalReason {
    /// Semantic similarity to query
    SemanticSimilarity,

    /// Due for review (FSRS)
    DueForReview,

    /// High difficulty priority
    HighDifficulty,

    /// User-specific priority
    UserPriority,
}

/// Review item for due reviews
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewItem {
    /// Knowledge point ID
    pub knowledge_point_id: Uuid,
    /// Current interval in days
    pub interval_days: f32,
    /// Current ease factor
    pub ease_factor: f32,
    /// Total number of reviews
    pub total_reviews: i32,
    /// Accuracy rate (0.0-1.0)
    pub accuracy_rate: f32,
    /// Days overdue (0 if not overdue)
    pub days_overdue: i64,
}

/// Retrieval errors
#[derive(Debug, thiserror::Error)]
pub enum RetrievalError {
    /// Vector service error
    #[error("Vector error: {0}")]
    VectorError(String),

    /// Database error
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// Context building error
    #[error("Context building error: {0}")]
    ContextError(String),
}

/// Mock knowledge point for FSRS calculation
fn mock_knowledge_point(id: Uuid) -> KnowledgePoint {
    KnowledgePoint {
        id,
        user_id: Uuid::new_v4(),
        content: String::new(),
        content_type: "concept".to_string(),
        source_type: "manual".to_string(),
        source_id: None,
        title: None,
        summary: None,
        tags: vec![],
        parent_id: None,
        related_ids: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
        last_reviewed_at: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retrieval_item_creation() {
        let item = RetrievalItem {
            knowledge_point_id: Uuid::new_v4(),
            title: Some("Test Knowledge".to_string()),
            score: 0.85,
            original_score: 0.75,
            retrieval_reason: RetrievalReason::SemanticSimilarity,
        };

        assert_eq!(item.title, Some("Test Knowledge".to_string()));
        assert!(item.score > item.original_score);
    }

    #[test]
    fn test_review_item_creation() {
        let item = ReviewItem {
            knowledge_point_id: Uuid::new_v4(),
            interval_days: 5.0,
            ease_factor: 2.6,
            total_reviews: 10,
            accuracy_rate: 0.8,
            days_overdue: 2,
        };

        assert_eq!(item.interval_days, 5.0);
        assert_eq!(item.days_overdue, 2);
    }
}
