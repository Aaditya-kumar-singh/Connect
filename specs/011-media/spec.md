# Phase 11 — Media

## Goal
Upload and download images, videos, documents, and voice messages through Cloudflare R2 or local MinIO.

## Scope
- Authenticated multipart upload.
- Conversation membership authorization.
- Strict extension + MIME + magic-byte validation.
- Per-type upload size limits.
- Filename sanitization.
- R2/MinIO object storage.
- Image metadata extraction and 300x300 WebP thumbnails.
- Image re-encoding to remove EXIF metadata.
- Media metadata persisted in PostgreSQL.
- Uploaded media creates a message and message attachment.
- Media messages use the existing message.new conversation fan-out.
- Signed download URLs expire after one hour.
- R2 keys remain internal.

## APIs
- POST /api/v1/media/upload
- GET /api/v1/media/{id}/url

## Storage
- Bucket: configured by R2_BUCKET_NAME.
- Object key: {conversation_id}/{media_id}/original.{extension}
- Thumbnail key: {conversation_id}/{media_id}/thumbnail.webp

## Limits
- Images: 10 MB.
- Videos: 50 MB.
- Documents: 25 MB.
- Voice: 10 MB.
- Voice duration is limited by the V1 client/server contract to five minutes; duration extraction remains a media-parser concern.

## Security
- Never trust filename or Content-Type alone.
- Reject magic-byte mismatches.
- Reject unsupported types and blocked executable/script/archive formats.
- Only conversation members can upload.
- Only members of a conversation containing the attachment can request its signed URL.
- Never expose internal R2 keys.

## Out of Scope
- Malware scanning provider integration.
- Video transcoding.
- Push notifications.
- Media CDN.
- Background cleanup worker.
- Avatar/group-avatar upload.
