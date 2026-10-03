# Functional Requirements — YBM Connect

| Field             | Value                                      |
|-------------------|--------------------------------------------|
| **Document ID**   | `DOC-REQ-002`                              |
| **Version**       | `1.0.0`                                    |
| **Status**        | `APPROVED`                                 |
| **Owner**         | Engineering Lead                           |
| **Last Updated**  | 2026-10-02                                 |
| **Related Docs**  | DOC-REQ-001, DOC-REQ-003, DOC-REQ-004     |
| **Related ADRs**  | ADR-001 through ADR-017                    |
| **Related Reqs**  | BR-001 through BR-011                      |

---

## Authentication & Account Management

### AUTH-001: User Registration
**Actors:** Unregistered user
**Preconditions:** User has a valid email address
**Postconditions:** Account created, email verification pending
**Description:** Users register with email, password, and display name. Password is hashed with Argon2id before storage. An email OTP is sent for verification. The account is in `PENDING_VERIFICATION` state until OTP is confirmed.
**Validation:** Email format, password minimum 8 chars with complexity requirements, display name 2-50 chars
**API:** `POST /api/v1/auth/register`
**Database:** Insert into `users`, `email_verifications`
**Security:** Rate limit 5 registrations per IP per hour
**Failure cases:** Duplicate email (409), invalid input (422), email send failure (503)
**Related:** BR-005, AUTH-002

### AUTH-002: Email OTP Verification
**Actors:** Registered user with pending verification
**Preconditions:** User has registered and received OTP email
**Postconditions:** Account status transitions to `ACTIVE`
**Description:** User submits the 6-digit OTP received via email. Server validates OTP against stored hash, checks expiry (10 minutes), and activates the account.
**API:** `POST /api/v1/auth/verify-email`
**Database:** Update `users.status`, mark `email_verifications.verified_at`
**Security:** Rate limit 5 attempts per OTP, OTP expires after 10 minutes, OTP is single-use
**Failure cases:** Invalid OTP (401), expired OTP (410), max attempts exceeded (429)
**Related:** BR-005, AUTH-001

### AUTH-003: User Login
**Actors:** Registered, verified user
**Preconditions:** Account is `ACTIVE`
**Postconditions:** Access token and refresh token issued, session created, device registered
**Description:** User provides email and password. Server verifies password against Argon2id hash. On success, creates a session and device record, returns short-lived access token (15 min) and refresh token (30 days).
**API:** `POST /api/v1/auth/login`
**Database:** Insert into `sessions`, `devices`
**Security:** Rate limit 10 login attempts per email per 15 minutes, constant-time comparison
**Failure cases:** Invalid credentials (401), account locked (423), account not verified (403)
**Related:** BR-005, AUTH-004, AUTH-005

### AUTH-004: Token Refresh
**Actors:** Authenticated user with valid refresh token
**Preconditions:** Refresh token exists and is not expired/revoked
**Postconditions:** New access token issued, refresh token optionally rotated
**Description:** Client presents refresh token to obtain a new access token. Refresh token is rotated (old one invalidated) to limit token reuse.
**API:** `POST /api/v1/auth/refresh`
**Database:** Update `refresh_tokens` (revoke old, create new)
**Security:** Refresh token rotation, detect reuse (invalidate all tokens for user if reused token detected)
**Failure cases:** Invalid/expired token (401), revoked token with reuse detection (401 + session invalidation)
**Related:** BR-005, AUTH-003

### AUTH-005: Logout
**Actors:** Authenticated user
**Preconditions:** User has an active session
**Postconditions:** Session terminated, tokens revoked for that device
**Description:** Invalidates the current session and all associated tokens. WebSocket connection for that device is closed.
**API:** `POST /api/v1/auth/logout`
**Database:** Update `sessions.ended_at`, revoke `refresh_tokens`
**Related:** BR-005

