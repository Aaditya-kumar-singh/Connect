# Observability — Logging — YBM Connect

| Field             | Value                                      |
|-------------------|--------------------------------------------|
| **Document ID**   | `DOC-OBS-001`                              |
| **Version**       | `1.0.0`                                    |
| **Status**        | `APPROVED`                                 |
| **Owner**         | Engineering Lead                           |
| **Last Updated**  | 2026-10-04                                 |
| **Related Docs**  | DOC-OBS-002 through 007, DOC-DEV-001      |
| **Related Reqs**  | NFR-021 through NFR-024                    |

---

## 1. Structured Logging

All logs are emitted as JSON using the `tracing` + `tracing-subscriber` crates.

### Log Entry Format

```json
{
  "timestamp": "2026-10-02T12:00:00.123456Z",
  "level": "INFO",
  "target": "ybm_connect::messaging::service",
  "message": "Message sent successfully",
  "request_id": "550e8400-e29b-41d4-a716-446655440000",
  "user_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "conversation_id": "c0c0c0c0-d1d1-e2e2-f3f3-a4a4a4a4a4a4",
  "message_id": "m1m1m1m1-n2n2-o3o3-p4p4-q5q5q5q5q5q5",
  "duration_ms": 12
}
```

### Required Fields (All Log Entries)

| Field | Type | Source | Description |
|-------|------|--------|-------------|
| `timestamp` | ISO 8601 | tracing-subscriber | When the event occurred |
| `level` | string | tracing macro | TRACE, DEBUG, INFO, WARN, ERROR |
| `target` | string | Rust module path | Which module emitted the log |
| `message` | string | Log message | Human-readable description |

### Context Fields (When Available)

| Field | When Present | Description |
|-------|-------------|-------------|
| `request_id` | Every HTTP/WS request | UUID v4, assigned by middleware |
| `user_id` | After authentication | Authenticated user |
| `device_id` | After authentication | Specific device |
| `session_id` | After authentication | Active session |
| `conversation_id` | Messaging/group operations | Target conversation |
| `message_id` | Message operations | Server message ID |
| `call_id` | Call operations | Active call |
| `error_code` | Error events | Typed error code |
| `duration_ms` | Operation timing | How long the operation took |
| `status_code` | HTTP responses | HTTP status code |

### NEVER Log

| Data | Why |
|------|-----|
| Passwords | Credential exposure |
| Password hashes | Hash exposure |
| OTP codes | OTP bypass |
| Access tokens | Session hijacking |
| Refresh tokens | Session hijacking |
| Private keys | System compromise |
| Full message content | Privacy violation |
| Email addresses in bulk | Privacy violation |
| Full request bodies containing auth | Token leakage |

**Exception:** In DEBUG level during local development, additional context may be logged. But NEVER in production (`RUST_LOG=ybm_connect=info` in production).

### Log Levels

| Level | Usage | Example |
|-------|-------|---------|
| **ERROR** | Something failed that shouldn't. Requires attention. | Database connection lost, worker panic, auth bypass attempt |
| **WARN** | Something unexpected but handled. | Rate limit hit, retry attempt, deprecated API called |
| **INFO** | Normal operational events. | Server started, user logged in, message sent, call connected |
| **DEBUG** | Detailed information for debugging. | Query parameters, cache hit/miss, WebSocket frame received |
| **TRACE** | Very detailed, high-volume. | Every Redis command, every SQL query text, every WebSocket ping |

## 2. Metrics

### Prometheus-Compatible Metrics

Exposed at `GET /metrics` in Prometheus text format. The endpoint is internal and requires `Authorization: Bearer <METRICS_AUTH_TOKEN>`. Phase 16 currently implements the HTTP, WebSocket, and PostgreSQL pool metrics listed below. Messaging, Redis-operation, and call-specific metrics remain future instrumentation work and are not exposed yet.

#### HTTP Metrics

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `http_requests_total` | Counter | `method`, `path`, `status` | Total HTTP requests |
| `http_request_duration_seconds` | Histogram | `method`, `path` | Request latency distribution |
| `http_request_size_bytes` | Histogram | `method`, `path` | Request body size |
| `http_response_size_bytes` | Histogram | `method`, `path` | Response body size |

