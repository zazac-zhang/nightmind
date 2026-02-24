# API 设计

## 1. 概述

NightMind 提供两类 API：
- **WebSocket API**: 实时双向通信（主要交互方式）
- **REST API**: 资源管理、配置、查询

---

## 2. WebSocket API

### 2.1 连接端点

```
WS /api/v1/ws/session
```

**认证**: 通过 Query Parameter 或 Header 传递 Token

```
ws://localhost:8080/api/v1/ws/session?token={jwt_token}
```

### 2.2 消息协议

所有消息均为 JSON 格式：

```json
{
  "type": "message_type",
  "data": { ... },
  "id": "optional_message_id",
  "timestamp": "2024-01-15T22:30:00Z"
}
```

### 2.3 客户端 → 服务端

#### 音频数据

```json
{
  "type": "audio_chunk",
  "data": {
    "format": "wav",
    "sample_rate": 16000,
    "encoding": "base64",
    "payload": "base64_encoded_audio_data"
  }
}
```

#### 文本输入

```json
{
  "type": "text_input",
  "data": {
    "text": "什么是装饰器？"
  }
}
```

#### 控制命令

```json
{
  "type": "command",
  "data": {
    "action": "pause|resume|end",
    "reason": "optional_reason"
  }
}
```

#### 状态查询

```json
{
  "type": "status_query",
  "data": {}
}
```

#### 中断/恢复

```json
{
  "type": "interrupt",
  "data": {
    "save_context": true
  }
}
```

### 2.4 服务端 → 客户端

#### TTS 音频

```json
{
  "type": "tts_audio",
  "data": {
    "format": "mp3",
    "encoding": "base64",
    "payload": "base64_encoded_audio",
    "text": "对应的文本（可选）"
  }
}
```

#### 触觉反馈

```json
{
  "type": "haptic",
  "data": {
    "pattern": "short1|short2|long1|light_vibrate",
    "intensity": 0.8
  }
}
```

#### 状态更新

```json
{
  "type": "state_update",
  "data": {
    "phase": "deep_dive",
    "elapsed_seconds": 450,
    "current_topic": "装饰器概念",
    "fatigue_score": 45
  }
}
```

#### 话题变化

```json
{
  "type": "topic_changed",
  "data": {
    "from": "Python 基础",
    "to": "装饰器概念",
    "depth": 2
  }
}
```

#### 阶段转换

```json
{
  "type": "phase_changed",
  "data": {
    "from": "warmup",
    "to": "deep_dive",
    "message": "让我们深入一点..."
  }
}
```

#### 错误

```json
{
  "type": "error",
  "data": {
    "code": "STT_SERVICE_UNAVAILABLE",
    "message": "语音识别服务暂时不可用，请稍后重试",
    "retry_after": 60
  }
}
```

#### 会话结束

```json
{
  "type": "session_ending",
  "data": {
    "reason": "user_initiated|time_elapsed|fatigue_detected",
    "summary": {
      "duration_seconds": 1800,
      "topics_discussed": ["装饰器概念", "函数式编程"],
      "insights": 3
    }
  }
}
```

### 2.5 心跳机制

```json
// 客户端每 30 秒发送
{
  "type": "ping",
  "data": {
    "timestamp": 1705334400
  }
}

// 服务端响应
{
  "type": "pong",
  "data": {
    "timestamp": 1705334400,
    "server_time": 1705334405
  }
}
```

---

## 3. REST API

### 3.1 基础信息

**Base URL**: `/api/v1`

**认证**: Bearer Token (JWT)

```http
Authorization: Bearer {jwt_token}
```

**响应格式**:

```json
{
  "data": { ... },
  "meta": {
    "page": 1,
    "per_page": 20,
    "total": 100
  },
  "error": null
}
```

### 3.2 用户管理

#### 注册

```http
POST /api/v1/auth/register
Content-Type: application/json

{
  "username": "nightmind_user",
  "email": "user@example.com",
  "password": "secure_password"
}

201 Created

{
  "data": {
    "id": "uuid",
    "username": "nightmind_user",
    "email": "user@example.com",
    "created_at": "2024-01-15T00:00:00Z"
  }
}
```

#### 登录

