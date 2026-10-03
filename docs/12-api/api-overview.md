# REST API Overview — YBM Connect

| Field             | Value                                      |
|-------------------|--------------------------------------------|
| **Document ID**   | `DOC-API-001`                              |
| **Version**       | `1.0.0`                                    |
| **Status**        | `APPROVED`                                 |
| **Owner**         | Engineering Lead                           |
| **Last Updated**  | 2026-10-02                                 |
| **Related Docs**  | DOC-API-002 through 013                    |

---

## 1. Base URL

```
Production:  https://api.ybmconnect.com/api/v1
Development: http://localhost:8080/api/v1
```

## 2. Common Response Format

### Success Response

```json
{
  "success": true,
  "data": { ... },
  "error": null,
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2026-10-02T12:00:00Z"
}
```

### Error Response

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Email format is invalid",
    "details": {
      "field": "email",
      "constraint": "Must be a valid email address"
    }
  },
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2026-10-02T12:00:00Z"
}
```

### Paginated Response

```json
{
  "success": true,
  "data": {
    "items": [ ... ],
    "cursor": "eyJpZCI6IjEyMyJ9",
    "has_more": true
  },
  "error": null,
  "request_id": "...",
  "timestamp": "..."
}
```

Pagination uses cursor-based pagination (not offset). The `cursor` is an opaque string (base64-encoded position marker). The client sends it back as `?cursor=...` to get the next page.

## 3. Common Headers

### Request Headers

| Header | Required | Description |
|--------|----------|-------------|
| `Authorization` | Yes (authenticated endpoints) | `Bearer {access_token}` |
| `Content-Type` | Yes (POST/PATCH/PUT) | `application/json` (or `multipart/form-data` for uploads) |
| `X-Request-ID` | Optional | Client-generated request ID. If provided, used instead of server-generated. |
| `X-Device-ID` | Yes (authenticated endpoints) | Device identifier |

### Response Headers

| Header | Description |
|--------|-------------|
| `X-Request-ID` | Request ID (echoed or server-generated) |
| `X-RateLimit-Limit` | Max requests in current window |
| `X-RateLimit-Remaining` | Remaining requests in current window |
| `X-RateLimit-Reset` | Unix timestamp when window resets |
| `Content-Type` | `application/json` |

## 4. Endpoint Catalog

### Authentication (`/api/v1/auth/`)

| Method | Path | Auth | Description | Rate Limit |
|--------|------|------|-------------|------------|
| `POST` | `/auth/register` | No | Create account | 5/hour/IP |
| `POST` | `/auth/verify-email` | No | Verify email OTP | 5 attempts/OTP |
| `POST` | `/auth/login` | No | Login | 10/15min/email |
| `POST` | `/auth/refresh` | Refresh Token | Refresh access token | 10/min |
| `POST` | `/auth/logout` | Yes | Logout current device | 10/min |
| `POST` | `/auth/logout-all` | Yes | Logout all devices | 5/min |
| `POST` | `/auth/forgot-password` | No | Request password reset | 3/hour/email |
| `POST` | `/auth/reset-password` | No | Reset with OTP | 5 attempts/OTP |
| `POST` | `/auth/resend-otp` | No | Resend verification OTP | 3/hour/email |

### Users (`/api/v1/users/`)

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/users/me` | Yes | Get current user profile |
| `PATCH` | `/users/me` | Yes | Update current user profile |
| `GET` | `/users/{id}` | Yes | Get another user's public profile |
| `GET` | `/users/search?q={query}` | Yes | Search users by display name or email |

### Devices & Sessions (`/api/v1/devices/`, `/api/v1/sessions/`)

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/devices` | Yes | List user's devices |
| `PATCH` | `/devices/{id}` | Yes | Update device push token |
| `DELETE` | `/devices/{id}` | Yes | Remove device (revokes session) |
| `GET` | `/sessions` | Yes | List active sessions |
| `DELETE` | `/sessions/{id}` | Yes | Revoke a specific session |

### Conversations (`/api/v1/conversations/`)

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `POST` | `/conversations` | Yes | Create 1-to-1 conversation |
| `GET` | `/conversations` | Yes | List conversations (paginated) |
| `GET` | `/conversations/{id}` | Yes | Get conversation details |
| `GET` | `/conversations/{id}/messages` | Yes | Get message history (paginated) |

### Messages (`/api/v1/messages/`)

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `POST` | `/messages/forward` | Yes | Forward message to another conversation |

**Note:** Most message operations (send, edit, delete, react, reply) use WebSocket events, not REST endpoints. REST is used only for operations that cross conversation boundaries (forwarding) or require multipart uploads (media).

### Groups (`/api/v1/groups/`)

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `POST` | `/groups` | Yes | Create group |
| `GET` | `/groups/{id}` | Yes | Get group details |
| `PATCH` | `/groups/{id}` | Yes | Update group (admin only) |
| `DELETE` | `/groups/{id}` | Yes | Delete group (admin only) |
| `POST` | `/groups/{id}/members` | Yes | Add members (admin only) |
| `DELETE` | `/groups/{id}/members/{user_id}` | Yes | Remove member (admin only) |
| `POST` | `/groups/{id}/leave` | Yes | Leave group |
| `PATCH` | `/groups/{id}/members/{user_id}/role` | Yes | Change member role (admin only) |

### Media (`/api/v1/media/`)

| Method | Path | Auth | Content-Type | Description |
|--------|------|------|--------------|-------------|
| `POST` | `/media/upload` | Yes | `multipart/form-data` | Upload media file |
| `GET` | `/media/{id}/url` | Yes | — | Get signed download URL |

### Calls (`/api/v1/calls/`)

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/calls/history` | Yes | Get call history (paginated) |
| `GET` | `/calls/{id}` | Yes | Get call details |

