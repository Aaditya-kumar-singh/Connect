# Performance — YBM Connect

| Field             | Value                                      |
|-------------------|--------------------------------------------|
| **Document ID**   | `DOC-PERF-001`                             |
| **Version**       | `1.0.0`                                    |
| **Status**        | `APPROVED`                                 |
| **Owner**         | Engineering Lead                           |
| **Last Updated**  | 2026-10-02                                 |
| **Related Docs**  | DOC-PERF-002 through 009                  |
| **Related Reqs**  | NFR-001 through NFR-007                    |

---

## 1. Performance Targets

### Latency

| Metric | Target | Expected (p50) | Max Acceptable (p95) | Measurement |
|--------|--------|-----------------|---------------------|-------------|
| REST API response | < 100ms | 50ms | 200ms | Server-side histogram |
| WebSocket message delivery (same region) | < 200ms | 100ms | 500ms | Sender→recipient timestamp diff |
| Database query (indexed) | < 20ms | 5ms | 50ms | Query timing middleware |
| Redis operation | < 5ms | 1ms | 10ms | Operation timing |
| Media upload (1MB) | < 2s | 1s | 3s | Client-side timing |
| Media upload (10MB) | < 8s | 4s | 10s | Client-side timing |
| WebSocket connection setup | < 500ms | 200ms | 1000ms | Client-side timing |
| Call setup (offer to connected) | < 3s | 2s | 5s (STUN), 8s (TURN) | Client-side timing |

### Throughput

| Metric | Target (single instance) | Measurement |
|--------|-------------------------|-------------|
| HTTP requests/sec | 2,000 | Load test |
| WebSocket messages/sec | 500 | Load test |
| Concurrent WebSocket connections | 1,000 | Load test |
| Database transactions/sec | 1,000 | Load test |
| Concurrent API users | 500 | Load test |

### Resource Usage

| Metric | Target | Max Acceptable | Measurement |
|--------|--------|---------------|-------------|
| Backend memory (idle) | < 100MB | 256MB | Process metrics |
| Backend memory (1K connections) | < 500MB | 1GB | Load test |
| Backend CPU (idle) | < 5% | 10% | Process metrics |
| Backend CPU (normal load) | < 40% | 70% | Load test |
| Database connections (active) | < 10 | 20 (of pool max) | Pool metrics |

## 2. Performance Testing Strategy

### Load Test Scenarios

#### Scenario 1: Steady State
- **Users:** 100 concurrent, sending 1 message/minute each
- **Duration:** 5 minutes
- **Expected:** p95 delivery < 500ms, 0% error rate
- **Tool:** k6 or custom Rust load test client

#### Scenario 2: Peak Load
- **Users:** 500 concurrent, sending 2 messages/minute each
- **Duration:** 5 minutes
- **Expected:** p95 delivery < 1s, < 0.1% error rate

#### Scenario 3: Burst
- **Users:** Ramp from 0 to 500 in 10 seconds
- **Duration:** 2 minutes
- **Expected:** System stabilizes within 30 seconds, no crashes

#### Scenario 4: Soak Test
- **Users:** 100 concurrent
- **Duration:** 1 hour
- **Expected:** No memory leaks (memory stays stable), no latency degradation

### k6 Test Example

```javascript
import ws from 'k6/ws';
import { check } from 'k6';

export const options = {
  stages: [
    { duration: '1m', target: 100 },
    { duration: '5m', target: 100 },
    { duration: '1m', target: 0 },
  ],
};

export default function () {
  const url = 'wss://api.ybmconnect.com/ws/connect?token=' + getToken();
  
  const res = ws.connect(url, {}, function (socket) {
    socket.on('open', () => {
      socket.send(JSON.stringify({
        type: 'message.send',
        request_id: uuidv4(),
        timestamp: new Date().toISOString(),
        payload: {
          conversation_id: getConversationId(),
          client_message_id: uuidv7(),
          content: 'Load test message',
          content_type: 'text',
        }
      }));
    });
    
    socket.on('message', (msg) => {
      const data = JSON.parse(msg);
      if (data.type === 'message.ack') {
        check(data, {
          'ack received': (d) => d.payload.server_message_id !== undefined,
        });
      }
    });
    
    socket.setTimeout(() => socket.close(), 5000);
  });
  
  check(res, { 'connected': (r) => r && r.status === 101 });
}
```

## 3. Optimization Guidelines

### Database Optimization

| Optimization | When to Apply | How |
|---|---|---|
| Connection pooling | Always (default) | sqlx pool with min/max connections |
| Prepared statements | Always (default) | sqlx prepares queries automatically |
| Index optimization | When p95 query time > 50ms | `EXPLAIN ANALYZE` on slow queries |
| Query result caching | When same query runs > 10x/sec | Cache in Redis with short TTL |
| Read replicas | When read load saturates primary | Configure sqlx to route reads |
| Table partitioning | When messages table > 100M rows | Partition by month on created_at |

### Redis Optimization

| Optimization | When to Apply | How |
|---|---|---|
| Pipeline commands | When sending multiple commands | `redis::pipe()` to batch |
| Lua scripts | When atomicity is needed | EVAL for rate limiting |
| Key expiry | Always | Every key has TTL |
| Memory policy | When memory > 80% | `maxmemory-policy allkeys-lru` |

### WebSocket Optimization

| Optimization | When to Apply | How |
|---|---|---|
| Message compression | When bandwidth is limited | WebSocket per-message deflate |
| Batch delivery | When many messages arrive simultaneously | Batch up to 10 messages per frame |
| Binary frames | When payload size matters | Switch from JSON to MessagePack/CBOR |
| Connection limits | Always | Per-user connection limit (5 devices max) |

## 4. Profiling Tools

| Tool | What It Measures | When to Use |
|------|-----------------|-------------|
| `cargo flamegraph` | CPU time per function | Identify hot functions |
| `tokio-console` | Tokio task activity, polls, wakeups | Debug async performance |
| `EXPLAIN ANALYZE` | PostgreSQL query execution plan | Optimize slow queries |
| `redis-cli --latency` | Redis round-trip latency | Diagnose Redis slowness |
| `criterion` | Rust microbenchmarks | Measure specific function performance |
| Browser DevTools (Network) | HTTP/WS latency | Client-side performance |

---

*Next: [latency.md](latency.md) · [throughput.md](throughput.md) · [optimization.md](optimization.md)*
