# Phase 11 Tasks — Media

- [x] T001 Inspect media requirements and approved architecture.
- [x] T002 Add R2/MinIO configuration and shared S3 client.
- [x] T003 Add media_objects migration.
- [x] T004 Add message_attachments migration.
- [x] T005 Implement media type, size, extension, MIME, and magic-byte validation.
- [x] T006 Implement filename sanitization.
- [x] T007 Implement R2/MinIO original upload.
- [x] T008 Implement image metadata extraction and 300x300 WebP thumbnails.
- [x] T009 Re-encode supported images before storage to strip EXIF metadata.
- [x] T010 Persist media metadata and attachment-backed message.
- [x] T011 Broadcast media message through existing message.new conversation fan-out.
- [x] T012 Implement one-hour membership-protected signed download URL.
- [x] T013 Add media validation tests.
- [ ] T014 Run live PostgreSQL + Redis + MinIO/R2 multipart integration.
- [x] T015 Run fmt/check/clippy/test/diff verification.

T014 remains open until object storage and service-backed integration is available.
