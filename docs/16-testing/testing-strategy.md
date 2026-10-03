# Testing Strategy — YBM Connect

| Field             | Value                                      |
|-------------------|--------------------------------------------|
| **Document ID**   | `DOC-TEST-001`                             |
| **Version**       | `1.0.0`                                    |
| **Status**        | `APPROVED`                                 |
| **Owner**         | Engineering Lead                           |
| **Last Updated**  | 2026-10-02                                 |
| **Related Docs**  | DOC-TEST-002 through 012, DOC-DEV-001     |

---

## 1. Testing Pyramid

```
          ╱╲
         ╱  ╲
        ╱ E2E╲          < 5% — Full user flows (Playwright/Detox)
       ╱      ╲
      ╱────────╲
     ╱   Load   ╲       < 5% — Performance validation (k6/Gatling)
    ╱    Stress   ╲
   ╱──────────────╲
  ╱  Integration   ╲    ~30% — API, WebSocket, DB tests (real deps)
 ╱                  ╲
╱────────────────────╲
│     Unit Tests      │  ~60% — Pure logic, no I/O (fast, isolated)
└─────────────────────┘
```

## 2. Testing Layers

### Unit Tests (~60% of total tests)

**What:** Test individual functions and methods in isolation. No database, no Redis, no network.
**Where:** `#[cfg(test)]` modules in each source file, or `tests/` directory.
**Framework:** Rust's built-in `#[test]`, `tokio::test` for async.
**Speed:** < 5 seconds for all unit tests.

**What to unit test:**
- Domain validation (email format, password strength, message length)
- Business logic (can user edit this message? has the edit window expired?)
- State machines (message states, call states)
- Serialization/deserialization of WebSocket events
- Error mapping (domain error → API error code)
- Utility functions (backoff calculation, TURN credential generation)

**What NOT to unit test:**
- Database queries (test in integration)
- HTTP handlers (test in integration)
- External API calls (test in integration with mocks)

### Integration Tests (~30% of total tests)

**What:** Test the interaction between components with real dependencies (PostgreSQL, Redis).
**Where:** `tests/` directory in the backend crate.
**Framework:** `tokio::test`, `sqlx::test` for DB fixtures.
**Dependencies:** Requires PostgreSQL and Redis (via Docker Compose or testcontainers).
**Speed:** < 2 minutes for all integration tests.

**What to integration test:**

| Category | Example Tests |
|----------|---------------|
| **Auth API** | Register, verify OTP, login, refresh token, logout, password reset |
| **Message API** | Send message, receive via WebSocket, delivery receipt, read receipt, edit, delete |
| **Group API** | Create group, add member, remove member, send group message |
| **Media API** | Upload image, generate thumbnail, download via signed URL |
| **WebSocket** | Connect, authenticate, send/receive events, reconnect, heartbeat timeout |
| **Database** | Migration up/down, constraint violations, index usage |
| **Idempotency** | Send same message twice, receive only one copy |
| **Authorization** | Access denied for non-members, cross-user access blocked |

### End-to-End Tests (~5%)

**What:** Test complete user flows through the real UI.
**Where:** `e2e/` directory.
**Framework:** Playwright (web), Detox (mobile).
**Speed:** < 10 minutes.

**Key E2E scenarios:**
1. Register → Verify → Login → Send Message → Receive → Logout
2. Create Group → Add Members → Send Group Message → Leave Group
3. Upload Image → View in Conversation → Download
4. Initiate Audio Call → Answer → End Call → Verify Call History
5. Login on Two Devices → Send from One → Verify on Both

### Load/Stress Tests (~5%)

**What:** Validate performance under expected and extreme load.
**Framework:** k6 or custom Rust test client.

**Load test scenarios:**

| Test | Concurrent Users | Duration | Success Criteria |
|------|-----------------|----------|------------------|
| Steady state | 100 users | 5 min | p95 msg latency < 500ms, 0 errors |
| Peak load | 500 users | 5 min | p95 msg latency < 1s, < 0.1% errors |
| Stress test | 1000 users | 10 min | No crashes, graceful degradation |
| Spike test | 0→500 users in 10s | 2 min | Recovery within 30s |
| Soak test | 100 users | 1 hour | No memory leaks, stable latency |

## 3. Messaging-Specific Test Scenarios

