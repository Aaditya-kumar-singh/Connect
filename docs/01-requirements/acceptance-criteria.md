# Acceptance Criteria — YBM Connect

| Field             | Value                                      |
|-------------------|--------------------------------------------|
| **Document ID**   | `DOC-REQ-006`                              |
| **Version**       | `1.0.0`                                    |
| **Status**        | `APPROVED`                                 |
| **Owner**         | Engineering Lead                           |
| **Last Updated**  | 2026-10-02                                 |
| **Related Docs**  | DOC-REQ-002, DOC-REQ-004, DOC-REQ-005     |

---

## Format

Each acceptance criterion follows the GIVEN-WHEN-THEN pattern and traces back to a functional requirement and user story.

---

## Authentication

### AC-AUTH-001 (AUTH-001, US-001)
**GIVEN** a user with a valid email address that is not already registered
**WHEN** they submit the registration form with email, password (≥ 8 chars with uppercase, lowercase, and digit), and display name (2-50 chars)
**THEN** an account is created in `PENDING_VERIFICATION` state, a 6-digit OTP is sent to their email, and a 201 response is returned with `{ user_id }`.

### AC-AUTH-002 (AUTH-001, US-001)
**GIVEN** a user trying to register with an email that already exists
**WHEN** they submit the registration form
**THEN** a 409 response is returned with error code `EMAIL_ALREADY_EXISTS`. The response MUST NOT reveal the existing user's information.

### AC-AUTH-003 (AUTH-002, US-002)
**GIVEN** a user with a pending verification and a valid, non-expired OTP
**WHEN** they submit the OTP within 10 minutes of issuance
**THEN** the account status transitions to `ACTIVE`, the OTP is consumed (single-use), and a 200 response confirms activation.

### AC-AUTH-004 (AUTH-002, US-002)
**GIVEN** a user submitting an expired or invalid OTP
**WHEN** they submit the verification request
**THEN** a 401 response is returned. After 5 failed attempts, the OTP is invalidated and a new one must be requested.

### AC-AUTH-005 (AUTH-003, US-003)
**GIVEN** a user with an active account and correct credentials
**WHEN** they submit login with email and password
**THEN** the response includes an access token (15-min expiry), a refresh token (30-day expiry), session ID, and device ID. A session record and device record are created.

### AC-AUTH-006 (AUTH-003, US-003)
**GIVEN** a user with more than 10 consecutive failed login attempts in 15 minutes
**WHEN** they attempt to login again
**THEN** the account is temporarily locked (429 response), lockout lasts 30 minutes, and an email notification is sent to the user.

---

## Messaging

### AC-MSG-001 (MSG-001, US-010)
**GIVEN** two authenticated users in a conversation, both online
**WHEN** User A sends a text message
**THEN** User B receives the message within 1 second, the message is persisted in PostgreSQL, and User A receives a server acknowledgement with the server-assigned message ID and timestamp.

### AC-MSG-002 (MSG-001, US-010)
**GIVEN** User A sends a message with the same `client_message_id` twice (retry scenario)
**WHEN** the server processes the second send
**THEN** only one message exists in the database, and the server returns the existing message as the acknowledgement (idempotency).

### AC-MSG-003 (MSG-002, US-011)
**GIVEN** User A has sent a message to User B, and User B's client has received it
**WHEN** User B's client sends a `message.delivered` event
**THEN** a delivery receipt is recorded with timestamp, and User A's client receives a `message.status` event updating the message to "delivered" (double check mark).

### AC-MSG-004 (MSG-003, US-012)
**GIVEN** a delivered message in User B's conversation
**WHEN** User B scrolls such that the message is in the visible viewport
**THEN** a `message.read` event is sent, User A receives a status update to "read" (blue double check mark).

### AC-MSG-005 (MSG-009, US-013)
**GIVEN** User B is offline when User A sends 3 messages
**WHEN** User B reconnects and sends `conversation.sync` with the ID of their last known message
**THEN** all 3 messages are delivered in chronological order within 2 seconds.