### AUTH-006: Logout All Devices
**Actors:** Authenticated user
**Preconditions:** User has one or more active sessions
**Postconditions:** All sessions terminated, all tokens revoked
**Description:** Invalidates all sessions and tokens across all devices. All WebSocket connections for the user are closed.
**API:** `POST /api/v1/auth/logout-all`
**Database:** Update all `sessions`, revoke all `refresh_tokens` for user
**Related:** BR-005, BR-009

### AUTH-007: Password Reset Request
**Actors:** User who has forgotten their password
**Preconditions:** Account exists
**Postconditions:** Reset OTP sent via email
**Description:** User requests password reset. Server sends OTP to registered email. Does NOT reveal whether the email exists (returns success regardless).
**API:** `POST /api/v1/auth/forgot-password`
**Security:** Rate limit 3 requests per email per hour, no account enumeration
**Related:** BR-005

### AUTH-008: Password Reset Confirmation
**Actors:** User with valid reset OTP
**Preconditions:** OTP is valid and not expired
**Postconditions:** Password updated, all existing sessions revoked
**Description:** User submits OTP and new password. Server validates OTP, hashes new password, updates the record, and revokes all existing sessions (forcing re-login on all devices).
**API:** `POST /api/v1/auth/reset-password`
**Database:** Update `users.password_hash`, revoke all `sessions` and `refresh_tokens`
**Related:** BR-005, AUTH-006

---

## Messaging

### MSG-001: Send Text Message
**Actors:** Authenticated user who is a member of the conversation
**Preconditions:** User is authenticated, conversation exists, user is a member
**Postconditions:** Message persisted, delivered to online recipients, delivery status tracked
**Description:** Client generates a `client_message_id` (UUID v7) and sends the message over WebSocket. Server validates, checks idempotency (deduplicates on `client_message_id`), persists to PostgreSQL, publishes via Redis pub/sub, and routes to all conversation members' connected devices.
**WebSocket event:** `message.send` (client → server), `message.new` (server → client)
**Database:** Insert into `messages`
**Idempotency:** `client_message_id` is unique per conversation; duplicate sends return the existing message
**Security:** Input sanitization, message size limit (4096 UTF-8 chars), rate limit 30 messages per minute per user
**Failure cases:** Not a member (403), conversation not found (404), duplicate detected (200 with existing message), server error (500)
**Related:** BR-001, BR-002, MSG-002, MSG-003

### MSG-002: Delivery Acknowledgement
**Actors:** Recipient's client
**Preconditions:** Message delivered to recipient's device
**Postconditions:** Delivery receipt stored, sender notified
**Description:** When a client receives a message, it sends a `message.delivered` event to the server. Server records the delivery timestamp in `message_receipts` and notifies the sender.
**WebSocket event:** `message.delivered` (client → server), `message.status` (server → sender)
**Database:** Insert/update `message_receipts`
**Related:** BR-002, MSG-001, MSG-003

### MSG-003: Read Receipt
**Actors:** Recipient user
**Preconditions:** Message has been delivered, user has viewed the message in the UI
**Postconditions:** Read timestamp stored, sender notified
**Description:** When a user reads a message (message enters viewport), client sends `message.read`. Server updates receipt, notifies sender. Read receipts are batched — client sends the latest read message ID per conversation, marking all prior messages as read.
**WebSocket event:** `message.read` (client → server), `message.status` (server → sender)
**Database:** Update `message_receipts`
**Related:** BR-002, MSG-002

### MSG-004: Edit Message
**Actors:** Original message sender
**Preconditions:** Message exists, user is the sender, message is within edit window (15 minutes)
**Postconditions:** Message content updated, edit history recorded, recipients notified
**Description:** Sender edits a previously sent message. The original content is preserved in `message_edits`. Recipients receive the updated content.
**API:** Via WebSocket `message.edit`
**Database:** Update `messages.content`, insert into `message_edits`
**Security:** Only the original sender can edit; edit window enforced server-side
**Related:** MSG-001

