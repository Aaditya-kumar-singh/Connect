# Tasks: Conversations

## Data
- [x] T001 Add reversible conversations migration.
- [x] T002 Add reversible conversation_members migration/indexes.

## API
- [x] T003 Implement conversation models and validation.
- [x] T004 Implement direct conversation creation with concurrency-safe duplicate prevention.
- [x] T005 Implement cursor-paginated conversation listing.
- [x] T006 Implement member-only conversation detail.

## Verification
- [x] T007 Add integration tests for create/duplicate/membership/block/pagination.
- [x] T008 Run cargo fmt -- --check.
- [x] T009 Run cargo clippy -- -D warnings.
- [ ] T010 Run cargo test with PostgreSQL and Redis services. Blocked on this workstation because Docker is not installed; integration tests are compiled and ignored until TEST_DATABASE_URL and TEST_REDIS_URL are supplied.
- [x] T011 Audit diff against spec, plan, AGENTS.md, GEMINI.md, and Ponytail rules.
