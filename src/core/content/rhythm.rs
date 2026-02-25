// ============================================================
// Rhythm Controller
// ============================================================
//! Pacing and rhythm management for content delivery.
//!
//! This module manages the timing and pacing of content delivery
/// to optimize learning and retention.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Delivery pace setting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Pace {
    /// Slow, relaxed pace for bedtime
    Slow,
    /// Moderate pace for active learning
    Moderate,
    /// Fast pace for review
    Fast,
}

impl Pace {
    /// Returns the word count per minute for this pace
    #[must_use]
    pub const fn words_per_minute(&self) -> u32 {
        match self {
            Self::Slow => 80,
            Self::Moderate => 150,
            Self::Fast => 200,
        }
    }

    /// Returns the recommended pause duration between messages
    #[must_use]
    pub const fn pause_duration(&self) -> Duration {
        match self {
            Self::Slow => Duration::seconds(5),
            Self::Moderate => Duration::seconds(2),
            Self::Fast => Duration::seconds(1),
        }
    }
}

/// Scheduled content item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledItem {
    /// Item content
    pub content: String,
    /// Scheduled delivery time
    pub delivery_time: DateTime<Utc>,
    /// Estimated duration for delivery
    pub estimated_duration: Duration,
    /// Priority (higher = earlier delivery)
    pub priority: u8,
    /// Item identifier
    pub id: uuid::Uuid,
}

impl ScheduledItem {
    /// Creates a new scheduled item
    #[must_use]
    pub fn new(content: impl Into<String>, delay_seconds: i64) -> Self {
        Self {
            content: content.into(),
            delivery_time: Utc::now() + Duration::seconds(delay_seconds),
            estimated_duration: Duration::seconds(0),
            priority: 5,
            id: uuid::Uuid::new_v4(),
        }
    }

    /// Sets the priority for this item
    #[must_use]
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }
}

/// Rhythm controller for managing content delivery timing
pub struct RhythmController {
    /// Current pace setting
    pace: Pace,
    /// Queue of scheduled items
    queue: VecDeque<ScheduledItem>,
    /// Last delivery time
    last_delivery: Option<DateTime<Utc>>,
}

impl Default for RhythmController {
    fn default() -> Self {
        Self::new()
    }
}

impl RhythmController {
    /// Creates a new rhythm controller
    #[must_use]
    pub fn new() -> Self {
        Self {
            pace: Pace::Slow,
            queue: VecDeque::new(),
            last_delivery: None,
        }
    }

    /// Sets the delivery pace
    pub fn set_pace(&mut self, pace: Pace) {
        self.pace = pace;
    }

    /// Gets the current pace
    #[must_use]
    pub const fn pace(&self) -> Pace {
        self.pace
    }

    /// Schedules content for delivery
    ///
    /// # Arguments
    ///
    /// * `content` - Content to deliver
    /// * `delay_seconds` - Delay before delivery
    pub fn schedule(&mut self, content: impl Into<String>, delay_seconds: i64) {
        let item = ScheduledItem::new(content, delay_seconds);
        self.insert_sorted(item);
    }

    /// Inserts an item into the queue maintaining priority order
    fn insert_sorted(&mut self, item: ScheduledItem) {
        let mut inserted = false;

        for i in 0..self.queue.len() {
            if item.delivery_time < self.queue[i].delivery_time
                || (item.delivery_time == self.queue[i].delivery_time
                    && item.priority > self.queue[i].priority)
            {
                self.queue.insert(i, item);
                inserted = true;
                return;
            }
        }

        if !inserted {
            self.queue.push_back(item);
        }
    }

    /// Returns the next item ready for delivery
    ///
    /// # Returns
    ///
    /// The next scheduled item if available and ready
    #[must_use]
    pub fn next(&mut self) -> Option<ScheduledItem> {
        let now = Utc::now();

        if let Some(item) = self.queue.front() {
            if item.delivery_time <= now {
                self.last_delivery = Some(now);
                return self.queue.pop_front();
            }
        }

        None
    }

    /// Peeks at the next item without removing it
    #[must_use]
    pub fn peek(&self) -> Option<&ScheduledItem> {
        self.queue.front()
    }

    /// Returns the number of items in the queue
    #[must_use]
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Returns whether the queue is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Clears all scheduled items
    pub fn clear(&mut self) {
        self.queue.clear();
    }

    /// Calculates the estimated word count for delivery duration
    #[must_use]
    pub fn estimate_delivery_duration(&self, word_count: usize) -> Duration {
        let wpm = self.pace.words_per_minute() as f64;
        let minutes = word_count as f64 / wpm;
        Duration::seconds((minutes * 60.0) as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pace_settings() {
        assert_eq!(Pace::Slow.words_per_minute(), 80);
        assert_eq!(Pace::Moderate.words_per_minute(), 150);
        assert_eq!(Pace::Fast.words_per_minute(), 200);
    }

    #[test]
    fn test_rhythm_controller() {
        let mut controller = RhythmController::new();
        assert!(controller.is_empty());

        controller.schedule("First message", -1); // Past
        controller.schedule("Second message", 60); // Future

        assert_eq!(controller.len(), 2);
        assert_eq!(controller.peek().unwrap().content, "First message");
    }

    #[test]
    fn test_estimated_duration() {
        let controller = RhythmController::new();
        let duration = controller.estimate_delivery_duration(150);

        // Slow pace: 150 words / 80 wpm ≈ 112.5 seconds
        assert!(duration.num_seconds() > 100);
        assert!(duration.num_seconds() < 120);
    }
}