### MSG-005: Delete Message
**Actors:** Original message sender
**Preconditions:** Message exists, user is the sender
**Postconditions:** Message marked as deleted, recipients notified
**Description:** Soft delete. The message content is replaced with a deletion marker. The message row is retained for referential integrity. Attachments are scheduled for cleanup.
**WebSocket event:** `message.delete` (client → server), `message.deleted` (server → recipients)
**Database:** Update `messages.deleted_at`, `messages.content = NULL`
**Related:** MSG-001

### MSG-006: React to Message
**Actors:** Conversation member
**Preconditions:** Message exists, user is a conversation member
**Postconditions:** Reaction stored, conversation members notified
**Description:** User adds an emoji reaction to a message. One reaction per emoji per user per message. Removing a reaction is a separate action.
**WebSocket event:** `message.react` (client → server), `message.reaction` (server → members)
**Database:** Insert/delete `message_reactions`
**Security:** Validate emoji is from allowed set, rate limit 20 reactions per minute
**Related:** MSG-001

### MSG-007: Reply to Message
**Actors:** Conversation member
**Preconditions:** Referenced message exists in the same conversation
**Postconditions:** New message created with `reply_to_message_id` set
**Description:** User sends a message that references another message. The reply is a regular message with `reply_to_message_id` pointing to the original. The client renders it with the quoted original.
**Database:** `messages.reply_to_message_id` FK
**Related:** MSG-001

### MSG-008: Forward Message
**Actors:** Conversation member
**Preconditions:** Source message exists, user is a member of both source and target conversations
**Postconditions:** New message created in target conversation with `forwarded_from_message_id`
**Description:** Creates a copy of the message in the target conversation. Media attachments are referenced (not duplicated in storage). The message is marked as forwarded.
**API:** `POST /api/v1/messages/forward` (REST, because it crosses conversations)
**Database:** Insert new `messages` with `forwarded_from_message_id`
**Related:** MSG-001

### MSG-009: Offline Message Delivery
**Actors:** System (automatic)
**Preconditions:** Recipient is offline when message is sent
**Postconditions:** Message delivered when recipient reconnects
**Description:** Messages are always persisted to PostgreSQL. When a client reconnects, it sends its last known message timestamp/ID. The server queries for all messages after that point and delivers them in order.
**WebSocket event:** `conversation.sync` (client → server), `message.new` batch (server → client)
**Related:** BR-002, MSG-001

---

## Presence & Indicators

### PRES-001: Online/Offline Presence
**Actors:** All authenticated users
**Preconditions:** User is authenticated
**Postconditions:** User's presence state visible to contacts/conversation members
**Description:** When a user connects via WebSocket, their presence is set to `ONLINE`. When they disconnect (or after a 30-second heartbeat timeout), presence changes to `OFFLINE` and `last_seen` is recorded. Presence is stored in Redis with TTL.
**WebSocket event:** `presence.update` (server → subscribed clients)
**Redis:** `presence:{user_id}` key with TTL
**Related:** BR-007

### PRES-002: Last Seen
**Actors:** Contacts/conversation members
**Preconditions:** Target user has previously been online
**Postconditions:** Last seen timestamp available
**Description:** `last_seen` is updated in PostgreSQL when presence transitions from ONLINE to OFFLINE. Returned in user profile and presence queries.
**Database:** `users.last_seen_at`
**Privacy:** Users can disable last seen visibility (future enhancement)
**Related:** BR-007, PRES-001

### PRES-003: Typing Indicators
**Actors:** Conversation member who is typing
**Preconditions:** User is in a conversation, WebSocket connected
**Postconditions:** Other conversation members see typing indicator
**Description:** Client sends `typing.start` when user begins typing. Server relays to other conversation members via Redis pub/sub. Client sends `typing.stop` when user stops typing (2-second debounce) or sends the message. Server auto-expires typing state after 5 seconds without renewal.
**WebSocket events:** `typing.start`, `typing.stop` (both directions)
**Redis:** Short-lived key `typing:{conversation_id}:{user_id}` with 5s TTL
**Related:** BR-007

