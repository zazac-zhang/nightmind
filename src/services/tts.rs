// ============================================================
// Text-to-Speech Service
// ============================================================
//! Text-to-speech synthesis.
//!
//! This module provides text-to-speech capabilities for converting
/// text responses into audio output.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::services::audio::{AudioData, AudioFormat};

/// Synthesis result
#[derive(Debug, Clone)]
pub struct SynthesisResult {
    /// Generated audio data
    pub audio: AudioData,
    /// Synthesis duration in seconds
    pub synthesis_time: f64,
    /// Word alignments (if available)
    pub alignments: Option<Vec<WordAlignment>>,
}

/// Word alignment with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordAlignment {
    /// The word
    pub word: String,
    /// Start time in seconds
    pub start: f64,
    /// End time in seconds
    pub end: f64,
}

/// Synthesis request
#[derive(Debug, Clone)]
pub struct SynthesisRequest {
    /// Text to synthesize
    pub text: String,
    /// Voice to use
    pub voice: String,
    /// Output audio format
    pub format: AudioFormat,
    /// Speech rate (0.1 - 2.0, 1.0 = normal)
    pub rate: f32,
    /// Pitch adjustment (0.1 - 2.0, 1.0 = normal)
    pub pitch: f32,
    /// Request ID
    pub request_id: Uuid,
}

impl SynthesisRequest {
    /// Creates a new synthesis request
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        Self {
            text,
            voice: "default".to_string(),
            format: AudioFormat::Mp3,
            rate: 1.0,
            pitch: 1.0,
            request_id: Uuid::new_v4(),
        }
    }

    /// Sets the voice to use
    #[must_use]
    pub fn with_voice(mut self, voice: impl Into<String>) -> Self {
        self.voice = voice.into();
        self
    }

    /// Sets the output format
    #[must_use]
    pub const fn with_format(mut self, format: AudioFormat) -> Self {
        self.format = format;
        self
    }

    /// Sets the speech rate
    #[must_use]
    pub fn with_rate(mut self, rate: f32) -> Self {
        self.rate = rate.clamp(0.1, 2.0);
        self
    }

    /// Sets the pitch
    #[must_use]
    pub fn with_pitch(mut self, pitch: f32) -> Self {
        self.pitch = pitch.clamp(0.1, 2.0);
        self
    }
}

/// Text-to-speech service
pub struct TtsService {
    /// Service configuration
    config: TtsConfig,
}

/// Text-to-speech configuration
#[derive(Debug, Clone)]
pub struct TtsConfig {
    /// API endpoint URL
    pub api_url: String,
    /// API key for authentication
    pub api_key: String,
    /// Default voice to use
    pub default_voice: String,
}

impl Default for TtsConfig {
    fn default() -> Self {
        Self {
            api_url: String::new(),
            api_key: String::new(),
            default_voice: "default".to_string(),
        }
    }
}

impl TtsService {
    /// Creates a new text-to-speech service
    #[must_use]
    pub fn new(config: TtsConfig) -> Self {
        Self { config }
    }

    /// Synthesizes text to speech
    ///
    /// # Arguments
    ///
    /// * `request` - Synthesis request
    ///
    /// # Returns
    ///
    /// Synthesis result or an error
    ///
    /// # Errors
    ///
    /// Returns an error if synthesis fails
    pub async fn synthesize(
        &self,
        request: SynthesisRequest,
    ) -> Result<SynthesisResult, TtsError> {
        let start = std::time::Instant::now();

        // Validate request
        if request.text.is_empty() {
            return Err(TtsError::EmptyText);
        }

        // Placeholder implementation
        let audio_data = AudioData::new(
            vec![],
            crate::services::audio::AudioConfig::default(),
            request.format,
        );

        Ok(SynthesisResult {
            audio: audio_data,
            synthesis_time: start.elapsed().as_secs_f64(),
            alignments: None,
        })
    }