| Scenario | What to Verify |
|----------|---------------|
| **Duplicate send** | Same `client_message_id` sent twice → only one message in DB |
| **Reconnect** | Client disconnects, reconnects → receives missed messages via sync |
| **Offline recipient** | Send to offline user → message persisted → delivered on reconnect |
| **Server restart** | Send messages, restart server → clients reconnect, no message loss |
| **Database failure** | DB goes down → server returns errors, no crash → DB recovers → normal operation |
| **Redis failure** | Redis goes down → messages still persist → no real-time routing → messages delivered on sync |
| **Network interruption** | Cut network mid-message → client retries → idempotency prevents duplicate |
| **Delayed ACK** | Simulate slow DB → ACK arrives late → client doesn't double-send (timeout + retry with same ID) |
| **Duplicate ACK** | Same ACK delivered twice → no side effects |
| **Out-of-order messages** | Messages arrive out of order → client sorts by server timestamp |
| **Concurrent sends** | Two users send simultaneously to same conversation → both messages persisted, correct order |
| **Large message** | Send exactly 4096 chars → accepted. Send 4097 → rejected with clear error |
| **Empty message** | Send empty content → rejected |
| **Message to non-member** | Send to conversation user is not a member of → 403 |
| **Message after leave** | Leave group, then send → 403 |

## 4. Call-Specific Test Scenarios

| Scenario | What to Verify |
|----------|---------------|
| **Call accepted** | Offer → Answer → ICE → Connected → Audio flows → End |
| **Call rejected** | Offer → Reject → Both return to idle, call recorded as REJECTED |
| **Call timeout** | Offer → 30s → Timeout → Call recorded as MISSED |
| **Caller disconnects** | During CONNECTED → Other party notified, call ends |
| **Receiver disconnects** | During CONNECTED → Other party notified, call ends |
| **ICE failure** | All ICE candidates fail → Call status FAILED, clear user message |
| **TURN fallback** | STUN fails → TURN relay used → Call connects (higher latency) |
| **Network switching** | WiFi → Mobile during call → ICE restart → Call recovers |
| **Microphone permission denied** | User denies mic → Call cannot start, clear error message |
| **Camera permission denied** | User denies camera → Video call falls back to audio-only |
| **Double call** | User A calls User B, User C also calls User B → B sees first call, C gets USER_IN_CALL |

## 5. Test Configuration

### Local Development
```bash
# Run unit tests (no dependencies needed)
cargo test --lib

# Run integration tests (requires Docker Compose running)
cargo test --test integration

# Run specific test
cargo test --test integration test_send_message

# Run with output
cargo test -- --nocapture
```

### CI Pipeline
```yaml
test:
  steps:
    - name: Unit tests
      run: cargo test --lib --no-fail-fast
    - name: Start dependencies
      run: docker compose -f docker-compose.test.yml up -d
    - name: Wait for dependencies
      run: ./scripts/wait-for-deps.sh
    - name: Integration tests
      run: cargo test --test integration --no-fail-fast
    - name: Stop dependencies
      run: docker compose -f docker-compose.test.yml down
```

### Test Database
Each integration test runs in a **transaction that is rolled back** after the test. This provides test isolation without the overhead of creating/destroying databases.

```rust
#[sqlx::test]
async fn test_send_message(pool: PgPool) {
    // pool is a transaction-wrapped connection
    // Automatically rolled back after this test
    let service = MessagingService::new(pool.clone());
    let msg = service.send_message(/* ... */).await.unwrap();
    assert_eq!(msg.content, "Hello");
    // No cleanup needed — transaction rolls back
}
```

## 6. Test Coverage Targets

| Module | Target | Rationale |
|--------|--------|-----------|
| Domain (validation, state machines) | 90%+ | Pure logic, easy to test, critical correctness |
| Service layer (business logic) | 80%+ | Core application behavior |
| Repository layer | 70%+ | Database interaction patterns |
| Handlers (REST, WebSocket) | 60%+ | Thin layer, covered by integration tests |
| Infrastructure (adapters) | 50%+ | External dependencies, harder to test |
| Overall | 75%+ | Balanced coverage |

## 7. Security Tests

| Test | Tool | Frequency |
|------|------|-----------|
| Dependency vulnerability scan | `cargo audit` | Every CI run |
| SQL injection | Custom tests + sqlmap | Monthly |
| XSS | Custom tests + OWASP ZAP | Monthly |
| IDOR | Custom integration tests | Every feature |
| Rate limit verification | Custom load tests | Every rate limit change |
| Auth bypass | Custom integration tests | Every auth change |
| File upload validation | Custom tests (malicious files) | Every media change |

---

*Next: [unit-testing.md](unit-testing.md) · [integration-testing.md](integration-testing.md)*
