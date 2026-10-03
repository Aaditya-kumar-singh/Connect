# Data Flow — YBM Connect

| Field             | Value                                      |
|-------------------|--------------------------------------------|
| **Document ID**   | `DOC-ARCH-004`                             |
| **Version**       | `1.0.0`                                    |
| **Status**        | `APPROVED`                                 |
| **Owner**         | Engineering Lead                           |
| **Last Updated**  | 2026-10-02                                 |
| **Related Docs**  | DOC-ARCH-001, DOC-MSG-001, DOC-CALL-003   |
| **Related ADRs**  | ADR-015, ADR-016                           |

---

## 1. User Registration Flow

```mermaid
sequenceDiagram
    participant C as Client
    participant CF as Cloudflare
    participant AX as Axum
    participant SVC as AuthService
    participant REPO as UserRepository
    participant PG as PostgreSQL
    participant EMAIL as Email Service

    C->>CF: POST /api/v1/auth/register
    CF->>AX: Proxy request
    AX->>AX: Rate limit check (5/hr per IP)
    AX->>SVC: register(email, password, display_name)
    SVC->>SVC: Validate email format
    SVC->>SVC: Validate password strength
    SVC->>REPO: find_by_email(email)
    REPO->>PG: SELECT FROM users WHERE email = $1
    PG-->>REPO: None (new user)
    SVC->>SVC: Hash password (Argon2id, spawn_blocking)
    SVC->>REPO: create_user(email, hash, display_name)
    REPO->>PG: BEGIN; INSERT INTO users; INSERT INTO user_profiles; COMMIT
    PG-->>REPO: User
    SVC->>SVC: Generate 6-digit OTP, hash with SHA-256
    SVC->>REPO: create_email_verification(user_id, otp_hash)
    REPO->>PG: INSERT INTO email_verifications
    SVC->>EMAIL: send_otp(email, otp_plaintext)
    EMAIL-->>SVC: Queued (async, best-effort)
    SVC-->>AX: Ok(user)
    AX-->>CF: 201 Created {user_id, status: PENDING_VERIFICATION}
    CF-->>C: Response
```

## 2. Login Flow

```mermaid
sequenceDiagram
    participant C as Client
    participant AX as Axum
    participant SVC as AuthService
    participant PG as PostgreSQL
    participant REDIS as Redis

    C->>AX: POST /api/v1/auth/login {email, password, device_id, device_name, device_type}
    AX->>AX: Rate limit check (10/15min per email)
    AX->>SVC: login(email, password, device_info)
    SVC->>PG: SELECT id, password_hash, status FROM users WHERE email = $1
    PG-->>SVC: User
    SVC->>SVC: Check status == ACTIVE
    SVC->>SVC: Verify password (Argon2id, spawn_blocking)
    SVC->>SVC: Generate access_token (32 random bytes)
    SVC->>SVC: Generate refresh_token (64 random bytes)
    SVC->>PG: BEGIN
    SVC->>PG: INSERT INTO devices ON CONFLICT UPDATE
    SVC->>PG: INSERT INTO sessions
    SVC->>PG: INSERT INTO refresh_tokens (token_hash, family_id)
    SVC->>PG: INSERT INTO audit_logs (LOGIN)
    SVC->>PG: COMMIT
    SVC->>REDIS: SET session:{sha256(access_token)} {user_id, session_id, device_id} EX 900
    SVC-->>AX: Ok(tokens)
    AX-->>C: 200 {access_token, refresh_token, expires_in: 900, user: {...}}
```

## 3. Send Message Flow (Complete)

