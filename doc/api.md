# NightMind API 设计

## 概述

NightMind 提供两类 API：
- **WebSocket API**: 实时双向通信（主要交互方式）
- **REST API**: 资源管理、查询

---

## WebSocket API

### 连接端点

```
WS /api/v1/ws/session
```

认证方式：Query Parameter 传递 JWT Token

```
ws://localhost:8080/api/v1/ws/session?token={jwt_token}
```

### 客户端 → 服务端

| 消息类型 | 描述 | 数据格式 |
|----------|------|----------|
| `text_input` | 文本输入 | `{text}` |
| `audio_chunk` | 音频数据（TBD） | `{format, sample_rate, payload}` |
| `command` | 控制命令 | `{action: pause\|resume\|end}` |
| `ping` | 心跳 | `{timestamp}` |

### 服务端 → 客户端

| 消息类型 | 描述 | 数据格式 |
|----------|------|----------|
| `text_response` | 文本回复 | `{text}` |
| `tts_audio` | TTS 音频（TBD） | `{format, payload}` |
| `state_update` | 会话状态 | `{phase, elapsed_seconds}` |
| `error` | 错误 | `{code, message}` |
| `pong` | 心跳响应 | `{timestamp}` |

### 消息协议

所有消息均为 JSON 格式：

```json
{
  "type": "message_type",
  "data": { ... },
  "timestamp": "2024-01-15T22:30:00Z"
}
```

---

## REST API

### 基础信息

**Base URL**: `/api/v1`

**认证**: Bearer Token (JWT)

```http
Authorization: Bearer {jwt_token}
```

### 端点概览

| 端点 | 方法 | 描述 | 优先级 |
|------|------|------|--------|
| `/sessions` | POST | 创建会话 | P0 |
| `/sessions/{id}` | GET | 获取会话详情 | P1 |
| `/sessions/{id}/end` | POST | 结束会话 | P1 |
| `/knowledge/search` | GET | 搜索知识点 | P2 |
| `/integrations/sync` | POST | 手动同步 | P2 |

### 响应格式

成功响应：

```json
{
  "data": { ... }
}
```

错误响应：

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human readable message"
  }
}
```

---

## 错误码

| Code | HTTP Status | 描述 |
|------|-------------|------|
| `UNAUTHORIZED` | 401 | 未认证 |
| `NOT_FOUND` | 404 | 资源不存在 |
| `INTERNAL_ERROR` | 500 | 服务器错误 |

---

## DTO 定义

核心 DTO 详见代码 `src/api/dto/`。

### WebSocket 消息 DTO

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct WsMessage<T> {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub data: T,
    pub timestamp: DateTime<Utc>,
}
```

### REST 响应 DTO

```rust
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorDetail>,
}
```
