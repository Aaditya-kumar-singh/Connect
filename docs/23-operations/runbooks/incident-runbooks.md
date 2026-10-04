# Incident Runbooks — YBM Connect

| Field             | Value                                      |
|-------------------|--------------------------------------------|
| **Document ID**   | `DOC-OPS-001`                              |
| **Version**       | `1.0.0`                                    |
| **Status**        | `APPROVED`                                 |
| **Owner**         | Engineering Lead                           |
| **Last Updated**  | 2026-10-02                                 |

---

## RB-001: API Unavailable

### Symptoms
- HTTP requests to `/health` or `/ready` return errors or timeout
- Users report inability to login, send messages, or load the app
- Monitoring shows 0 successful responses

### Detection
- Health check monitoring alerts
- Error rate metric > 99% for 2 minutes
- Cloudflare alerts origin is down

### Immediate Actions
1. Check if Axum process is running: `systemctl status ybm-connect` (or Docker: `docker ps`)
2. If not running: restart immediately: `systemctl restart ybm-connect`
3. If running but unresponsive: check CPU and memory: `top` / `htop`

### Diagnosis
```bash
# Check process status
systemctl status ybm-connect

# Check recent logs
journalctl -u ybm-connect --since "10 minutes ago" | tail -100

# Check if port is listening
ss -tlnp | grep 8080

# Check database connectivity
psql -h localhost -U ybm -d ybm_connect -c "SELECT 1"

# Check Redis connectivity
redis-cli ping

# Check file descriptors (WebSocket connections use FDs)
ls /proc/$(pgrep ybm-connect)/fd | wc -l
```

### Likely Causes
1. **Process crashed** — Check logs for panic or OOM
2. **Port conflict** — Another process using port 8080
3. **Database unavailable** — Readiness check fails
4. **Redis unavailable** — Readiness check fails
5. **Out of memory** — OOM killer terminated process
6. **File descriptor exhaustion** — Too many WebSocket connections

### Resolution
| Cause | Action |
|-------|--------|
| Process crashed | Restart, check logs for root cause |
| OOM | Increase memory limit, check for memory leaks |
| DB unavailable | See RB-005 |
| Redis unavailable | See RB-006 |
| FD exhaustion | Increase ulimit, check for connection leaks |

### Recovery Verification
1. `/health` returns 200
2. `/ready` returns 200 with all checks OK
3. Test login flow manually
4. Test sending a message
5. Check metrics show normal error rate using the configured `METRICS_AUTH_TOKEN`

### Post-Incident
- Add root cause to post-mortem document
- If OOM: review memory allocation patterns
- If crash: add crash-specific monitoring

---

## RB-002: Messages Delayed

### Symptoms
- Users report messages taking > 5 seconds to appear
- `message_delivery_latency_seconds` p95 > 5s
- Messages eventually arrive (not lost)

### Detection
- Alert: message delivery latency p95 > 5s for 5 minutes

### Immediate Actions
1. Check Redis connectivity and latency
2. Check PostgreSQL query latency
3. Check WebSocket connection count (is it near the limit?)

### Diagnosis
```bash
# Check Redis latency
redis-cli --latency

# Check Redis pub/sub subscriptions
redis-cli PUBSUB CHANNELS 'conversation:*' | wc -l

# Check PostgreSQL slow queries
psql -c "SELECT pid, now() - pg_stat_activity.query_start AS duration, query 
         FROM pg_stat_activity 
         WHERE state != 'idle' 
         ORDER BY duration DESC 
         LIMIT 10;"

# Check backend CPU and message processing rate
curl localhost:8080/metrics | grep message_processing
```

### Likely Causes
1. **Redis latency** — Redis overloaded or swapping
2. **Database slow queries** — Missing index, lock contention
3. **Backend CPU saturation** — Too many messages being processed
4. **Network latency** — Between backend and Redis/PostgreSQL
5. **Message worker backlog** — Bounded channel is full

### Resolution
| Cause | Action |
|-------|--------|
| Redis latency | Check Redis memory usage, eviction policy |
| Slow DB queries | Identify query with `EXPLAIN ANALYZE`, add missing index |
| CPU saturation | Check for CPU-bound work on async threads, scale up |
| Worker backlog | Increase channel capacity, add more worker tasks |

---

## RB-003: Messages Missing

### Symptoms
- Users report sent messages not appearing for recipient
- Sender sees ACK (checkmark) but recipient never receives

### Detection
- User reports
- `messages_sent_total` increasing but `messages_delivered_total` not matching

