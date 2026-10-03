# Scalability — YBM Connect

| Field             | Value                                      |
|-------------------|--------------------------------------------|
| **Document ID**   | `DOC-ARCH-010`                             |
| **Version**       | `1.0.0`                                    |
| **Status**        | `APPROVED`                                 |
| **Owner**         | Engineering Lead                           |
| **Last Updated**  | 2026-10-02                                 |
| **Related Docs**  | DOC-ARCH-001, DOC-ARCH-009                 |
| **Related ADRs**  | ADR-011, ADR-004, ADR-005                  |

---

## Scaling Strategy

YBM Connect is designed with a **scale-up first, scale-out when needed** approach. The modular monolith runs on a single instance initially, and the architecture allows horizontal scaling without code changes when the time comes.

### Phase 1: Single Instance (Current Target)

```
Client ─── Cloudflare ─── Axum (1 instance) ─── PostgreSQL (1 instance)
                                              └── Redis (1 instance)
```

**Capacity:** ~1,000 concurrent WebSocket connections, ~500 messages/sec.
**Bottleneck:** CPU and memory on the single Axum instance.
**Scaling trigger:** Sustained CPU > 70% or memory > 80% or WebSocket connections > 800.

### Phase 2: Horizontal Backend Scaling

```
                         ┌── Axum Instance 1 ──┐
Client ── Cloudflare ──  ├── Axum Instance 2 ──├── PostgreSQL (primary + read replica)
                         └── Axum Instance N ──┘     └── Redis (single or Sentinel)
```

**Changes required:**
1. Load balancer in front of Axum instances (Cloudflare or HAProxy)
2. Redis pub/sub for cross-instance WebSocket message routing (already designed)
3. Sticky sessions for WebSocket connections (or Redis-based session lookup)
4. Shared session storage (already in PostgreSQL)

**No code changes needed** — the Redis pub/sub routing is built into the message delivery path from day one.

### Phase 3: Database Scaling

| Strategy | When | Complexity |
|----------|------|------------|
| **Connection pooling** (PgBouncer) | Connection count > PostgreSQL max_connections | Low |
| **Read replicas** | Read-heavy queries (conversation history, search) saturate primary | Medium |
| **Table partitioning** (messages by date) | Messages table > 100M rows, query latency increases | Medium |
| **Vertical scaling** | Before horizontal; larger instance | Low |
| **Sharding** | When single-primary write throughput is exceeded | High (avoid as long as possible) |

### Scaling Bottleneck Analysis

| Component | Scaling Dimension | Limit | Strategy |
|-----------|------------------|-------|----------|
| WebSocket connections | Memory (per-connection state) | ~10K per instance (OS file descriptors, memory) | Add instances |
| Message throughput | CPU (validation, DB writes) | ~500 msg/sec per instance | Add instances, batch DB writes |
| Database writes | Disk I/O, transaction throughput | ~5K TPS (standard PostgreSQL) | PgBouncer, larger instance, partitioning |
| Database reads | CPU, memory (buffer cache) | Read replicas for read-heavy | Read replicas |
| Redis pub/sub | Network, CPU | ~100K msg/sec | Redis Cluster (rarely needed) |
| Media storage (R2) | Cloudflare-managed | Effectively unlimited | Cloudflare manages |
| TURN bandwidth | Network I/O | Depends on server bandwidth | Add TURN instances, use TURN pools |

## What We Do NOT Over-Engineer

1. **No sharding** — Single PostgreSQL instance handles the expected scale. Partitioning the messages table by month is the first optimization if needed.
2. **No Redis Cluster** — Single Redis instance with Sentinel for failover is sufficient.
3. **No message queue** — Redis pub/sub replaces the need for a dedicated queue (Kafka, RabbitMQ) at this scale.
4. **No service mesh** — Single binary; no inter-service network to manage.
5. **No multi-region** — Single-region deployment initially.

---

*Next: [architecture-decisions/](architecture-decisions/) · [data-flow.md](data-flow.md)*
