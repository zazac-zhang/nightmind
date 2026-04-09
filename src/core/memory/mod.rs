// ============================================================
// Memory Consolidation Module
// ============================================================
//! Memory consolidation and spaced repetition systems.
//!
//! This module implements the core memory consolidation algorithms
//! based on cognitive science principles: FSRS (Free Spaced Repetition Scheduler),
//! sleep-dependent memory consolidation, and the testing effect.

pub mod fsrs;
pub mod retrieval;
pub mod extractor;

pub use fsrs::{EveningFSRS, Rating};
pub use retrieval::{KnowledgeRetriever, RetrievalContext, RetrievalError, ReviewItem};
pub use extractor::{
    KnowledgeExtractor, AnkiCard, ObsidianNote, ManualKnowledgeInput,
    DailyLearningSummary, KnowledgeSource,
};

// Re-exports for convenience