    /// Gets available voices
    ///
    /// # Returns
    ///
    /// List of available voice IDs
    ///
    /// # Errors
    ///
    /// Returns an error if fetching voices fails
    pub async fn get_voices(&self) -> Result<Vec<Voice>, TtsError> {
        // Placeholder implementation
        Ok(vec![
            Voice {
                id: "default".to_string(),
                name: "Default Voice".to_string(),
                language: "en-US".to_string(),
                gender: VoiceGender::Neutral,
            }
        ])
    }
}

/// Available voice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Voice {
    /// Voice ID
    pub id: String,
    /// Voice display name
    pub name: String,
    /// Voice language code
    pub language: String,
    /// Voice gender
    pub gender: VoiceGender,
}

/// Voice gender
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceGender {
    /// Male voice
    Male,
    /// Female voice
    Female,
    /// Neutral/unspecified voice
    Neutral,
}

/// Text-to-speech errors
#[derive(Debug, thiserror::Error)]
pub enum TtsError {
    /// Empty text provided
    #[error("Cannot synthesize empty text")]
    EmptyText,

    /// Voice not found
    #[error("Voice not found: {0}")]
    VoiceNotFound(String),

    /// API error
    #[error("API error: {0}")]
    Api(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// Synthesis failed
    #[error("Synthesis failed: {0}")]
    SynthesisFailed(String),
}

/// SSML (Speech Synthesis Markup Language) helper
pub struct SsmlBuilder;

impl SsmlBuilder {
    /// Creates a basic SSML document
    #[must_use]
    pub fn build(text: &str, voice: &str) -> String {
        format!(
            r#"<speak version="1.0" xmlns="http://www.w3.org/2001/10/synthesis">
  <voice name="{}">
    {}
  </voice>
</speak>"#,
            voice, text
        )
    }

    /// Adds emphasis to text
    #[must_use]
    pub fn emphasis(text: &str, level: SsmlEmphasis) -> String {
        format!(r#"<emphasis level="{}">{}</emphasis>"#, level.as_str(), text)
    }

    /// Adds a pause
    #[must_use]
    pub fn pause(duration_ms: u64) -> String {
        format!(r#"<break time="{}ms"/>"#, duration_ms)
    }

    /// Adjusts speaking rate
    #[must_use]
    pub fn rate(text: &str, rate: f32) -> String {
        let rate_str = if rate < 0.75 {
            "x-slow"
        } else if rate < 1.0 {
            "slow"
        } else if rate < 1.25 {
            "medium"
        } else if rate < 1.75 {
            "fast"
        } else {
            "x-fast"
        };
        format!(r#"<prosody rate="{}">{}</prosody>"#, rate_str, text)
    }
}

/// SSML emphasis levels
#[derive(Debug, Clone, Copy)]
pub enum SsmlEmphasis {
    /// Strong emphasis
    Strong,
    /// Moderate emphasis
    Moderate,
    /// Reduced emphasis
    Reduced,
}

impl SsmlEmphasis {
    /// Returns the SSML string value
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Strong => "strong",
            Self::Moderate => "moderate",
            Self::Reduced => "reduced",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synthesis_request() {
        let request = SynthesisRequest::new("Hello, world!")
            .with_voice("en-US-Wavenet-D")
            .with_rate(1.2)
            .with_pitch(1.1);

        assert_eq!(request.text, "Hello, world!");
        assert_eq!(request.voice, "en-US-Wavenet-D");
        assert!((request.rate - 1.2).abs() < f32::EPSILON);
    }

    #[test]
    fn test_ssml_builder() {
        let ssml = SsmlBuilder::build("Hello", "default");
        assert!(ssml.contains("<speak"));
        assert!(ssml.contains("Hello"));
        assert!(ssml.contains("</speak>"));
    }

    #[test]
    fn test_ssml_emphasis() {
        let emphasized = SsmlBuilder::emphasis("important", SsmlEmphasis::Strong);
        assert!(emphasized.contains("<emphasis"));
        assert!(emphasized.contains("strong"));
    }

    #[test]
    fn test_rate_clamping() {
        let request = SynthesisRequest::new("test").with_rate(5.0);
        assert_eq!(request.rate, 2.0); // Should be clamped to max

        let request = SynthesisRequest::new("test").with_rate(0.05);
        assert_eq!(request.rate, 0.1); // Should be clamped to min
    }
}