```http
POST /api/v1/auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "secure_password"
}

200 OK

{
  "data": {
    "token": "jwt_token",
    "refresh_token": "refresh_token",
    "expires_in": 3600,
    "user": { ... }
  }
}
```

#### 获取用户信息

```http
GET /api/v1/users/me

200 OK

{
  "data": {
    "id": "uuid",
    "username": "nightmind_user",
    "email": "user@example.com",
    "preferences": { ... },
    "integration_config": { ... }
  }
}
```

#### 更新用户偏好

```http
PATCH /api/v1/users/me/preferences
Content-Type: application/json

{
  "voice": {
    "gender": "female",
    "speed": 1.0
  },
  "session": {
    "default_duration": 3600
  }
}

200 OK
```

### 3.3 会话管理

#### 创建会话

```http
POST /api/v1/sessions
Content-Type: application/json

{
  "planned_duration": 3600,
  "auto_start": true
}

201 Created

{
  "data": {
    "id": "session_uuid",
    "ws_url": "ws://localhost:8080/api/v1/ws/session?token=...",
    "phase": "warmup",
    "started_at": "2024-01-15T22:00:00Z"
  }
}
```

#### 获取会话列表

```http
GET /api/v1/sessions?page=1&per_page=20&status=active

200 OK

{
  "data": [
    {
      "id": "uuid",
      "phase": "deep_dive",
      "started_at": "2024-01-15T22:00:00Z",
      "elapsed_seconds": 450,
      "current_topic": "装饰器概念"
    }
  ],
  "meta": { ... }
}
```

#### 获取会话详情

```http
GET /api/v1/sessions/{session_id}

200 OK

{
  "data": {
    "id": "uuid",
    "user_id": "uuid",
    "phase": "deep_dive",
    "state": { ... },
    "topic_stack": { ... },
    "metrics": { ... },
    "started_at": "2024-01-15T22:00:00Z",
    "sync_status": { ... }
  }
}
```

#### 结束会话

```http
POST /api/v1/sessions/{session_id}/end
Content-Type: application/json

{
  "reason": "user_initiated"
}

200 OK

{
  "data": {
    "id": "uuid",
    "ended_at": "2024-01-15T23:00:00Z",
    "duration_seconds": 3600,
    "summary": {
      "topics_discussed": [...],
      "insights_generated": 5,
      "cards_created": 3
    }
  }
}
```

#### 恢复会话

```http
POST /api/v1/sessions/{session_id}/restore

200 OK

{
  "data": {
    "can_restore": true,
    "recovery_prompt": "我们刚才聊到「装饰器概念」，正想请你理解它的作用。要继续吗？",
    "elapsed_seconds": 450,
    "snapshot": { ... }
  }
}
```

### 3.4 知识管理

#### 创建知识点

```http
POST /api/v1/knowledge
Content-Type: application/json

{
  "content": "装饰器是 Python 中用于修改函数或类行为的工具...",
  "content_type": "concept",
  "source_type": "manual",
  "title": "装饰器概念",
  "tags": ["python", "decorator"]
}

201 Created
```

#### 搜索知识点

```http
GET /api/v1/knowledge/search?q=装饰器&limit=10

200 OK

{
  "data": [
    {
      "id": "uuid",
      "content": "...",
      "score": 0.95
    }
  ]
}
```

#### 获取待复习内容

```http
GET /api/v1/knowledge/due

200 OK

{
  "data": [
    {
      "id": "uuid",
      "content": "...",
      "due_date": "2024-01-15",
      "interval_days": 5,
      "ease_factor": 2.5
    }
  ]
}
```

#### 提交复习结果

```http
POST /api/v1/knowledge/{id}/review
Content-Type: application/json

{
  "rating": "good",
  "time_spent_seconds": 30
}

200 OK

{
  "data": {
    "next_review_date": "2024-01-20",
    "interval_days": 10,
    "ease_factor": 2.6
  }
}
```

### 3.5 外部集成

#### 配置集成

```http
PATCH /api/v1/integrations/{service}
Content-Type: application/json

{
  "enabled": true,
  "config": {
    "connect_url": "http://localhost:8765",
    "default_deck": "NightMind"
  }
}

200 OK
```

#### 同步状态

