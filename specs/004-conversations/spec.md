# Feature Specification: Conversations

**Feature Branch**: `004-conversations`
**Status**: Approved from roadmap/API/schema

## Scope

Implement authenticated 1-to-1 conversation creation and management. This phase does not implement messages, WebSockets, groups, presence, or real-time delivery.

## API

- `POST /api/v1/conversations`
- `GET /api/v1/conversations?limit=&cursor=`
- `GET /api/v1/conversations/{id}`

All endpoints require an authenticated session.

## Behaviour

- A direct conversation always contains exactly two active users.
- A user cannot create a conversation with themselves.
- Block relationships prevent creating a direct conversation.
- Creating a conversation for an existing pair returns the existing conversation rather than creating a duplicate.
- Duplicate prevention must remain safe under concurrent creation requests.
- Only active conversation members may retrieve conversation details.
- Conversation lists return only conversations in which the authenticated user is an active member.
- Listing is bounded and cursor-paginated.
- Conversation timestamps use server time.

## Data

Add `conversations` and `conversation_members` using the approved database schema. Existing user/block data is reused.

## Security

- Never expose password/authentication fields.
- Enforce membership on detail/list operations.
- Re-check blocking at creation.
- Do not add message or group behaviour in this phase.