### Contacts & Blocks (`/api/v1/contacts/`, `/api/v1/blocks/`)

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/contacts` | Yes | List contacts |
| `POST` | `/contacts` | Yes | Add contact |
| `DELETE` | `/contacts/{user_id}` | Yes | Remove contact |
| `GET` | `/blocks` | Yes | List blocked users |
| `POST` | `/blocks` | Yes | Block user |
| `DELETE` | `/blocks/{user_id}` | Yes | Unblock user |

### System (`/api/v1/`)

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/health` | No | Liveness check |
| `GET` | `/ready` | No | Readiness check |
| `GET` | `/metrics` | Internal | Prometheus metrics |

## 5. Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `VALIDATION_ERROR` | 400 | Input validation failed |
| `INVALID_PAYLOAD` | 400 | Malformed request body |
| `UNAUTHORIZED` | 401 | Missing or invalid auth token |
| `TOKEN_EXPIRED` | 401 | Access token has expired |
| `FORBIDDEN` | 403 | Not authorized for this action |
| `NOT_FOUND` | 404 | Resource not found |
| `EMAIL_ALREADY_EXISTS` | 409 | Email already registered |
| `CONVERSATION_EXISTS` | 409 | Direct conversation already exists |
| `ALREADY_MEMBER` | 409 | User is already a group member |
| `INVALID_OTP` | 401 | OTP is incorrect |
| `OTP_EXPIRED` | 410 | OTP has expired |
| `EDIT_WINDOW_EXPIRED` | 422 | Message edit window (15 min) has passed |
| `INVALID_FILE_TYPE` | 422 | Uploaded file type not allowed |
| `FILE_TOO_LARGE` | 422 | Uploaded file exceeds size limit |
| `ACCOUNT_LOCKED` | 423 | Account temporarily locked |
| `RATE_LIMITED` | 429 | Too many requests |
| `INTERNAL_ERROR` | 500 | Unexpected server error |
| `SERVICE_UNAVAILABLE` | 503 | Dependency unavailable |

## 6. API Versioning

- Version is in the URL path: `/api/v1/`, `/api/v2/`
- Breaking changes increment the version number
- Breaking changes: removing fields, changing field types, changing semantics, removing endpoints
- Non-breaking changes (safe to add without version bump): adding optional fields, adding new endpoints, adding new error codes
- The previous API version is supported for at least **one release cycle** (minimum 30 days)
- Deprecated versions return a `Deprecation` header: `Deprecation: true`

## 7. Example: Register Endpoint

```
POST /api/v1/auth/register

Request:
  Content-Type: application/json
  
  {
    "email": "alice@example.com",
    "password": "SecureP@ss1",
    "display_name": "Alice"
  }

Validation:
  - email: valid RFC 5322 format, max 255 chars, not already registered
  - password: 8-128 chars, >= 1 uppercase, >= 1 lowercase, >= 1 digit
  - display_name: 2-50 chars, printable characters only

Success Response (201 Created):
  {
    "success": true,
    "data": {
      "user_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
      "email": "alice@example.com",
      "status": "PENDING_VERIFICATION",
      "message": "Verification email sent. Please check your inbox."
    },
    "error": null,
    "request_id": "...",
    "timestamp": "..."
  }

Error Response (409 Conflict):
  {
    "success": false,
    "data": null,
    "error": {
      "code": "EMAIL_ALREADY_EXISTS",
      "message": "An account with this email already exists",
      "details": {}
    },
    "request_id": "...",
    "timestamp": "..."
  }

Error Response (422 Unprocessable Entity):
  {
    "success": false,
    "data": null,
    "error": {
      "code": "VALIDATION_ERROR",
      "message": "Password must contain at least one uppercase letter",
      "details": {
        "field": "password",
        "constraint": "uppercase_required"
      }
    },
    "request_id": "...",
    "timestamp": "..."
  }
```

---

*Next: [authentication-api.md](authentication-api.md) · [websocket-protocol.md](websocket-protocol.md)*