---

## Groups

### GRP-001: Create Group
**Actors:** Authenticated user
**Preconditions:** User is authenticated
**Postconditions:** Group created, creator is admin
**Description:** User creates a group with a name and optional description. The creator is automatically added as a member with `ADMIN` role. A group conversation is created.
**API:** `POST /api/v1/groups`
**Database:** Insert into `groups`, `group_members`, `conversations`, `conversation_members`
**Validation:** Group name 1-100 chars, description 0-500 chars
**Related:** BR-006

### GRP-002: Add Group Members
**Actors:** Group admin
**Preconditions:** Actor is admin of the group, target user exists, target is not already a member
**Postconditions:** New member added, group members notified
**Description:** Admin adds one or more users to the group. Each new member is added to the group and the associated conversation.
**API:** `POST /api/v1/groups/{group_id}/members`
**Security:** Only admins can add members
**Related:** BR-006, GRP-001

### GRP-003: Remove Group Member
**Actors:** Group admin
**Preconditions:** Actor is admin, target is a member, target is not the last admin
**Postconditions:** Member removed, group members notified
**Description:** Admin removes a member from the group and its conversation.
**API:** `DELETE /api/v1/groups/{group_id}/members/{user_id}`
**Related:** BR-006

### GRP-004: Leave Group
**Actors:** Group member
**Preconditions:** User is a group member
**Postconditions:** User removed from group, remaining members notified. If last admin, admin role transfers to longest-tenured member.
**API:** `POST /api/v1/groups/{group_id}/leave`
**Related:** BR-006

### GRP-005: Update Group Info
**Actors:** Group admin
**Preconditions:** Actor is admin
**Postconditions:** Group name/description/avatar updated
**API:** `PATCH /api/v1/groups/{group_id}`
**Related:** BR-006

### GRP-006: Group Roles
**Actors:** Group admin
**Preconditions:** Actor is admin, target is a member
**Postconditions:** Target's role updated
**Description:** Roles: `ADMIN`, `MEMBER`. Admins can promote members to admin or demote other admins (except themselves). The group must always have at least one admin.
**API:** `PATCH /api/v1/groups/{group_id}/members/{user_id}/role`
**Related:** BR-006

---

## Media

### MEDIA-001: Upload Image
**Actors:** Authenticated user in a conversation
**Preconditions:** User is a conversation member
**Postconditions:** Image stored in R2, thumbnail generated, message created with attachment
**Description:** Client uploads image via multipart form to REST endpoint. Server validates file type (JPEG, PNG, WebP, GIF), file size (max 10MB), generates thumbnail, stores original and thumbnail in R2, creates message with attachment reference.
**API:** `POST /api/v1/media/upload`
**Storage:** Cloudflare R2 bucket `ybm-connect-media`
**Security:** File type validation (magic bytes, not just extension), malware scanning (future), image dimension limits
**Related:** BR-003

### MEDIA-002: Upload Video
**Actors:** Authenticated user in a conversation
**Preconditions:** User is a conversation member
**Postconditions:** Video stored in R2, thumbnail extracted, message created
**Description:** Similar to image upload. Max 50MB. Server extracts first-frame thumbnail. No server-side transcoding in V1 (clients are expected to send compatible formats: MP4/H.264).
**API:** `POST /api/v1/media/upload`
**Related:** BR-003

### MEDIA-003: Upload Document
**Actors:** Authenticated user in a conversation
**Preconditions:** User is a conversation member
**Postconditions:** Document stored in R2, message created
**Description:** Supports PDF, DOC/DOCX, XLS/XLSX, PPT/PPTX, TXT, ZIP. Max 25MB. No content processing beyond type validation and size check.
**API:** `POST /api/v1/media/upload`
**Related:** BR-003

