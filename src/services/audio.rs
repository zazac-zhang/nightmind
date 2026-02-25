// ============================================================
// Audio Service
// ============================================================
//! Audio processing and playback capabilities.
//!
//! This module provides audio recording, processing, and playback
/// functionality for the application.

use serde::{Deserialize, Serialize};

/// Audio format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioFormat {
    /// WAV format
    Wav,
    /// MP3 format
    Mp3,
    /// Opus format
    Opus,
    /// FLAC format
    Flac,
}

impl AudioFormat {
    /// Returns the file extension for this format
    #[must_use]
    pub const fn extension(&self) -> &str {
        match self {
            Self::Wav => "wav",
            Self::Mp3 => "mp3",
            Self::Opus => "opus",
            Self::Flac => "flac",
        }
    }

    /// Returns the MIME type for this format
    #[must_use]
    pub const fn mime_type(&self) -> &str {
        match self {
            Self::Wav => "audio/wav",
            Self::Mp3 => "audio/mpeg",
            Self::Opus => "audio/opus",
            Self::Flac => "audio/flac",
        }
    }
}

/// Audio configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Number of channels (1 = mono, 2 = stereo)
    pub channels: u16,
    /// Bits per sample
    pub bits_per_sample: u16,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 16000,
            channels: 1,
            bits_per_sample: 16,
        }
    }
}

impl AudioConfig {
    /// Creates a new audio configuration
    #[must_use]
    pub fn new(sample_rate: u32, channels: u16, bits_per_sample: u16) -> Self {
        Self {
            sample_rate,
            channels,
            bits_per_sample,
        }
    }

    /// Returns the byte rate (bytes per second)
    #[must_use]
    pub const fn byte_rate(&self) -> u32 {
        (self.sample_rate * self.channels as u32 * self.bits_per_sample as u32) / 8
    }
}

/// Audio data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioData {
    /// Audio samples
    pub samples: Vec<i16>,
    /// Audio format
    pub format: AudioFormat,
    /// Audio configuration
    pub config: AudioConfig,
    /// Duration in seconds
    pub duration: f64,
}

impl AudioData {
    /// Creates new audio data
    #[must_use]
    pub fn new(samples: Vec<i16>, config: AudioConfig, format: AudioFormat) -> Self {
        let duration = samples.len() as f64 / config.sample_rate as f64 / config.channels as f64;

        Self {
            samples,
            format,
            config,
            duration,
        }
    }

    /// Returns the number of samples
    #[must_use]
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Returns whether the audio data is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Converts to raw bytes
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let capacity = self.samples.len() * (self.config.bits_per_sample as usize / 8);
        let mut bytes = Vec::with_capacity(capacity);

        for sample in &self.samples {
            bytes.extend_from_slice(&sample.to_le_bytes());
        }

        bytes
    }
}

/// Audio recording state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordingState {
    /// Not recording
    Idle,
    /// Currently recording
    Recording,
    /// Paused
    Paused,
}

/// Audio recorder
pub struct AudioRecorder {
    /// Current recording state
    state: RecordingState,
    /// Audio configuration
    config: AudioConfig,
    /// Recorded samples
    samples: Vec<i16>,
}

impl AudioRecorder {
    /// Creates a new audio recorder
    #[must_use]
    pub fn new(config: AudioConfig) -> Self {
        Self {
            state: RecordingState::Idle,
            config,
            samples: Vec::new(),
        }
    }

    /// Starts recording
    pub fn start(&mut self) {
        self.state = RecordingState::Recording;
        self.samples.clear();
    }

    /// Stops recording
    pub fn stop(&mut self) {
        self.state = RecordingState::Idle;
    }

    /// Pauses recording
    pub fn pause(&mut self) {
        if self.state == RecordingState::Recording {
            self.state = RecordingState::Paused;
        }
    }

    /// Resumes recording
    pub fn resume(&mut self) {
        if self.state == RecordingState::Paused {
            self.state = RecordingState::Recording;
        }
    }

    /// Gets the current recording state
    #[must_use]
    pub const fn state(&self) -> RecordingState {
        self.state
    }

    /// Gets the recorded audio data
    #[must_use]
    pub fn get_data(&self) -> AudioData {
        AudioData::new(
            self.samples.clone(),
            self.config,
            AudioFormat::Wav,
        )
    }

    /// Adds samples to the recording
    pub fn add_samples(&mut self, samples: &[i16]) {
        if self.state == RecordingState::Recording {
            self.samples.extend_from_slice(samples);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_format() {
        assert_eq!(AudioFormat::Wav.extension(), "wav");
        assert_eq!(AudioFormat::Mp3.mime_type(), "audio/mpeg");
    }

    #[test]
    fn test_audio_config() {
        let config = AudioConfig::new(16000, 1, 16);
        assert_eq!(config.sample_rate, 16000);
        assert_eq!(config.byte_rate(), 32000);
    }

    #[test]
    fn test_audio_recorder() {
        let mut recorder = AudioRecorder::new(AudioConfig::default());
        assert_eq!(recorder.state(), RecordingState::Idle);

        recorder.start();
        assert_eq!(recorder.state(), RecordingState::Recording);

        recorder.pause();
        assert_eq!(recorder.state(), RecordingState::Paused);

        recorder.resume();
        assert_eq!(recorder.state(), RecordingState::Recording);

        recorder.stop();
        assert_eq!(recorder.state(), RecordingState::Idle);
    }
}
