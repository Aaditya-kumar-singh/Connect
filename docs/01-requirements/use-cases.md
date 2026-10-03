# Use Cases — YBM Connect

| Field             | Value                                      |
|-------------------|--------------------------------------------|
| **Document ID**   | `DOC-REQ-005`                              |
| **Version**       | `1.0.0`                                    |
| **Status**        | `APPROVED`                                 |
| **Owner**         | Engineering Lead                           |
| **Last Updated**  | 2026-10-02                                 |
| **Related Docs**  | DOC-REQ-002, DOC-REQ-004                  |

---

## UC-001: Send and Receive a Text Message

**Primary Actor:** Sender (User A)
**Secondary Actor:** Recipient (User B)
**Preconditions:** Both users are registered and verified. A 1-to-1 conversation exists (or is created on first message).
**Trigger:** User A types a message and presses send.

### Main Flow
1. User A composes a text message in the conversation UI.
2. Client generates `client_message_id` (UUID v7).
3. Client sends `message.send` event over WebSocket with payload: `{ conversation_id, client_message_id, content, content_type: "text" }`.
4. Server receives the event, authenticates via WebSocket session.
5. Server validates: content length (≤ 4096 chars), user is conversation member, input sanitization.
6. Server performs idempotency check: looks up `client_message_id` in `messages` table.
7. If not a duplicate: server inserts message into PostgreSQL within a transaction.
8. Server sends `message.ack` to sender with `{ client_message_id, server_message_id, timestamp }`.
9. Server publishes `message.new` event to Redis channel `conversation:{conversation_id}`.
10. For each recipient connected via WebSocket (possibly on different backend instances): the Redis subscriber picks up the event and sends `message.new` to the recipient's WebSocket.
11. Recipient's client receives `message.new`, renders the message, and sends `message.delivered` back to server.
12. Server records delivery receipt in `message_receipts` table.
13. Server forwards `message.status` (delivered) to sender.
14. Sender's client updates the message status indicator.

### Alternative Flows

**AF-1: Recipient is offline**
- Step 10: No WebSocket connection exists for the recipient.
- Server sends push notification (NOTIF-001).
- When recipient reconnects, client sends `conversation.sync` with last known message ID.
- Server returns all undelivered messages.

**AF-2: Duplicate message (idempotency)**
- Step 6: `client_message_id` already exists.
- Server returns the existing message as the ACK. No new message is created.

**AF-3: Sender is not a conversation member**
- Step 5: Authorization fails.
- Server sends error `{ code: "FORBIDDEN", message: "Not a conversation member" }`.

**AF-4: Database write fails**
- Step 7: PostgreSQL write fails.
- Server sends `message.error` to sender with `{ client_message_id, error: "INTERNAL_ERROR" }`.
- Client retries (with the same `client_message_id`, ensuring idempotency).

**AF-5: Redis publish fails**
- Step 9: Redis is unavailable.
- Message is persisted (step 7 succeeded). ACK is sent to sender.
- Recipient receives the message on next `conversation.sync` (reconnection/polling).
- This is a degraded mode: real-time delivery is delayed but message is not lost.

### Postconditions
- Message is persisted in PostgreSQL.
- All online recipients have received the message.
- Offline recipients will receive it on reconnection.
- Delivery receipts are recorded.

---

## UC-002: 1-to-1 Audio Call

**Primary Actor:** Caller (User A)
**Secondary Actor:** Callee (User B)
**Preconditions:** Both users are registered. User B is online (or reachable via push notification).