### MEDIA-004: Voice Message
**Actors:** Authenticated user in a conversation
**Preconditions:** User is a conversation member
**Postconditions:** Audio file stored in R2, duration extracted, message created
**Description:** Client records audio (Opus in OGG container), sends to server. Server validates format, extracts duration metadata, stores in R2.
**API:** `POST /api/v1/media/upload`
**Validation:** Max 5 minutes, max 10MB, audio format validation
**Related:** BR-003

### MEDIA-005: Download Media
**Actors:** Conversation member
**Preconditions:** User is a member of the conversation containing the media
**Postconditions:** Media URL returned
**Description:** Client requests a download URL for a media attachment. Server verifies authorization (user is conversation member), generates a time-limited signed URL to R2, returns it. Client fetches directly from R2.
**API:** `GET /api/v1/media/{media_id}/url`
**Security:** Signed URLs with 1-hour expiry, authorization check
**Related:** BR-003

---

## Calls

### CALL-001: Initiate Audio Call
**Actors:** Authenticated user
**Preconditions:** Target user exists, not blocked, not in an active call
**Postconditions:** Call signaling initiated, callee receives incoming call event
**Description:** Caller sends call offer via WebSocket. Server validates, creates call record, forwards offer to callee. Callee has 30 seconds to answer.
**WebSocket events:** `call.offer` (caller → server → callee), `call.ringing` (server → caller)
**Database:** Insert into `calls`, `call_participants`
**Related:** BR-004

### CALL-002: Initiate Video Call
**Actors:** Authenticated user
**Preconditions:** Same as CALL-001
**Postconditions:** Same as CALL-001, with video track included in SDP
**Description:** Identical to audio call flow but SDP includes video codec negotiation.
**Related:** BR-004, CALL-001

### CALL-003: Answer Call
**Actors:** Callee
**Preconditions:** Incoming call exists, within timeout window
**Postconditions:** SDP answer sent, ICE exchange begins, media connection established
**WebSocket events:** `call.answer` (callee → server → caller)
**Related:** CALL-001

### CALL-004: Reject Call
**Actors:** Callee
**Preconditions:** Incoming call exists
**Postconditions:** Call ended with REJECTED status
**WebSocket events:** `call.reject` (callee → server → caller)
**Database:** Update `calls.status = REJECTED`
**Related:** CALL-001

### CALL-005: End Call
**Actors:** Either call participant
**Preconditions:** Call is active or ringing
**Postconditions:** Call ended, duration recorded
**WebSocket events:** `call.end` (participant → server → other participant)
**Database:** Update `calls.ended_at`, `calls.status = ENDED`
**Related:** CALL-001

### CALL-006: Call History
**Actors:** Authenticated user
**Preconditions:** User has participated in calls
**Postconditions:** Call history returned
**API:** `GET /api/v1/calls/history`
**Database:** Query `calls` + `call_participants` where user is participant
**Related:** BR-004

---

## Notifications

### NOTIF-001: Push Notification for New Message
**Actors:** System
**Preconditions:** Recipient's device has registered for push, app is not in foreground
**Postconditions:** Push notification delivered to device
**Description:** When a message cannot be delivered via WebSocket (device is offline or app is backgrounded), the server sends a push notification via FCM (Android) or Web Push (browser). Notification contains sender name and a preview of the message content (first 100 chars).
**Related:** BR-008

### NOTIF-002: Push Notification for Incoming Call
**Actors:** System
**Preconditions:** Callee's device has push registration, incoming call exists
**Postconditions:** Push notification delivered, displays incoming call UI
**Description:** High-priority push notification that triggers the incoming call UI on the device.
**Related:** BR-008, CALL-001

---

*Next: [non-functional-requirements.md](non-functional-requirements.md) · [user-stories.md](user-stories.md)*
