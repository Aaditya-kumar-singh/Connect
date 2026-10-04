# Phase 11 Plan — Media

1. Inspect approved media, database, storage, and API contracts.
2. Add R2/MinIO configuration and shared S3 client.
3. Add media_objects and message_attachments migrations.
4. Implement strict media validation and filename sanitization.
5. Implement R2/MinIO upload and image thumbnail generation.
6. Persist media metadata and create an attachment-backed message.
7. Reuse existing WebSocket message fan-out.
8. Implement membership-protected one-hour signed downloads.
9. Add unit and live integration placeholders.
10. Verify fmt, check, clippy, tests, and diff.

Live object-storage integration remains pending until PostgreSQL, Redis, and MinIO/R2 are available.
