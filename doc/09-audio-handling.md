# 音频处理设计

## 1. 概述

音频处理是 NightMind 的核心交互方式，负责：
- 实时语音识别 (STT)
- 自然语音合成 (TTS)
- 音频流编解码
- 全双工通信管理

---

## 2. 架构设计

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Audio Processing Pipeline                     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Client ──[WebSocket]──> AudioBuffer ──[STT]──> Transcript ──> Agent│
│     ▲                        │                                       │
│     │                        ▼                                       │
│     └───────[TTS]──────< AudioQueue ─────< Response Stream          │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 3. STT (语音识别)

### 3.1 服务接口

```rust
#[async_trait]
pub trait SttService: Send + Sync {
    /// 非流式识别（用于短音频）
    async fn transcribe(&self, audio_data: Vec<u8>) -> Result<String>;

    /// 流式识别（用于实时音频流）
    async fn transcribe_stream(
        &self,
        stream: AudioStream,
    ) -> Result<impl Stream<Item = String>>;

    /// 带语言检测的识别
    async fn transcribe_with_language_detection(
        &self,
        audio_data: Vec<u8>,
    ) -> Result<TranscriptionResult>;
}

#[derive(Debug, Clone)]
pub struct TranscriptionResult {
    pub text: String,
    pub language: String,
    pub confidence: f32,
    pub duration: Duration,
    pub segments: Vec<Segment>,
}

#[derive(Debug, Clone)]
pub struct Segment {
    pub text: String,
    pub start: Duration,
    pub end: Duration,
    pub confidence: f32,
}
```

### 3.2 Whisper 实现

```rust
pub struct WhisperSttService {
    api_key: String,
    model: WhisperModel,
    client: reqwest::Client,
}

#[derive(Debug, Clone)]
pub enum WhisperModel {
    Tiny,
    Base,
    Small,
    Medium,
    Large,
}

impl WhisperSttService {
    pub async fn transcribe(&self, audio_data: Vec<u8>) -> Result<String> {
        // 1. 音频预处理（如果需要）
        let processed = self.preprocess_audio(audio_data).await?;

        // 2. 调用 Whisper API
        let response = self.client
            .post("https://api.openai.com/v1/audio/transcriptions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(
                Form::new()
                    .part("file", audio_part(processed))
                    .part("model", Part::text(self.model.as_str()))
                    .part("language", Part::text("zh"))
                    .part("response_format", Part::text("json"))
            )
            .send()
            .await?;

        // 3. 解析响应
        let result: WhisperResponse = response.json().await?;
        Ok(result.text)
    }

    pub async fn transcribe_stream(
        &self,
        mut audio_stream: AudioStream,
    ) -> Result<impl Stream<Item = String>> {
        let (tx, rx) = tokio::sync::mpsc::channel(32);

        tokio::spawn(async move {
            let mut buffer = Vec::new();
            let buffer_duration = Duration::from_secs(3);  // 每3秒处理一次

            while let Some(chunk) = audio_stream.next().await {
                match chunk {
                    Ok(audio_data) => {
                        buffer.extend(audio_data);

                        // 检查是否达到处理阈值
                        if self.duration_from_bytes(&buffer) >= buffer_duration {
                            if let Ok(text) = self.transcribe(buffer.clone()).await {
                                let _ = tx.send(text).await;
                            }
                            buffer.clear();
                        }
                    }
                    Err(e) => {
                        tracing::error!("Audio stream error: {:?}", e);
                        break;
                    }
                }
            }

            // 处理剩余数据
            if !buffer.is_empty() {
                if let Ok(text) = self.transcribe(buffer).await {
                    let _ = tx.send(text).await;
                }
            }
        });

        Ok(tokio_stream::wrappers::ReceiverStream::new(rx))
    }

    fn duration_from_bytes(&self, bytes: &[u8]) -> Duration {
        // 假设 16kHz, 16-bit, mono
        let samples = bytes.len() / 2;
        Duration::from_secs_f64(samples as f64 / 16000.0)
    }
}

#[derive(Debug, Deserialize)]
struct WhisperResponse {
    text: String,
}
```