### Main Flow
1. User A taps the call button in a conversation with User B.
2. Client creates a WebRTC `RTCPeerConnection`.
3. Client adds local audio track (requests microphone permission).
4. Client creates an SDP offer via `createOffer()`.
5. Client sets local description via `setLocalDescription(offer)`.
6. Client sends `call.offer` via WebSocket: `{ callee_id, sdp_offer, call_type: "audio" }`.
7. Server creates a `calls` record with status `CALLING`.
8. Server forwards `call.offer` to callee via WebSocket (or push notification if offline).
9. Callee's client receives `call.offer`, shows incoming call UI.
10. Callee accepts: creates `RTCPeerConnection`, adds local audio track.
11. Callee creates SDP answer via `createAnswer()`.
12. Callee sends `call.answer` via WebSocket: `{ call_id, sdp_answer }`.
13. Server updates call status to `CONNECTING`, forwards answer to caller.
14. Both peers exchange ICE candidates via `call.ice_candidate` events (trickle ICE).
15. ICE connectivity check succeeds. `RTCPeerConnection.connectionState` → `connected`.
16. Server updates call status to `CONNECTED`.
17. Audio flows directly between peers (P2P) or via TURN relay.
18. Either party ends the call: sends `call.end`.
19. Server updates call status to `ENDED`, records duration.
20. Both clients close `RTCPeerConnection` and release media tracks.

### Alternative Flows

**AF-1: Callee rejects**
- Step 9: Callee taps reject.
- `call.reject` sent to server. Server updates status to `REJECTED`. Caller notified.

**AF-2: Callee doesn't answer (timeout)**
- Step 9: 30 seconds elapse without answer.
- Server sends `call.timeout` to both parties. Status set to `MISSED`.

**AF-3: ICE fails (need TURN)**
- Step 14: Direct connectivity fails.
- ICE agent falls back to TURN relay candidates.
- Media flows through TURN server (higher latency but still works).

**AF-4: ICE fails completely**
- Step 14: No path found (TURN unavailable or misconfigured).
- `RTCPeerConnection.connectionState` → `failed`.
- Client sends `call.end` with reason `ICE_FAILED`.
- Call status set to `FAILED`.

**AF-5: Network change during call**
- During step 17: User switches from WiFi to mobile data.
- ICE restart triggered. New candidates exchanged.
- Call may briefly interrupt then resume. Status set to `RECONNECTING` then `CONNECTED`.

---

## UC-003: Group Messaging

**Primary Actor:** Group member (User A)
**Secondary Actors:** Other group members (Users B, C, D, ...)
**Preconditions:** Group exists, User A is a member.

### Main Flow
1. User A sends a message in the group conversation.
2. Message flow follows UC-001 steps 1-8.
3. At step 9, server publishes to Redis channel `conversation:{group_conversation_id}`.
4. All online group members receive the message via their WebSocket connections.
5. Each member's client sends `message.delivered`.
6. Server records delivery receipts per member.
7. Read receipts work the same as 1-to-1 but are tracked per member.

### Key Differences from 1-to-1
- Multiple delivery receipts (one per member per device).
- Read receipt aggregation: sender sees "delivered to X, read by Y" counts.
- Push notifications sent to all offline members.
- Message fan-out is O(N) where N = group members. For groups up to 256, this is acceptable with Redis pub/sub.

---

## UC-004: Media Upload and Sharing

**Primary Actor:** User A (sender)
**Secondary Actor:** User B (recipient)
**Preconditions:** User A is a member of the conversation. File is within size limits.

### Main Flow
1. User A selects a file (image, video, document) from device.
2. Client validates file locally: type, size, dimensions (for images).
3. Client uploads file via `POST /api/v1/media/upload` with multipart form data including `conversation_id`.
4. Server validates: file type (magic bytes), file size, user is conversation member.
5. For images: server generates thumbnail (320px width, WebP format).
6. Server uploads original and thumbnail to Cloudflare R2 with path: `{conversation_id}/{media_id}/{filename}`.
7. Server creates `media_objects` record with metadata (size, type, dimensions, duration, R2 key).
8. Server creates a `messages` record with `content_type: "image"` and `message_attachments` linking to the media object.
9. Server sends `message.ack` to sender.
10. Server publishes `message.new` with media metadata (thumbnail URL, dimensions, size).
11. Recipient's client receives message, displays thumbnail placeholder.
12. Recipient taps to view full image: client requests `GET /api/v1/media/{media_id}/url`.
13. Server verifies authorization, generates signed R2 URL (1-hour expiry), returns it.
14. Client fetches image directly from R2.

---

*Next: [acceptance-criteria.md](acceptance-criteria.md) · [requirements-traceability.md](requirements-traceability.md)*
