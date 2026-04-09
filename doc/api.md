# NightMind API 设计

## 概述

NightMind 提供两类 API：
- **WebSocket API**: 实时双向通信（主要交互方式）
- **REST API**: 资源管理、配置、查询

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
| `audio_chunk` | 音频数据 | `{format, sample_rate, encoding, payload}` |
| `text_input` | 文本输入 | `{text}` |
| `command` | 控制命令 | `{action: pause\|resume\|end}` |
| `ping` | 心跳 | `{timestamp}` |

### 服务端 → 客户端

| 消息类型 | 描述 | 数据格式 |
|----------|------|----------|
| `tts_audio` | TTS 音频 | `{format, encoding, payload, text?}` |
| `content_transform` | 内容转换状态 | `{transformed, confidence, reading_time}` |
| `state_update` | 会话状态 | `{phase, elapsed_seconds, current_topic}` |
| `haptic` | 触觉反馈 | `{pattern, intensity}` |
| `error` | 错误 | `{code, message}` |
| `pong` | 心跳响应 | `{timestamp, server_time}` |

### 消息协议

所有消息均为 JSON 格式：

```json
{
  "type": "message_type",
  "data": { ... },
  "id": "optional_message_id",
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

| 端点 | 方法 | 描述 |
|------|------|------|
| `/auth/register` | POST | 用户注册 |
| `/auth/login` | POST | 用户登录 |
| `/users/me` | GET | 获取当前用户 |
| `/users/me/preferences` | PATCH | 更新偏好 |
| `/sessions` | POST | 创建会话 |
| `/sessions` | GET | 获取会话列表 |
| `/sessions/{id}` | GET | 获取会话详情 |
| `/sessions/{id}/end` | POST | 结束会话 |
| `/sessions/{id}/restore` | POST | 恢复会话 |
| `/knowledge` | POST | 创建知识点 |
| `/knowledge/search` | GET | 搜索知识点 |
| `/knowledge/due` | GET | 获取待复习内容 |
| `/knowledge/{id}/review` | POST | 提交复习结果 |

### 响应格式

成功响应：

```json
{
  "data": { ... },
  "meta": {
    "page": 1,
    "per_page": 20,
    "total": 100
  }
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
| `FORBIDDEN` | 403 | 无权限 |
| `NOT_FOUND` | 404 | 资源不存在 |
| `RATE_LIMIT_EXCEEDED` | 429 | 超出速率限制 |
| `INTERNAL_ERROR` | 500 | 服务器错误 |
| `SERVICE_UNAVAILABLE` | 503 | 服务不可用 |

---

## 速率限制

| Endpoint | 限制 | 窗口 |
|----------|------|------|
| WebSocket 连接 | 5 连接/用户 | 1 分钟 |
| REST API | 100 请求/用户 | 1 分钟 |
| 创建会话 | 10 会话/用户 | 1 小时 |

响应头：

```http
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1705334460
```

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
    pub id: Option<String>,
    pub timestamp: DateTime<Utc>,
}
```

### REST 响应 DTO

```rust
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<PaginationMeta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorDetail>,
}
```