```mermaid
sequenceDiagram
    participant C as Sender Client
    participant WS as WebSocket Handler
    participant MW as Auth Middleware
    participant SVC as MessagingService
    participant REPO as MessageRepository
    participant PG as PostgreSQL
    participant REDIS as Redis
    participant R as Recipient Client

    C->>WS: message.send {client_message_id, conversation_id, content}
    WS->>MW: Validate auth (session from connection state)
    MW-->>WS: user_id, session_id
    WS->>SVC: send_message(user_id, payload)
    
    SVC->>SVC: Validate content (length, type)
    SVC->>REPO: is_conversation_member(user_id, conversation_id)
    REPO->>PG: SELECT 1 FROM conversation_members WHERE ...
    PG-->>REPO: true
    
    SVC->>REPO: check_blocked(user_id, recipient_ids)
    REPO->>PG: SELECT 1 FROM blocked_users WHERE ...
    PG-->>REPO: not blocked
    
    SVC->>REPO: insert_message(message)
    REPO->>PG: INSERT INTO messages ... ON CONFLICT (conversation_id, client_message_id) DO NOTHING RETURNING *
    PG-->>REPO: Message (new or existing)
    
    SVC-->>WS: Ok(message)
    WS-->>C: message.ack {server_message_id, timestamp}
    
    SVC->>SVC: Get recipient user_ids from conversation_members
    
    alt Recipient on same instance
        SVC->>R: message.new (in-process channel)
    else Recipient on different instance
        SVC->>REDIS: PUBLISH conversation:{id} message_event
        REDIS->>R: message_event (via subscription on other instance)
    end
    
    alt Recipient offline
        SVC->>SVC: Queue push notification
        SVC->>SVC: NotificationWorker sends FCM/WebPush
    end
```

## 4. Media Upload + Send Flow

```mermaid
sequenceDiagram
    participant C as Client
    participant AX as Axum
    participant MEDIA as MediaService
    participant R2 as Cloudflare R2
    participant PG as PostgreSQL
    participant WS as WebSocket

    C->>AX: POST /media/upload (multipart: file + conversation_id)
    AX->>MEDIA: upload(user_id, file, conversation_id)
    MEDIA->>MEDIA: Validate: magic bytes, size, type, membership
    MEDIA->>MEDIA: Generate media_id, r2_key
    MEDIA->>R2: PUT object (original file)
    R2-->>MEDIA: Ok
    
    opt Image or Video
        MEDIA->>MEDIA: Generate thumbnail (spawn_blocking)
        MEDIA->>R2: PUT thumbnail
    end
    
    MEDIA->>PG: INSERT INTO media_objects
    MEDIA-->>AX: 201 {media_id}
    AX-->>C: {media_id, thumbnail_url}
    
    Note over C: Client now sends a message referencing this media
    C->>WS: message.send {content_type: "image", client_message_id, conversation_id}
    Note over WS: Normal message flow + INSERT INTO message_attachments
```

## 5. Call Setup Flow (Complete)

```mermaid
sequenceDiagram
    participant CA as Caller
    participant AX as Axum (WebSocket)
    participant SVC as CallService
    participant PG as PostgreSQL
    participant REDIS as Redis
    participant CE as Callee

    CA->>AX: call.offer {callee_id, call_type: "audio", sdp_offer}
    AX->>SVC: initiate_call(caller_id, callee_id, sdp_offer)
    SVC->>REDIS: GET call:{callee_id}
    REDIS-->>SVC: None (callee not in call)
    SVC->>PG: INSERT INTO calls (status: CALLING)
    SVC->>PG: INSERT INTO call_participants (caller)
    SVC->>REDIS: SET call:{caller_id} {call_id} EX 3600
    SVC->>REDIS: SET call:{callee_id} {call_id} EX 3600
    
    SVC->>CE: call.offer {call_id, caller_id, sdp_offer}
    SVC->>CA: call.ringing {call_id}
    SVC->>PG: UPDATE calls SET status = 'RINGING'
    SVC->>SVC: Start 30s timeout timer
    
    alt Callee answers
        CE->>AX: call.answer {call_id, sdp_answer}
        SVC->>PG: UPDATE calls SET status = 'CONNECTING'
        SVC->>CA: call.answer {sdp_answer}
        
        Note over CA, CE: ICE candidate exchange (trickle)
        CA->>AX: call.ice_candidate {candidate}
        AX->>CE: call.ice_candidate {candidate}
        CE->>AX: call.ice_candidate {candidate}
        AX->>CA: call.ice_candidate {candidate}
        
        Note over CA, CE: ICE succeeds, media flows P2P
        SVC->>PG: UPDATE calls SET status = 'CONNECTED', started_at = NOW()
        SVC->>PG: INSERT INTO call_participants (callee)
    
    else Callee rejects
        CE->>AX: call.reject {call_id}
        SVC->>PG: UPDATE calls SET status = 'REJECTED', ended_at = NOW()
        SVC->>CA: call.rejected {call_id}
        SVC->>REDIS: DEL call:{caller_id} call:{callee_id}
    
    else Timeout (30s)
        SVC->>PG: UPDATE calls SET status = 'MISSED', ended_at = NOW()
        SVC->>CA: call.timeout {call_id}
        SVC->>CE: call.timeout {call_id}
        SVC->>REDIS: DEL call:{caller_id} call:{callee_id}
        SVC->>SVC: Send push notification to callee (MISSED_CALL)
    end
```