### 3.3 本地 Whisper (可选)

```rust
pub struct LocalWhisperService {
    model: WhisperModel,
}

impl LocalWhisperService {
    pub fn new(model: WhisperModel) -> Result<Self> {
        // 加载本地 Whisper 模型
        // 使用 burn/candle/triton 等框架
        Ok(Self { model })
    }

    pub async fn transcribe(&self, audio_data: Vec<u8>) -> Result<String> {
        // 使用本地模型推理
        Ok(String::new())
    }
}
```

---

## 4. TTS (语音合成)

### 4.1 服务接口

```rust
#[async_trait]
pub trait TtsService: Send + Sync {
    /// 非流式合成
    async fn synthesize(&self, text: &str, options: &TtsOptions)
        -> Result<Vec<u8>>;

    /// 流式合成
    async fn synthesize_stream(
        &self,
        text: &str,
        options: &TtsOptions,
    ) -> Result<impl Stream<Item = Vec<u8>>>;

    /// 估算时长
    fn estimate_duration(&self, text: &str, options: &TtsOptions)
        -> Duration;
}

#[derive(Debug, Clone)]
pub struct TtsOptions {
    pub voice: String,
    pub speed: f32,
    pub pitch: f32,
    pub format: AudioFormat,
    pub language: String,
}

#[derive(Debug, Clone, Copy)]
pub enum AudioFormat {
    Mp3,
    Wav,
    Ogg,
    Pcm,
}
```

### 4.2 ElevenLabs 实现

```rust
pub struct ElevenLabsTtsService {
    api_key: String,
    client: reqwest::Client,
    default_voice: String,
}

impl ElevenLabsTtsService {
    pub async fn synthesize(
        &self,
        text: &str,
        options: &TtsOptions,
    ) -> Result<Vec<u8>> {
        let response = self.client
            .post(format!(
                "https://api.elevenlabs.io/v1/text-to-speech/{}",
                options.voice.as_str()
            ))
            .header("xi-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "text": text,
                "model_id": "eleven_multilingual_v2",
                "voice_settings": {
                    "stability": 0.5,
                    "similarity_boost": 0.75,
                    "speed": options.speed,
                }
            }))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.bytes().await?.to_vec())
        } else {
            Err(anyhow::anyhow!("TTS request failed: {}", response.status()))
        }
    }

    pub async fn synthesize_stream(
        &self,
        text: &str,
        options: &TtsOptions,
    ) -> Result<impl Stream<Item = Vec<u8>>> {
        let (tx, rx) = tokio::sync::mpsc::channel(16);

        // 按句子分段
        let sentences: Vec<&str> = text.split_inclusive(&['.', '。', '!', '！', '?', '？'][..])
            .collect();

        for sentence in sentences {
            let audio = self.synthesize(sentence, options).await?;
            tx.send(audio).await?;
        }

        Ok(tokio_stream::wrappers::ReceiverStream::new(rx))
    }

    fn estimate_duration(&self, text: &str, options: &TtsOptions) -> Duration {
        // 平均语速: 150 词/分钟 (英文) 或 200 字/分钟 (中文)
        let char_count = text.chars().count() as f32;
        let speed_factor = 1.0 / options.speed;
        Duration::from_secs_f64((char_count / 200.0) * speed_factor * 60.0)
    }
}
```

### 4.3 Azure TTS 实现

```rust
pub struct AzureTtsService {
    api_key: String,
    region: String,
    client: reqwest::Client,
}

impl AzureTtsService {
    pub async fn synthesize(
        &self,
        text: &str,
        options: &TtsOptions,
    ) -> Result<Vec<u8>> {
        let ssml = self.build_ssml(text, options);

        let response = self.client
            .post(format!(
                "https://{}.tts.speech.microsoft.com/cognitiveservices/v1",
                self.region
            ))
            .header("Ocp-Apim-Subscription-Key", &self.api_key)
            .header("Content-Type", "application/ssml+xml")
            .header("X-Microsoft-OutputFormat", "audio-16khz-128kbitrate-mono-mp3")
            .body(ssml)
            .send()
            .await?;

        Ok(response.bytes().await?.to_vec())
    }

    fn build_ssml(&self, text: &str, options: &TtsOptions) -> String {
        format!(
            r#"<speak version='1.0' xml:lang='{}'>
                <voice xml:lang='{}' name='{}'>
                    <prosody rate='{}'>
                        {}
                    </prosody>
                </voice>
            </speak>"#,
            options.language,
            options.language,
            options.voice,
            (options.speed * 100.0) as i32,
            html_escape::encode_text(text.as_bytes())
        )
    }
}
```

