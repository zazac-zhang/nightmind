// ============================================================
// Topic Stack
// ============================================================
//! Topic tracking for conversation context.
//!
//! This module manages a stack of topics discussed in a session,
/// allowing for context tracking and easy navigation.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use uuid::Uuid;

/// A topic discussed in the session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topic {
    /// Unique topic identifier
    pub id: Uuid,
    /// Topic title/label
    pub title: String,
    /// Detailed topic description
    pub description: Option<String>,
    /// Timestamp when topic was introduced
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Associated keywords for this topic
    pub keywords: Vec<String>,
    /// Parent topic if this is a subtopic
    pub parent_id: Option<Uuid>,
}

impl Topic {
    /// Creates a new topic
    ///
    /// # Arguments
    ///
    /// * `title` - Topic title
    #[must_use]
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            description: None,
            timestamp: chrono::Utc::now(),
            keywords: Vec::new(),
            parent_id: None,
        }
    }

    /// Sets the topic description
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Adds keywords to the topic
    #[must_use]
    pub fn with_keywords(mut self, keywords: Vec<String>) -> Self {
        self.keywords = keywords;
        self
    }

    /// Sets the parent topic
    #[must_use]
    pub fn with_parent(mut self, parent_id: Uuid) -> Self {
        self.parent_id = Some(parent_id);
        self
    }
}

/// Stack for managing conversation topics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicStack {
    /// Stack of topics (most recent first)
    topics: VecDeque<Topic>,
    /// Maximum stack size
    max_size: usize,
}

impl Default for TopicStack {
    fn default() -> Self {
        Self::new(50)
    }
}

impl TopicStack {
    /// Creates a new topic stack
    ///
    /// # Arguments
    ///
    /// * `max_size` - Maximum number of topics to track
    #[must_use]
    pub fn new(max_size: usize) -> Self {
        Self {
            topics: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    /// Pushes a new topic onto the stack
    ///
    /// # Arguments
    ///
    /// * `topic` - Topic to add
    pub fn push(&mut self, topic: Topic) {
        if self.topics.len() >= self.max_size {
            self.topics.pop_back();
        }
        self.topics.push_front(topic);
    }

    /// Pops the most recent topic from the stack
    ///
    /// # Returns
    ///
    /// The most recent topic, or None if empty
    #[must_use]
    pub fn pop(&mut self) -> Option<Topic> {
        self.topics.pop_front()
    }

    /// Returns the current topic without removing it
    #[must_use]
    pub fn current(&self) -> Option<&Topic> {
        self.topics.front()
    }

    /// Returns all topics in order
    #[must_use]
    pub fn topics(&self) -> &VecDeque<Topic> {
        &self.topics
    }

    /// Returns the number of topics in the stack
    #[must_use]
    pub fn len(&self) -> usize {
        self.topics.len()
    }

    /// Returns whether the stack is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.topics.is_empty()
    }

    /// Clears all topics from the stack
    pub fn clear(&mut self) {
        self.topics.clear();
    }

    /// Finds topics containing the given keyword
    ///
    /// # Arguments
    ///
    /// * `keyword` - Keyword to search for
    ///
    /// # Returns
    ///
    /// Vector of matching topics
    #[must_use]
    pub fn find_by_keyword(&self, keyword: &str) -> Vec<&Topic> {
        self.topics
            .iter()
            .filter(|t| t.keywords.iter().any(|k| k.eq_ignore_ascii_case(keyword)))
            .collect()
    }

    /// Finds topics with titles matching the query
    ///
    /// # Arguments
    ///
    /// * `query` - Search query
    ///
    /// # Returns
    ///
    /// Vector of matching topics
    #[must_use]
    pub fn search(&self, query: &str) -> Vec<&Topic> {
        self.topics
            .iter()
            .filter(|t| {
                t.title
                    .to_lowercase()
                    .contains(&query.to_lowercase())
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topic_creation() {
        let topic = Topic::new("Test Topic")
            .with_description("A test topic")
            .with_keywords(vec!["test".to_string(), "example".to_string()]);

        assert_eq!(topic.title, "Test Topic");
        assert_eq!(topic.description, Some("A test topic".to_string()));
        assert_eq!(topic.keywords.len(), 2);
    }

    #[test]
    fn test_topic_stack() {
        let mut stack = TopicStack::new(10);

        assert!(stack.is_empty());

        stack.push(Topic::new("Topic 1"));
        stack.push(Topic::new("Topic 2"));

        assert_eq!(stack.len(), 2);
        assert_eq!(stack.current().unwrap().title, "Topic 2");

        let popped = stack.pop();
        assert!(popped.is_some());
        assert_eq!(popped.unwrap().title, "Topic 2");
        assert_eq!(stack.len(), 1);
    }

    #[test]
    fn test_topic_search() {
        let mut stack = TopicStack::new(10);

        stack.push(
            Topic::new("Programming")
                .with_keywords(vec!["code".to_string(), "development".to_string()]),
        );
        stack.push(Topic::new("Math"));

        let results = stack.find_by_keyword("code");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Programming");
    }

    #[test]
    fn test_max_size_enforcement() {
        let mut stack = TopicStack::new(3);

        for i in 0..5 {
            stack.push(Topic::new(format!("Topic {i}")));
        }

        assert_eq!(stack.len(), 3);
        // Oldest topics should be removed
        assert!(stack.search("Topic 0").is_empty());
        assert!(stack.search("Topic 1").is_empty());
        assert!(!stack.search("Topic 2").is_empty());
    }
}
