# Phase 12 — Push Notifications

## Goal
Deliver background message notifications through a provider abstraction with durable notification audit records.

## Scope
- Notification audit records.
- Background Tokio notification worker.
- Provider trait.
- Console provider for local development.
- FCM HTTP v1 provider for Android.
- Invalid FCM token removal.
- Temporary provider retry with bounded exponential backoff.
- New-message notification generation only when the recipient has no active WebSocket connection.
- Conversation mute suppression.
- Direct and group message payloads.
- Device push-token reuse from Phase 3.

## APIs
Push tokens continue to use the existing:
- PATCH /api/v1/devices/{device_id}

## Notification payload
- NEW_MESSAGE
- sender name
- first 100 characters of message content, or [media]
- conversation_id
- message_id
- sender_id
- conversation tag

## Security
- Never log push tokens.
- FCM credentials remain server-side.
- Notification records contain structured data only.
- Invalid tokens are removed from the device record.

## Dependencies
- Phase 3 device push-token storage.
- Phase 6 messaging.
- Phase 13 call events will later enqueue INCOMING_CALL and MISSED_CALL jobs.

## Known integration boundary
Web Push provider wiring is retained as a provider slot but is not enabled in this backend build yet because the approved architecture requires browser subscription encryption/VAPID support and the current project has no browser subscription registration endpoint beyond the generic device token field.