---

## 5. 音频流处理

### 5.1 音频缓冲

```rust
pub struct AudioBuffer {
    buffer: Arc<Mutex<VecDeque<u8>>>,
    capacity: usize,
    sample_rate: u32,
    channels: u16,
}

impl AudioBuffer {
    pub fn new(capacity: Duration, sample_rate: u32, channels: u16) -> Self {
        let bytes_per_sample = 2;  // 16-bit
        let capacity_bytes = (capacity.as_secs_f64()
            * sample_rate as f64
            * channels as f64
            * bytes_per_sample as f64) as usize;

        Self {
            buffer: Arc::new(Mutex::new(VecDeque::with_capacity(capacity_bytes))),
            capacity: capacity_bytes,
            sample_rate,
            channels,
        }
    }

    pub async fn write(&self, data: Vec<u8>) -> Result<()> {
        let mut buffer = self.buffer.lock().await;
        buffer.extend(data);

        // 保持缓冲区大小
        while buffer.len() > self.capacity {
            buffer.pop_front();
        }

        Ok(())
    }

    pub async fn read(&self, duration: Duration) -> Vec<u8> {
        let bytes_needed = (duration.as_secs_f64()
            * self.sample_rate as f64
            * self.channels as f64
            * 2.0) as usize;

        let mut buffer = self.buffer.lock().await;
        let len = buffer.len().min(bytes_needed);
        buffer.drain(..len).collect()
    }

    pub async fn is_empty(&self) -> bool {
        self.buffer.lock().await.is_empty()
    }

    pub async fn duration(&self) -> Duration {
        let buffer = self.buffer.lock().await;
        let samples = buffer.len() / 2 / self.channels as usize;
        Duration::from_secs_f64(samples as f64 / self.sample_rate as f64)
    }
}
```

### 5.2 VAD (语音活动检测)

```rust
pub struct VadDetector {
    threshold: f32,
    silence_duration: Duration,
    last_voice_activity: Arc<Mutex<DateTime<Utc>>>,
}

impl VadDetector {
    pub fn new(threshold: f32, silence_duration: Duration) -> Self {
        Self {
            threshold,
            silence_duration,
            last_voice_activity: Arc::new(Mutex::new(Utc::now())),
        }
    }

    pub async fn process(&self, audio_data: &[u8]) -> VadResult {
        // 计算能量
        let energy = self.calculate_energy(audio_data);

        // 判断是否为语音
        let is_speech = energy > self.threshold;

        if is_speech {
            *self.last_voice_activity.lock().await = Utc::now();
        }

        let silence_duration = Utc::now() - *self.last_voice_activity.lock().await;
        let is_silence = silence_duration > self.silence_duration;

        VadResult {
            is_speech,
            is_silence,
            energy,
            silence_duration,
        }
    }

    fn calculate_energy(&self, audio_data: &[u8]) -> f32 {
        // 解码 16-bit PCM
        let samples: Vec<i16> = audio_data
            .chunks_exact(2)
            .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();

        // 计算 RMS
        let sum: i64 = samples.iter().map(|&s| (s as i64).pow(2)).sum();
        (sum as f32 / samples.len() as f32).sqrt()
    }
}

#[derive(Debug, Clone)]
pub struct VadResult {
    pub is_speech: bool,
    pub is_silence: bool,
    pub energy: f32,
    pub silence_duration: Duration,
}
```

---

## 6. 音频处理流程

### 6.1 完整流程

