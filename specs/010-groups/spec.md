# Phase 10 — Groups

## Goal
Support multi-member group conversations with explicit ADMIN/MEMBER roles and group metadata.

## Scope
- Create group and its GROUP conversation atomically.
- Initial membership includes creator plus requested users.
- Creator is ADMIN; other members are MEMBER.
- Group name: 3-100 characters.
- Description: up to 500 characters.
- Maximum 256 members.
- Maximum 100 active groups per creator.
- Get group details and active members.
- Admin-only member addition/removal.
- Admin/member roles.
- Last-admin protection.
- Member leave with automatic promotion of the longest-serving remaining member when the last admin leaves.
- Admin-only group metadata update.
- Group soft deletion.
- Group state changes create system messages in the group conversation.
- Existing conversation membership authorization automatically governs group message send/history/receipts.
- Existing WebSocket message fan-out is reused for group messaging.

## APIs
- POST /api/v1/groups
- GET /api/v1/groups/{id}
- PATCH /api/v1/groups/{id}
- DELETE /api/v1/groups/{id}
- POST /api/v1/groups/{id}/members
- DELETE /api/v1/groups/{id}/members/{user_id}
- POST /api/v1/groups/{id}/leave
- PATCH /api/v1/groups/{id}/members/{user_id}/role

## Authorization
- Only active group members can view group details.
- ADMIN is required for add/remove/update/delete/change-role.
- Members cannot remove themselves through the admin remove endpoint.
- Admins cannot remove another ADMIN.
- The final ADMIN cannot demote themselves.
- A group always retains at least one ADMIN while it has members.
- Users blocked in either direction cannot be added to a group.

## System Messages
content_type = system, sender_id = NULL.
Examples:
- Alice created the group
- Alice added Bob to the group
- Alice removed Bob from the group
- Bob left the group
- Alice changed the group name to New Name
- Alice made Bob an admin

## Data
- groups stores metadata and links one-to-one to conversations.
- group_members stores ADMIN/MEMBER membership.
- groups.deleted_at supports soft deletion.
- messages.sender_id becomes nullable to support system messages.

## Out of Scope
- Media upload/avatar generation.
- Push notification delivery.
- Group-specific WebSocket event types.
- Call features.
- New message delivery/read receipt semantics.
