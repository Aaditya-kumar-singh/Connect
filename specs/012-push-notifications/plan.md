# Phase 12 Plan — Push Notifications

1. Inspect approved notification, authentication/device, messaging, and environment contracts.
2. Add notifications audit migration.
3. Add notification payload/job/provider abstractions.
4. Add console provider.
5. Add FCM HTTP v1 provider with service-account OAuth.
6. Add background worker, retries, and invalid-token cleanup.
7. Generate NEW_MESSAGE jobs for recipients without active WebSocket connections.
8. Respect conversation mute state.
9. Add unit/live integration tests.
10. Verify formatting, compilation, linting, tests, and diff.

Phase 13 will add call-specific notification events.