```rust
pub struct AudioPipeline {
    stt: Arc<dyn SttService>,
    tts: Arc<dyn TtsService>,
    vad: VadDetector,
    buffer: AudioBuffer,
}

impl AudioPipeline {
    pub async fn process_client_audio(
        &self,
        audio_data: Vec<u8>,
        session_id: Uuid,
    ) -> Result<TranscriptionResult> {
        // 1. VAD 检测
        let vad_result = self.vad.process(&audio_data).await;

        if !vad_result.is_speech {
            return Ok(TranscriptionResult {
                text: String::new(),
                language: "zh".to_string(),
                confidence: 0.0,
                duration: Duration::ZERO,
                segments: vec![],
            });
        }

        // 2. 写入缓冲
        self.buffer.write(audio_data).await?;

        // 3. 检查是否需要处理
        if self.buffer.duration().await >= Duration::from_secs(2) {
            // 4. STT 识别
            let audio = self.buffer.read(Duration::from_secs(3)).await;
            let result = self.stt.transcribe(audio).await?;

            Ok(result)
        } else {
            Ok(TranscriptionResult::empty())
        }
    }

    pub async fn stream_agent_response(
        &self,
        text: &str,
        options: &TtsOptions,
        event_tx: broadcast::Sender<SessionEvent>,
    ) -> Result<()> {
        // 1. 分句处理
        let sentences: Vec<&str> = text.split_inclusive(&['.', '。'][..])
            .collect();

        // 2. 流式 TTS
        for sentence in sentences {
            let audio = self.tts.synthesize(sentence, options).await?;

            // 3. 发送音频
            event_tx.send(SessionEvent::AudioTTS(audio))?;

            // 4. 等待播放（估算）
            let duration = self.tts.estimate_duration(sentence, options);
            tokio::time::sleep(duration).await;
        }

        Ok(())
    }
}
```

---

## 7. 音频格式

### 7.1 支持格式

| 格式 | 编码 | 采样率 | 声道 | 用途 |
|------|------|--------|------|------|
| WAV | PCM 16-bit | 16kHz | Mono | STT 输入 |
| MP3 | MP3 | 24kHz | Mono | TTS 输出 |
| OGG | Opus | 24kHz | Mono | TTS 输出（低延迟） |
| PCM | PCM 16-bit | 16kHz | Mono | WebSocket 传输 |

### 7.2 音频转换

```rust
pub trait AudioConverter: Send + Sync {
    async fn convert(&self, input: AudioData, output_format: AudioFormat)
        -> Result<AudioData>;
}

pub struct AudioData {
    pub data: Vec<u8>,
    pub format: AudioFormat,
    pub sample_rate: u32,
    pub channels: u16,
}

// 使用 symphonia/rodio 进行格式转换
pub struct SymphoniaConverter;

impl AudioConverter for SymphoniaConverter {
    async fn convert(&self, input: AudioData, output_format: AudioFormat)
        -> Result<AudioData>
    {
        // 实现音频格式转换
        Ok(AudioData {
            data: vec![],
            format: output_format,
            sample_rate: input.sample_rate,
            channels: input.channels,
        })
    }
}
```

---

## 8. 性能优化

### 8.1 连接池

```rust
pub struct TtsServicePool {
    services: Vec<Arc<dyn TtsService>>,
    current: Arc<AtomicUsize>,
}

impl TtsServicePool {
    pub async fn synthesize(&self, text: &str, options: &TtsOptions)
        -> Result<Vec<u8>>
    {
        let index = self.current.fetch_add(1, Ordering::Relaxed) % self.services.len();
        let service = &self.services[index];
        service.synthesize(text, options).await
    }
}
```

### 8.2 缓存

```rust
pub struct TtsCache {
    cache: Arc<Mutex<LruCache<String, Vec<u8>>>>,
}

impl TtsCache {
    pub async fn get(&self, text: &str) -> Option<Vec<u8>> {
        self.cache.lock().await.get(text).cloned()
    }

    pub async fn set(&self, text: String, audio: Vec<u8>) {
        self.cache.lock().await.put(text, audio);
    }
}
```