## 6. Token Refresh Flow

```mermaid
sequenceDiagram
    participant C as Client
    participant AX as Axum
    participant SVC as AuthService
    participant PG as PostgreSQL
    participant REDIS as Redis

    C->>AX: POST /auth/refresh {refresh_token} (or cookie)
    AX->>SVC: refresh(refresh_token)
    SVC->>SVC: Hash token: sha256(refresh_token)
    SVC->>PG: SELECT * FROM refresh_tokens WHERE token_hash = $1
    
    alt Token valid (not revoked, not expired)
        SVC->>SVC: Generate new access_token + new refresh_token
        SVC->>PG: BEGIN (SERIALIZABLE)
        SVC->>PG: UPDATE refresh_tokens SET revoked_at = NOW() WHERE id = $1
        SVC->>PG: INSERT INTO refresh_tokens (new_hash, same family_id)
        SVC->>PG: COMMIT
        SVC->>REDIS: SET session:{new_access_hash} ... EX 900
        SVC->>REDIS: DEL session:{old_access_hash}
        SVC-->>AX: Ok(new_tokens)
        AX-->>C: 200 {access_token, refresh_token, expires_in: 900}
    
    else Token revoked (REUSE ATTACK)
        SVC->>PG: SELECT family_id FROM refresh_tokens WHERE token_hash = $1
        SVC->>PG: UPDATE refresh_tokens SET revoked_at = NOW() WHERE family_id = $1
        Note over SVC: All tokens in this family are now revoked
        SVC->>PG: UPDATE sessions SET ended_at = NOW() WHERE ...
        SVC->>REDIS: Delete all session keys for this user
        SVC-->>AX: Err(UNAUTHORIZED, "Token reuse detected")
        AX-->>C: 401 {error: "Session invalidated for security"}
    
    else Token expired
        SVC-->>AX: Err(UNAUTHORIZED, "Refresh token expired")
        AX-->>C: 401 {error: "Please log in again"}
    end
```

## 7. Conversation Sync Flow (Reconnection)

```mermaid
sequenceDiagram
    participant C as Client
    participant WS as WebSocket
    participant SVC as MessagingService
    participant PG as PostgreSQL

    Note over C: Client reconnects after being offline

    C->>WS: conversation.sync {conversations: [{id, last_message_id, last_timestamp}, ...]}
    WS->>SVC: sync_conversations(user_id, sync_request)
    
    loop For each conversation
        SVC->>PG: SELECT * FROM messages<br/>WHERE conversation_id = $1<br/>AND created_at > $2<br/>ORDER BY created_at ASC<br/>LIMIT 100
        PG-->>SVC: [message1, message2, ...]
    end
    
    SVC-->>WS: sync_response
    WS-->>C: conversation.sync_response {conversations: [{id, messages: [...], has_more}, ...]}
    
    Note over C: Client merges messages into local state,<br/>sorted by server timestamp
    
    opt More messages available
        C->>WS: conversation.sync {id, last_message_id: last_received_id}
        Note over C,WS: Repeat until has_more == false
    end
    
    C->>WS: message.delivered {message_ids: [all newly received]}
```

---

*Next: [architecture-overview.md](../02-architecture/architecture-overview.md) · [messaging-architecture.md](../06-messaging/messaging-architecture.md)*