### AC-MSG-006 (MSG-004, US-014)
**GIVEN** User A sent a message less than 15 minutes ago
**WHEN** User A edits the message content
**THEN** all conversation members see the updated content, an "edited" indicator is displayed, and the edit history is preserved in `message_edits`.

### AC-MSG-007 (MSG-004, US-014)
**GIVEN** User A sent a message more than 15 minutes ago
**WHEN** User A attempts to edit it
**THEN** the edit is rejected with error code `EDIT_WINDOW_EXPIRED`.

### AC-MSG-008 (MSG-005, US-015)
**GIVEN** User A sent a message
**WHEN** User A deletes it
**THEN** all conversation members see "This message was deleted" in place of the original content. The message row remains in the database with `deleted_at` set and `content` nulled.

---

## Presence & Typing

### AC-PRES-001 (PRES-001, US-020)
**GIVEN** User B is offline
**WHEN** User B opens the app and establishes a WebSocket connection
**THEN** User B's presence changes to ONLINE within 2 seconds, and User A (in a shared conversation) receives a `presence.update` event.

### AC-PRES-002 (PRES-001, US-021)
**GIVEN** User B closes the app (WebSocket disconnects)
**WHEN** 30 seconds elapse without a heartbeat renewal
**THEN** User B's presence changes to OFFLINE, `last_seen_at` is updated, and User A receives a `presence.update`.

### AC-PRES-003 (PRES-003, US-022)
**GIVEN** User A and User B are in a conversation, both online
**WHEN** User A starts typing
**THEN** User B sees a "typing..." indicator within 1 second. The indicator disappears 5 seconds after User A stops typing (or immediately when the message is sent).

---

## Groups

### AC-GRP-001 (GRP-001, US-030)
**GIVEN** an authenticated user
**WHEN** they create a group with name "Project Team" and add Users B and C
**THEN** the group is created, a conversation is created, all 3 users are members, the creator has ADMIN role, and a system message "User A created the group" appears.

### AC-GRP-002 (GRP-003, US-032)
**GIVEN** User A is an admin of a group containing User B
**WHEN** User A removes User B
**THEN** User B is removed from the group and conversation, User B cannot send or receive group messages, a system message "User A removed User B" appears, and User B receives a notification.

---

## Media

### AC-MEDIA-001 (MEDIA-001, US-040)
**GIVEN** User A selects a 2MB JPEG image
**WHEN** they upload it in a conversation
**THEN** the upload completes in < 5 seconds, a thumbnail is generated, the message appears with the thumbnail in the chat, and tapping it loads the full-resolution image from a signed R2 URL.

### AC-MEDIA-002 (MEDIA-001)
**GIVEN** User A selects a file named `malware.exe` renamed to `photo.jpg`
**WHEN** they attempt to upload it
**THEN** server detects the file is not actually a JPEG (magic byte validation fails) and rejects with 422 and error code `INVALID_FILE_TYPE`.

---

## Calls

### AC-CALL-001 (CALL-001, US-050)
**GIVEN** User A and User B are online
**WHEN** User A initiates an audio call to User B
**THEN** User B receives an incoming call event within 2 seconds, and if accepted, audio flows between both parties within 8 seconds.

### AC-CALL-002 (CALL-004, US-052)
**GIVEN** User B receives an incoming call
**WHEN** User B rejects the call
**THEN** the call is recorded with status REJECTED, both parties return to the conversation view, and the call appears in call history.

### AC-CALL-003 (CALL-001)
**GIVEN** User A initiates a call and User B is unreachable
**WHEN** 30 seconds elapse without a response
**THEN** the call is marked as MISSED, User A is notified, and User B receives a push notification about the missed call.

---

*Next: [requirements-traceability.md](requirements-traceability.md)*
