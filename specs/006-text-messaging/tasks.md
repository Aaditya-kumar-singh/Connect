# Phase 6 Tasks

- [x] T001 Create Phase 6 spec/plan/tasks.
- [x] T002 Add messages/message_edits/message_reactions migrations and message FK.
- [x] T003 Add messaging models and validation.
- [x] T004 Add message repository with cursor history and idempotent insert.
- [x] T005 Add send/edit/delete/reaction service operations.
- [x] T006 Add local connection-manager delivery helpers.
- [x] T007 Extend Redis pub/sub for cross-instance message delivery.
- [x] T008 Add WebSocket message event dispatcher.
- [x] T009 Add message history REST endpoint.
- [x] T010 Add message forward REST endpoint.
- [x] T011 Add focused unit tests.
- [ ] T012 Add ignored live messaging integration test.
- [x] T013 Run fmt/check/clippy/tests/diff audit and verify Phase 7+ scope is untouched.

T012 remains open because PostgreSQL/Redis services and an authenticated live WebSocket test environment are not currently verified. No speculative runtime infrastructure was added solely to force the test.