```http
GET /api/v1/integrations/sync-status

200 OK

{
  "data": {
    "anki": {
      "enabled": true,
      "last_sync": "2024-01-15T23:00:00Z",
      "status": "synced"
    },
    "obsidian": {
      "enabled": true,
      "last_sync": "2024-01-15T23:00:00Z",
      "status": "synced"
    }
  }
}
```

#### 手动同步

```http
POST /api/v1/integrations/sync

200 OK

{
  "data": {
    "started_at": "2024-01-15T23:05:00Z",
    "status": "in_progress"
  }
}
```

### 3.6 统计与分析

#### 获取用户统计

```http
GET /api/v1/stats/summary?period=7d

200 OK

{
  "data": {
    "total_sessions": 10,
    "total_duration_seconds": 36000,
    "avg_session_duration": 3600,
    "knowledge_points_reviewed": 50,
    "insights_generated": 15,
    "completion_rate": 0.85
  }
}
```

#### 获取学习曲线

```http
GET /api/v1/stats/learning-curve?days=30

200 OK

{
  "data": {
    "dates": ["2024-01-01", "2024-01-02", ...],
    "reviews": [5, 8, 12, ...],
    "retention_rate": [0.8, 0.85, 0.82, ...]
  }
}
```

---

## 4. 错误码

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `UNAUTHORIZED` | 401 | 未认证或 Token 无效 |
| `FORBIDDEN` | 403 | 无权限访问 |
| `NOT_FOUND` | 404 | 资源不存在 |
| `RATE_LIMIT_EXCEEDED` | 429 | 超出速率限制 |
| `INTERNAL_ERROR` | 500 | 服务器内部错误 |
| `SERVICE_UNAVAILABLE` | 503 | 服务暂时不可用 |
| `STT_SERVICE_UNAVAILABLE` | 503 | STT 服务不可用 |
| `TTS_SERVICE_UNAVAILABLE` | 503 | TTS 服务不可用 |
| `LLM_SERVICE_UNAVAILABLE` | 503 | LLM 服务不可用 |
| `SESSION_NOT_FOUND` | 404 | 会话不存在 |
| `SESSION_ALREADY_CLOSED` | 400 | 会话已关闭 |
| `INVALID_STATE_TRANSITION` | 400 | 无效的状态转换 |
| `INTEGRATION_ERROR` | 500 | 外部集成错误 |

---

## 5. 速率限制

| Endpoint | Limit | Window |
|----------|-------|--------|
| WebSocket 连接 | 5 连接/用户 | 1 分钟 |
| REST API | 100 请求/用户 | 1 分钟 |
| 创建会话 | 10 会话/用户 | 1 小时 |
| 搜索知识点 | 50 请求/用户 | 1 分钟 |

Rate Limit Headers:

```http
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1705334460
```

---

## 6. DTO 定义

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// 请求 DTO
#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub planned_duration: Option<u64>,
    pub auto_start: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePreferencesRequest {
    pub voice: Option<VoicePreferences>,
    pub session: Option<SessionPreferences>,
    pub content: Option<ContentPreferences>,
}

#[derive(Debug, Deserialize)]
pub struct ReviewRequest {
    pub rating: ReviewRating,
    pub time_spent_seconds: Option<u64>,
}

// 响应 DTO
#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub id: Uuid,
    pub phase: SessionState,
    pub started_at: DateTime<Utc>,
    pub elapsed_seconds: u64,
    pub current_topic: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SessionDetail {
    pub id: Uuid,
    pub user_id: Uuid,
    pub phase: SessionState,
    pub state: serde_json::Value,
    pub topic_stack: TopicStack,
    pub metrics: SessionMetrics,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub sync_status: SyncStatus,
}

#[derive(Debug, Serialize)]
pub struct KnowledgePointResponse {
    pub id: Uuid,
    pub content: String,
    pub content_type: ContentType,
    pub score: Option<f32>,
    pub source_type: SourceType,
}

// WebSocket 消息
#[derive(Debug, Serialize, Deserialize)]
pub struct WsMessage<T> {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AudioChunkData {
    pub format: String,
    pub sample_rate: u32,
    pub encoding: String,
    pub payload: String,
}

#[derive(Debug, Serialize)]
pub struct TtsAudioData {
    pub format: String,
    pub encoding: String,
    pub payload: String,
    pub text: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct HapticData {
    pub pattern: String,
    pub intensity: f32,
}
```