### Diagnosis
```bash
# Check if message exists in database
psql -c "SELECT id, conversation_id, sender_id, content_type, created_at 
         FROM messages 
         WHERE id = '{message_id}';"

# Check if receipt exists
psql -c "SELECT * FROM message_receipts WHERE message_id = '{message_id}';"

# Check if recipient is connected
redis-cli SMEMBERS "connections:{recipient_user_id}"

# Check if pub/sub event was published (check logs)
grep "{message_id}" /var/log/ybm-connect/*.log
```

### Resolution
- **Message in DB, no delivery receipt:** Redis pub/sub may have failed. Recipient will receive on next sync.
- **Message not in DB:** Investigate why ACK was sent without persistence (CRITICAL bug).
- **Recipient not connected:** Push notification should have been sent. Check notification logs.

---

## RB-005: Database Unavailable

### Symptoms
- `/ready` returns 503 with PostgreSQL check failed
- All state-changing operations fail
- Login fails, message send fails

### Immediate Actions
1. Check PostgreSQL process: `systemctl status postgresql`
2. Check disk space: `df -h`
3. Check PostgreSQL logs: `tail -100 /var/log/postgresql/postgresql-16-main.log`
4. If stopped: restart: `systemctl restart postgresql`

### Diagnosis
```bash
# Check if PostgreSQL is accepting connections
pg_isready -h localhost -U ybm

# Check connection count
psql -c "SELECT count(*) FROM pg_stat_activity;"

# Check for locks
psql -c "SELECT pid, mode, relation::regclass, page, tuple 
         FROM pg_locks 
         WHERE NOT granted;"

# Check replication lag (if using replicas)
psql -c "SELECT pg_last_wal_receive_lsn() - pg_last_wal_replay_lsn() AS lag;"
```

### Likely Causes
1. **Process crashed** — OOM, disk full, configuration error
2. **Connection exhaustion** — max_connections reached
3. **Disk full** — WAL or data directory full
4. **Lock contention** — Long-running transaction blocking others
5. **Network issue** — Backend cannot reach database host

### Recovery
1. Restart PostgreSQL if crashed
2. If connection exhaustion: kill idle connections, increase max_connections, add PgBouncer
3. If disk full: emergency cleanup of WAL files, expand disk
4. After recovery: backend connection pool reconnects automatically

---

## RB-006: Redis Unavailable

### Symptoms
- `/ready` returns 503 with Redis check failed
- Messages still persist (PostgreSQL works) but no real-time delivery
- Presence shows stale data
- Typing indicators don't work

### Immediate Actions
1. Check Redis process: `systemctl status redis`
2. Restart if stopped: `systemctl restart redis`
3. System enters degraded mode automatically

### Diagnosis
```bash
redis-cli ping
redis-cli INFO memory
redis-cli INFO clients
```

### Recovery
1. Restart Redis
2. Backend connection pool reconnects automatically (with backoff)
3. Presence rebuilds from active WebSocket connections
4. Rate limit counters reset
5. Clients receive missed messages on next conversation sync

---

## RB-010: Deployment Rollback

### Symptoms
- New deployment introduced a regression
- Error rate spiked after deployment
- Users report broken functionality

### Immediate Actions
1. **Rollback backend:**
   ```bash
   # If using containers:
   docker pull ybm-connect:{previous_version}
   docker stop ybm-connect
   docker run -d --name ybm-connect ybm-connect:{previous_version}
   
   # If using systemd:
   systemctl stop ybm-connect
   cp /opt/ybm-connect/bin/ybm-connect.backup /opt/ybm-connect/bin/ybm-connect
   systemctl start ybm-connect
   ```

2. **Rollback database migration (if applicable):**
   ```bash
   sqlx migrate revert --source backend/migrations
   ```
   ⚠️ Only if the migration has a DOWN script and is safe to revert.

3. **Rollback frontend:**
   ```bash
   # Revert to previous deployment
   # If using Cloudflare Pages: use the Cloudflare dashboard to rollback to previous deployment
   ```

### Recovery Verification
1. Error rate returns to pre-deployment levels
2. `/health` and `/ready` return 200
3. Manual test of affected functionality
4. Monitor for 15 minutes before declaring stable

### Post-Incident
- Root cause analysis on what the deployment broke
- Add regression test for the specific failure
- Review deployment checklist

---

*Each runbook follows the same structure: Symptoms → Detection → Immediate Actions → Diagnosis → Likely Causes → Resolution → Recovery Verification → Post-Incident*

---

*See also: [common-errors.md](common-errors.md) · [troubleshooting/](troubleshooting/)*