#### WebSocket Metrics

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `ws_connections_active` | Gauge | — | Current active WebSocket connections |
| `ws_connections_total` | Counter | — | Total connections since start |
| `ws_messages_received_total` | Counter | `type` | Messages received from clients |
| `ws_messages_sent_total` | Counter | `type` | Messages sent to clients |
| `ws_message_processing_seconds` | Histogram | `type` | Event processing duration |

#### Messaging Metrics

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `messages_sent_total` | Counter | `content_type` | Total messages sent |
| `messages_delivered_total` | Counter | — | Delivery receipts recorded |
| `messages_read_total` | Counter | — | Read receipts recorded |
| `message_delivery_latency_seconds` | Histogram | — | Sender → recipient delivery time |

#### Database Metrics

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `db_pool_connections_active` | Gauge | — | Active DB connections |
| `db_pool_connections_idle` | Gauge | — | Idle DB connections |
| `db_pool_connections_max` | Gauge | — | Max pool size |
| `db_query_duration_seconds` | Histogram | `query_name` | Query latency |
| `db_query_errors_total` | Counter | `query_name` | Query errors |

#### Redis Metrics

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `redis_operations_total` | Counter | `operation` | Redis commands executed |
| `redis_operation_duration_seconds` | Histogram | `operation` | Redis command latency |
| `redis_errors_total` | Counter | `operation` | Redis command errors |

#### Call Metrics

| Metric | Type | Labels | Description |
|--------|------|--------|-------------|
| `calls_initiated_total` | Counter | `type` (audio/video) | Calls started |
| `calls_connected_total` | Counter | `type` | Calls that reached CONNECTED |
| `calls_failed_total` | Counter | `reason` | Calls that failed |
| `call_duration_seconds` | Histogram | `type` | Call duration distribution |

## 3. Health Checks

### Liveness: `GET /health`

Returns 200 if the process is running. No dependency checks.

```json
{ "status": "ok", "uptime_seconds": 86400 }
```

### Readiness: `GET /ready`

Returns 200 only if all dependencies are accessible.

```json
{
  "status": "ready",
  "checks": {
    "postgresql": { "status": "ok", "latency_ms": 2 },
    "redis": { "status": "ok", "latency_ms": 1 }
  }
}
```

If any check fails:

```json
{
  "status": "not_ready",
  "checks": {
    "postgresql": { "status": "ok", "latency_ms": 2 },
    "redis": { "status": "error", "error": "Connection refused" }
  }
}
```

Status code: 503.

## 4. Alerts

| Alert | Condition | Severity | Action |
|-------|-----------|----------|--------|
| **High error rate** | `http_requests_total{status=~"5.."}` > 5% of total in 5 min | CRITICAL | Page on-call, check logs |
| **High latency** | `http_request_duration_seconds` p95 > 2s for 5 min | HIGH | Check DB, Redis, CPU |
| **WebSocket connection spike** | `ws_connections_active` > 90% of limit | HIGH | Prepare to scale |
| **DB connection exhaustion** | `db_pool_connections_active` > 90% of max | CRITICAL | Increase pool, check leaks |
| **Redis unavailable** | Redis health check fails for > 30s | HIGH | Check Redis, switch to degraded mode |
| **Message delivery delay** | `message_delivery_latency_seconds` p95 > 5s | HIGH | Check Redis pub/sub, backend load |
| **Worker crash loop** | Same worker restarts > 5 times in 5 min | HIGH | Check worker logs, disable if needed |
| **Disk space low** | PostgreSQL disk > 85% | HIGH | Cleanup, expand disk |
| **Certificate expiry** | TLS cert expires in < 14 days | MEDIUM | Renew via Cloudflare |

## 5. Tracing

Distributed tracing using `tracing` crate with OpenTelemetry export (optional).

Each request creates a **span** that propagates through:
1. HTTP/WebSocket handler
2. Service layer
3. Repository layer
4. Database query
5. Redis operation

```
[request span: POST /api/v1/messages/forward]
├── [auth middleware: validate_token]
├── [service: forward_message]
│   ├── [repository: find_message]
│   │   └── [db: SELECT * FROM messages WHERE id = $1]
│   ├── [repository: check_membership]
│   │   └── [db: SELECT * FROM conversation_members WHERE ...]
│   ├── [repository: insert_message]
│   │   └── [db: INSERT INTO messages ...]
│   └── [redis: PUBLISH conversation:xxx]
└── [response: 201 Created, 45ms]
```

---

*Next: [metrics.md](metrics.md) · [tracing.md](tracing.md) · [alerts.md](alerts.md)*
