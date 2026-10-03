# WebSocket Protocol — YBM Connect

| Field             | Value                                      |
|-------------------|--------------------------------------------|
| **Document ID**   | `DOC-API-003`                              |
| **Version**       | `1.0.0`                                    |
| **Status**        | `APPROVED`                                 |
| **Owner**         | Engineering Lead                           |
| **Last Updated**  | 2026-10-02                                 |
| **Related Docs**  | DOC-MSG-001, DOC-ARCH-001, DOC-API-001    |
| **Related Reqs**  | MSG-001 through MSG-009, PRES-001 through PRES-003, CALL-001 through CALL-006 |

---

## 1. Connection Lifecycle

### 1.1 Connection Establishment

```
Client                          Server
  |                               |
  |---- HTTP GET /ws/connect ---->|  (Upgrade: websocket)
  |     Authorization: Bearer {access_token}
  |     X-Device-Id: {device_id}  |
  |                               |
  |<--- 101 Switching Protocols --|  (WebSocket handshake)
  |                               |
  |<--- server.connected --------|  (session info)
  |                               |
  |---- ping -------------------->|
  |<--- pong --------------------|
```

**Authentication:** The access token is provided as a query parameter (`?token=...`) or `Authorization` header during the HTTP upgrade request. The server validates the token, extracts `user_id` and `session_id`, and associates them with the WebSocket connection.

**Device Identification:** `X-Device-Id` header (or query parameter) identifies the specific device. This is required for multi-device message routing.

**Connection Rejection Codes:**
- `4001` — Invalid or expired access token
- `4003` — Account suspended
- `4008` — Device ID missing
- `4029` — Rate limited (too many connections)

### 1.2 Heartbeat

```
Every 30 seconds:
  Client ---- ping ---->  Server
  Server ---- pong ---->  Client

If server receives no ping for 60 seconds:
  Server closes connection with code 4000 (heartbeat timeout)
  Server updates presence to OFFLINE
  Server updates last_seen_at
```

The heartbeat serves two purposes:
1. **Keepalive:** Prevents intermediate proxies (Cloudflare, load balancers) from closing idle connections
2. **Presence:** The server uses heartbeat to determine if a client is truly connected

### 1.3 Reconnection

```
Client detects disconnect:
  Wait: exponential backoff (1s, 2s, 4s, 8s, ..., max 30s) + jitter (0-1s)
  Attempt WebSocket connect with fresh access token
  If access token expired: refresh first, then connect
  On connect success:
    Send conversation.sync with last known state
    Resume normal operation
  On connect failure:
    Increment backoff
    Retry
```

### 1.4 Graceful Disconnect

```
Client ---- close (1000, "logout") ----> Server
Server ---- close (1000, "goodbye") ---> Client
Server: invalidate session, update presence
```

## 2. Message Frame Format

Every WebSocket message is a JSON text frame with this structure:

```json
{
  "type": "message.send",
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2026-10-02T12:00:00.000Z",
  "payload": {}
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | `string` | Yes | Event type identifier. Dot-separated namespace. |
| `request_id` | `string (UUID)` | Yes (client→server) | Unique ID for request-response correlation. |
| `timestamp` | `string (ISO 8601)` | Yes | Event timestamp. Server uses its own clock for authoritative timestamps. |
| `payload` | `object` | Yes | Event-specific data. |

**Server → Client messages** also include `request_id` when they are a response to a client request. Unsolicited server events (e.g., incoming message from another user) use a server-generated `request_id`.

### Error Frame Format

```json
{
  "type": "error",
  "request_id": "...",
  "timestamp": "...",
  "payload": {
    "code": "MESSAGE_TOO_LONG",
    "message": "Message content exceeds 4096 characters",
    "details": {
      "max_length": 4096,
      "actual_length": 5200
    }
  }
}
```

## 3. Event Catalog

### 3.1 Client → Server Events

#### `auth.ping`
**Purpose:** Heartbeat keepalive
**Auth:** Must be authenticated
**Payload:** `{}`
**Response:** `auth.pong`
**Rate limit:** 1 per 20 seconds minimum interval
**Idempotency:** N/A (stateless)

#### `message.send`
**Purpose:** Send a new message to a conversation
**Auth:** Must be authenticated, must be conversation member
**Payload:**
```json
{
  "conversation_id": "uuid",
  "client_message_id": "uuid-v7",
  "content": "Hello, world!",
  "content_type": "text",
  "reply_to_message_id": "uuid | null"
}
```
**Validation:**
- `content`: 1-4096 UTF-8 chars for text, required unless content_type is media
- `content_type`: one of `text`, `image`, `video`, `document`, `voice`
- `client_message_id`: valid UUID, unique per (conversation_id, client_message_id)
- `reply_to_message_id`: must exist in same conversation if provided
**Response:** `message.ack` on success, `error` on failure
**Idempotency:** Yes, on `client_message_id`
**Ordering:** Server timestamp is authoritative
**Example error codes:** `FORBIDDEN`, `CONVERSATION_NOT_FOUND`, `MESSAGE_TOO_LONG`, `INVALID_CONTENT_TYPE`

#### `message.delivered`
**Purpose:** Acknowledge that a message was received by this device
**Auth:** Must be authenticated, must be conversation member
**Payload:**
```json
{
  "message_ids": ["uuid1", "uuid2"]
}
```
**Validation:** All message IDs must exist and belong to conversations the user is a member of
**Response:** None (fire-and-forget from client perspective; server processes asynchronously)
**Idempotency:** Yes (delivering the same message ID twice is a no-op)
**Rate limit:** 60 per minute

#### `message.read`
**Purpose:** Mark messages as read
**Auth:** Must be authenticated, must be conversation member
**Payload:**
```json
{
  "conversation_id": "uuid",
  "last_read_message_id": "uuid"
}
```
**Note:** This marks ALL messages up to and including `last_read_message_id` as read. This is a cursor-based read receipt, not per-message.
**Response:** None
**Idempotency:** Yes (reading the same message twice is a no-op)

#### `message.edit`
**Purpose:** Edit a previously sent message
**Auth:** Must be the original sender
**Payload:**
```json
{
  "message_id": "uuid",
  "content": "Updated content"
}
```
**Validation:** Message must exist, user must be sender, must be within 15-minute edit window
**Response:** `message.edited` broadcast to conversation members
**Error codes:** `FORBIDDEN`, `MESSAGE_NOT_FOUND`, `EDIT_WINDOW_EXPIRED`

#### `message.delete`
**Purpose:** Soft-delete a message
**Auth:** Must be the original sender
**Payload:**
```json
{
  "message_id": "uuid"
}
```
**Response:** `message.deleted` broadcast to conversation members

#### `message.react`
**Purpose:** Add or remove an emoji reaction
**Auth:** Must be conversation member
**Payload:**
```json
{
  "message_id": "uuid",
  "emoji": "👍",
  "action": "add"
}
```
**`action`:** `add` or `remove`
**Response:** `message.reaction` broadcast
**Rate limit:** 20 per minute

#### `typing.start`
**Purpose:** Notify that user is typing in a conversation
**Auth:** Must be conversation member
**Payload:**
```json
{
  "conversation_id": "uuid"
}
```
**Response:** Broadcast `typing.start` to other members
**Expiry:** Server auto-expires after 5 seconds without renewal
**Rate limit:** 1 per 2 seconds per conversation

#### `typing.stop`
**Purpose:** Notify that user stopped typing
**Auth:** Must be conversation member
**Payload:**
```json
{
  "conversation_id": "uuid"
}
```

#### `conversation.sync`
**Purpose:** Request missed messages after reconnection
**Auth:** Must be authenticated
**Payload:**
```json
{
  "conversations": [
    {
      "conversation_id": "uuid",
      "last_message_id": "uuid",
      "last_message_timestamp": "2026-10-02T11:00:00Z"
    }
  ]
}
```
**Response:** `conversation.sync_response` with batched messages per conversation
**Note:** Client sends its last known state for each conversation. Server returns all messages after that point.

#### `call.offer`
**Purpose:** Initiate a call
**Auth:** Must be authenticated
**Payload:**
```json
{
  "callee_id": "uuid",
  "call_type": "audio",
  "sdp_offer": "v=0\r\no=- ..."
}
```
**Response:** `call.ringing` when callee is notified
**Error codes:** `USER_NOT_FOUND`, `USER_BLOCKED`, `USER_IN_CALL`

#### `call.answer`
**Purpose:** Accept an incoming call
**Payload:**
```json
{
  "call_id": "uuid",
  "sdp_answer": "v=0\r\no=- ..."
}
```

#### `call.ice_candidate`
**Purpose:** Exchange ICE candidates for WebRTC connectivity
**Payload:**
```json
{
  "call_id": "uuid",
  "candidate": {
    "candidate": "candidate:...",
    "sdpMid": "0",
    "sdpMLineIndex": 0
  }
}
```
**Note:** Trickle ICE — candidates are sent as they are discovered, not all at once.

#### `call.reject`
**Purpose:** Reject an incoming call
**Payload:**
```json
{
  "call_id": "uuid"
}
```

#### `call.end`
**Purpose:** End an active call
**Payload:**
```json
{
  "call_id": "uuid",
  "reason": "NORMAL"
}
```
**`reason`:** `NORMAL`, `ICE_FAILED`, `TIMEOUT`, `ERROR`

---

### 3.2 Server → Client Events

#### `server.connected`
**Direction:** Server → Client (on connection)
**Payload:**
```json
{
  "session_id": "uuid",
  "user_id": "uuid",
  "device_id": "uuid",
  "server_time": "2026-10-02T12:00:00Z"
}
```
**Purpose:** Confirms connection and provides server time for clock sync.

#### `auth.pong`
**Direction:** Server → Client
**Payload:** `{}`

#### `message.ack`
**Direction:** Server → Client (to sender)
**Payload:**
```json
{
  "client_message_id": "uuid",
  "server_message_id": "uuid",
  "timestamp": "2026-10-02T12:00:00.123Z"
}
```
**Purpose:** Confirms message persistence. Contains the authoritative server timestamp and ID.

#### `message.new`
**Direction:** Server → Client (to recipients)
**Payload:**
```json
{
  "message": {
    "id": "uuid",
    "conversation_id": "uuid",
    "sender_id": "uuid",
    "sender_display_name": "Alice",
    "content": "Hello!",
    "content_type": "text",
    "reply_to": null,
    "attachments": [],
    "created_at": "2026-10-02T12:00:00.123Z"
  }
}
```

#### `message.status`
**Direction:** Server → Client (to sender)
**Payload:**
```json
{
  "message_id": "uuid",
  "status": "delivered",
  "user_id": "uuid",
  "timestamp": "2026-10-02T12:00:01.456Z"
}
```
**`status`:** `delivered` or `read`

#### `message.edited`
**Direction:** Server → Client (to all conversation members)
**Payload:**
```json
{
  "message_id": "uuid",
  "content": "Updated content",
  "edited_at": "2026-10-02T12:01:00Z"
}
```

#### `message.deleted`
**Direction:** Server → Client (to all conversation members)
**Payload:**
```json
{
  "message_id": "uuid",
  "deleted_at": "2026-10-02T12:02:00Z"
}
```

#### `message.reaction`
**Direction:** Server → Client (to all conversation members)
**Payload:**
```json
{
  "message_id": "uuid",
  "user_id": "uuid",
  "emoji": "👍",
  "action": "add"
}
```

#### `typing.start` / `typing.stop`
**Direction:** Server → Client (to other conversation members)
**Payload:**
```json
{
  "conversation_id": "uuid",
  "user_id": "uuid",
  "display_name": "Alice"
}
```

#### `presence.update`
**Direction:** Server → Client (to contacts/conversation members)
**Payload:**
```json
{
  "user_id": "uuid",
  "status": "online",
  "last_seen_at": null
}
```
**`status`:** `online` or `offline`

#### `conversation.sync_response`
**Direction:** Server → Client
**Payload:**
```json
{
  "conversations": [
    {
      "conversation_id": "uuid",
      "messages": [ ... ],
      "has_more": false
    }
  ]
}
```

#### `call.offer` (relayed)
**Direction:** Server → Callee
**Payload:**
```json
{
  "call_id": "uuid",
  "caller_id": "uuid",
  "caller_display_name": "Alice",
  "call_type": "audio",
  "sdp_offer": "v=0\r\n..."
}
```

#### `call.answer` (relayed)
**Direction:** Server → Caller
**Payload:**
```json
{
  "call_id": "uuid",
  "sdp_answer": "v=0\r\n..."
}
```

#### `call.ice_candidate` (relayed)
**Direction:** Server → Other party
**Payload:** Same as client→server version

#### `call.ringing`
**Direction:** Server → Caller
**Payload:**
```json
{
  "call_id": "uuid"
}
```

#### `call.rejected`
**Direction:** Server → Caller
**Payload:**
```json
{
  "call_id": "uuid"
}
```

#### `call.ended`
**Direction:** Server → Both parties
**Payload:**
```json
{
  "call_id": "uuid",
  "reason": "NORMAL",
  "duration_seconds": 180
}
```

#### `call.timeout`
**Direction:** Server → Both parties
**Payload:**
```json
{
  "call_id": "uuid"
}
```

#### `notification`
**Direction:** Server → Client
**Payload:**
```json
{
  "notification_id": "uuid",
  "type": "GROUP_INVITE",
  "title": "Group Invitation",
  "body": "Alice added you to Project Team",
  "data": {
    "group_id": "uuid",
    "conversation_id": "uuid"
  }
}
```

#### `error`
**Direction:** Server → Client
**Payload:**
```json
{
  "code": "FORBIDDEN",
  "message": "You are not a member of this conversation",
  "details": {}
}
```

## 4. Error Codes (WebSocket)

| Code | HTTP Equivalent | Description |
|------|----------------|-------------|
| `INVALID_PAYLOAD` | 400 | Malformed JSON or missing required fields |
| `UNAUTHORIZED` | 401 | Token expired or invalid |
| `FORBIDDEN` | 403 | Not authorized for this action |
| `NOT_FOUND` | 404 | Resource not found |
| `CONFLICT` | 409 | Resource already exists |
| `MESSAGE_TOO_LONG` | 422 | Content exceeds 4096 chars |
| `INVALID_CONTENT_TYPE` | 422 | Unsupported content type |
| `EDIT_WINDOW_EXPIRED` | 422 | Message older than 15 minutes |
| `RATE_LIMITED` | 429 | Too many requests |
| `USER_BLOCKED` | 403 | Target user has blocked sender |
| `USER_IN_CALL` | 409 | Target user is already in a call |
| `SERVICE_UNAVAILABLE` | 503 | Backend service temporarily unavailable |
| `INTERNAL_ERROR` | 500 | Unexpected server error |

## 5. Rate Limits (WebSocket)

| Event | Limit | Window |
|-------|-------|--------|
| `message.send` | 30 | per minute |
| `message.edit` | 10 | per minute |
| `message.delete` | 10 | per minute |
| `message.react` | 20 | per minute |
| `message.delivered` | 60 | per minute |
| `message.read` | 30 | per minute |
| `typing.start` | 30 | per minute |
| `call.offer` | 5 | per minute |
| `conversation.sync` | 5 | per minute |
| Total events | 120 | per minute |

---

*Next: [REST.md](REST.md) · [authentication-api.md](authentication-api.md)*
