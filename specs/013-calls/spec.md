# Phase 13 — 1-to-1 Calls

## Scope
- 1-to-1 audio/video calls only.
- Rust backend handles authenticated WebSocket signaling; clients handle WebRTC media.
- PostgreSQL persists call state/history.
- TURN credentials are time-limited HMAC credentials.
- Group calls/SFU are excluded.

## Events
Client → server: `call.offer`, `call.answer`, `call.connected`, `call.ice_candidate`, `call.reject`, `call.end`.
Server → client: `call.offer`, `call.answer`, `call.ice_candidate`, `call.ringing`, `call.rejected`, `call.ended`, `call.timeout`.

## State
CALLING → RINGING → CONNECTING → CONNECTED → ENDED, with REJECTED/MISSED/FAILED terminal states.

## Security
- Authenticated participants only.
- Blocked users cannot call each other.
- A user may participate in only one active call.
- SDP and ICE payloads are size-limited.
- Call events are rate-limited.
- TURN credentials expire after 24 hours.
