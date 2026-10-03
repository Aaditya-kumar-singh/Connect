# User Stories — YBM Connect

| Field             | Value                                      |
|-------------------|--------------------------------------------|
| **Document ID**   | `DOC-REQ-004`                              |
| **Version**       | `1.0.0`                                    |
| **Status**        | `APPROVED`                                 |
| **Owner**         | Engineering Lead                           |
| **Last Updated**  | 2026-10-02                                 |
| **Related Docs**  | DOC-REQ-002, DOC-REQ-005, DOC-REQ-006     |

---

## Authentication

| ID      | Story | Acceptance Criteria | Priority |
|---------|-------|---------------------|----------|
| US-001  | As a new user, I want to register with my email and password so that I can create an account. | Account created, verification email sent, cannot login until verified. | CRITICAL |
| US-002  | As a registered user, I want to verify my email with an OTP so that my account becomes active. | Valid OTP activates account; expired/invalid OTP shows error. | CRITICAL |
| US-003  | As a verified user, I want to log in with my email and password so that I can access the platform. | Correct credentials return tokens; incorrect credentials show error; account lock after 10 failed attempts. | CRITICAL |
| US-004  | As a logged-in user, I want my session to persist so that I don't have to log in every time I open the app. | Refresh token silently renews access token; session persists across app restarts. | HIGH |
| US-005  | As a user, I want to log out from my current device so that my session is terminated securely. | Session invalidated, tokens revoked, WebSocket disconnected. | HIGH |
| US-006  | As a user, I want to log out from all devices at once for security. | All sessions ended, all tokens revoked, all WebSocket connections closed. | MEDIUM |
| US-007  | As a user who forgot my password, I want to reset it via email OTP. | OTP sent, new password accepted, all sessions revoked, can login with new password. | HIGH |
| US-008  | As a user, I want to see and manage my active sessions/devices. | List of devices shown with last active time; can revoke individual sessions. | MEDIUM |

## Messaging

| ID      | Story | Acceptance Criteria | Priority |
|---------|-------|---------------------|----------|
| US-010  | As a user, I want to send a text message to another user so that we can communicate. | Message appears on recipient's screen in < 1 second; persists after server restart. | CRITICAL |
| US-011  | As a user, I want to see when my message has been delivered to the recipient. | Double-check indicator appears when server confirms delivery to recipient's device. | HIGH |
| US-012  | As a user, I want to see when my message has been read by the recipient. | Blue double-check indicator appears when recipient views the message. | HIGH |
| US-013  | As a user, I want to receive messages sent while I was offline when I come back online. | All missed messages appear in correct order on reconnection. | CRITICAL |
| US-014  | As a user, I want to edit a message I sent (within 15 minutes) to correct mistakes. | Edited message shows updated content with "edited" indicator; original sender only. | MEDIUM |
| US-015  | As a user, I want to delete a message I sent so that it's removed for everyone. | Message replaced with "This message was deleted" for all participants. | MEDIUM |
| US-016  | As a user, I want to react to a message with an emoji. | Reaction appears below the message; visible to all conversation members. | MEDIUM |
| US-017  | As a user, I want to reply to a specific message so that my response has context. | Reply shows quoted original message above the new message. | HIGH |
| US-018  | As a user, I want to forward a message to another conversation. | Message appears in target conversation marked as "Forwarded". | MEDIUM |
| US-019  | As a user, I want to see messages across all my devices. | Sending from device A shows the message on device B within 2 seconds. | HIGH |

## Presence & Typing

| ID      | Story | Acceptance Criteria | Priority |
|---------|-------|---------------------|----------|
| US-020  | As a user, I want to see which of my contacts are currently online. | Online indicator (green dot) visible next to online users. | MEDIUM |
| US-021  | As a user, I want to see when a contact was last active. | "Last seen" timestamp shown in conversation header. | MEDIUM |
| US-022  | As a user, I want to see when someone is typing in our conversation. | "typing..." indicator appears and disappears in < 2 seconds. | MEDIUM |

## Groups

| ID      | Story | Acceptance Criteria | Priority |
|---------|-------|---------------------|----------|
| US-030  | As a user, I want to create a group with a name and add members. | Group created, members added, all members see the group. | HIGH |
| US-031  | As a group admin, I want to add new members to the group. | New member appears in group, can see new messages from join point onward. | HIGH |
| US-032  | As a group admin, I want to remove members from the group. | Removed member can no longer see or send messages in the group. | HIGH |
| US-033  | As a group member, I want to leave the group voluntarily. | User removed from group, remaining members notified. | HIGH |
| US-034  | As a group admin, I want to change the group name and description. | Updated info visible to all members immediately. | MEDIUM |

## Media

| ID      | Story | Acceptance Criteria | Priority |
|---------|-------|---------------------|----------|
| US-040  | As a user, I want to send an image in a conversation. | Image thumbnail visible in chat; tapping opens full resolution. | HIGH |
| US-041  | As a user, I want to send a video in a conversation. | Video thumbnail visible; tapping plays the video. | HIGH |
| US-042  | As a user, I want to send a document (PDF, Word, etc.) in a conversation. | Document icon shown with filename; tapping downloads. | MEDIUM |
| US-043  | As a user, I want to send a voice message by pressing and holding a button. | Audio records while held; plays back with duration indicator. | HIGH |

## Calls

| ID      | Story | Acceptance Criteria | Priority |
|---------|-------|---------------------|----------|
| US-050  | As a user, I want to make an audio call to another user. | Call connects, both parties hear each other, call ends cleanly. | HIGH |
| US-051  | As a user, I want to make a video call to another user. | Call connects, both parties see and hear each other. | HIGH |
| US-052  | As a user receiving a call, I want to accept or reject it. | Accept starts call; reject ends it for both parties. | HIGH |
| US-053  | As a user, I want to see my call history (who, when, duration). | List of past calls with timestamps and duration. | MEDIUM |

## Notifications

| ID      | Story | Acceptance Criteria | Priority |
|---------|-------|---------------------|----------|
| US-060  | As a user, I want push notifications when I receive a message and the app is in the background. | Notification shows sender name and message preview. | HIGH |
| US-061  | As a user, I want a push notification (with ringtone) when someone calls me. | Full-screen incoming call UI appears. | HIGH |

---

*Next: [use-cases.md](use-cases.md) · [acceptance-criteria.md](acceptance-criteria.md)*
