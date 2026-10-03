# Requirements Traceability Matrix — YBM Connect

| Field             | Value                                      |
|-------------------|--------------------------------------------|
| **Document ID**   | `DOC-REQ-007`                              |
| **Version**       | `1.0.0`                                    |
| **Status**        | `APPROVED`                                 |
| **Owner**         | Engineering Lead                           |
| **Last Updated**  | 2026-10-02                                 |
| **Related Docs**  | All DOC-REQ-* documents                    |

---

## Purpose

This matrix traces every requirement from business need → functional requirement → user story → acceptance criteria → API → database → code module → test → deployment phase.

## Authentication Traceability

| Business Req | Func Req | User Story | Acceptance | API | DB Tables | Code Module | Test | Phase |
|---|---|---|---|---|---|---|---|---|
| BR-005 | AUTH-001 | US-001 | AC-AUTH-001, AC-AUTH-002 | `POST /api/v1/auth/register` | users, email_verifications | `auth::registration` | unit, integration, security | Phase 2 |
| BR-005 | AUTH-002 | US-002 | AC-AUTH-003, AC-AUTH-004 | `POST /api/v1/auth/verify-email` | users, email_verifications | `auth::verification` | unit, integration | Phase 2 |
| BR-005 | AUTH-003 | US-003 | AC-AUTH-005, AC-AUTH-006 | `POST /api/v1/auth/login` | users, sessions, devices, refresh_tokens | `auth::login` | unit, integration, security | Phase 2 |
| BR-005 | AUTH-004 | US-004 | — | `POST /api/v1/auth/refresh` | refresh_tokens | `auth::token` | unit, integration | Phase 2 |
| BR-005 | AUTH-005 | US-005 | — | `POST /api/v1/auth/logout` | sessions, refresh_tokens | `auth::session` | unit, integration | Phase 2 |
| BR-005 | AUTH-006 | US-006 | — | `POST /api/v1/auth/logout-all` | sessions, refresh_tokens | `auth::session` | unit, integration | Phase 2 |
| BR-005 | AUTH-007 | US-007 | — | `POST /api/v1/auth/forgot-password` | otp_requests | `auth::recovery` | unit, integration | Phase 2 |
| BR-005 | AUTH-008 | US-007 | — | `POST /api/v1/auth/reset-password` | users, sessions, refresh_tokens | `auth::recovery` | unit, integration, security | Phase 2 |

## Messaging Traceability

| Business Req | Func Req | User Story | Acceptance | API/Event | DB Tables | Code Module | Test | Phase |
|---|---|---|---|---|---|---|---|---|
| BR-001, BR-002 | MSG-001 | US-010 | AC-MSG-001, AC-MSG-002 | WS `message.send` | messages | `messaging::send` | unit, integration, load | Phase 6 |
| BR-002 | MSG-002 | US-011 | AC-MSG-003 | WS `message.delivered` | message_receipts | `messaging::receipts` | unit, integration | Phase 7 |
| BR-002 | MSG-003 | US-012 | AC-MSG-004 | WS `message.read` | message_receipts | `messaging::receipts` | unit, integration | Phase 7 |
| BR-001 | MSG-004 | US-014 | AC-MSG-006, AC-MSG-007 | WS `message.edit` | messages, message_edits | `messaging::edit` | unit, integration | Phase 6 |
| BR-001 | MSG-005 | US-015 | AC-MSG-008 | WS `message.delete` | messages | `messaging::delete` | unit, integration | Phase 6 |
| BR-001 | MSG-006 | US-016 | — | WS `message.react` | message_reactions | `messaging::reactions` | unit, integration | Phase 6 |
| BR-001 | MSG-007 | US-017 | — | WS `message.send` (reply) | messages | `messaging::send` | unit, integration | Phase 6 |
| BR-001 | MSG-008 | US-018 | — | `POST /api/v1/messages/forward` | messages, message_attachments | `messaging::forward` | unit, integration | Phase 6 |
| BR-002 | MSG-009 | US-013 | AC-MSG-005 | WS `conversation.sync` | messages | `messaging::sync` | unit, integration, chaos | Phase 6 |

## Presence & Typing Traceability

| Business Req | Func Req | User Story | Acceptance | API/Event | DB/Redis | Code Module | Test | Phase |
|---|---|---|---|---|---|---|---|---|
| BR-007 | PRES-001 | US-020 | AC-PRES-001, AC-PRES-002 | WS `presence.update` | Redis + users.last_seen_at | `presence::tracker` | unit, integration | Phase 8 |
| BR-007 | PRES-002 | US-021 | AC-PRES-002 | REST profile query | users.last_seen_at | `users::profile` | unit | Phase 8 |
| BR-007 | PRES-003 | US-022 | AC-PRES-003 | WS `typing.start/stop` | Redis TTL key | `presence::typing` | unit, integration | Phase 9 |

## Group Traceability

| Business Req | Func Req | User Story | Acceptance | API | DB Tables | Code Module | Test | Phase |
|---|---|---|---|---|---|---|---|---|
| BR-006 | GRP-001 | US-030 | AC-GRP-001 | `POST /api/v1/groups` | groups, group_members, conversations, conversation_members | `groups::create` | unit, integration | Phase 10 |
| BR-006 | GRP-002 | US-031 | — | `POST /api/v1/groups/{id}/members` | group_members, conversation_members | `groups::membership` | unit, integration | Phase 10 |
| BR-006 | GRP-003 | US-032 | AC-GRP-002 | `DELETE /api/v1/groups/{id}/members/{uid}` | group_members, conversation_members | `groups::membership` | unit, integration | Phase 10 |
| BR-006 | GRP-004 | US-033 | — | `POST /api/v1/groups/{id}/leave` | group_members, conversation_members | `groups::membership` | unit, integration | Phase 10 |

## Media Traceability

| Business Req | Func Req | User Story | Acceptance | API | DB Tables | Code Module | Test | Phase |
|---|---|---|---|---|---|---|---|---|
| BR-003 | MEDIA-001 | US-040 | AC-MEDIA-001, AC-MEDIA-002 | `POST /api/v1/media/upload` | media_objects, messages, message_attachments | `media::upload` | unit, integration, security | Phase 11 |
| BR-003 | MEDIA-005 | US-040 | — | `GET /api/v1/media/{id}/url` | media_objects | `media::download` | unit, integration | Phase 11 |

## Calls Traceability

| Business Req | Func Req | User Story | Acceptance | API/Event | DB Tables | Code Module | Test | Phase |
|---|---|---|---|---|---|---|---|---|
| BR-004 | CALL-001 | US-050 | AC-CALL-001 | WS `call.offer` | calls, call_participants | `calls::signaling` | unit, integration, WebRTC | Phase 13 |
| BR-004 | CALL-003 | US-052 | — | WS `call.answer` | calls | `calls::signaling` | unit, integration | Phase 13 |
| BR-004 | CALL-004 | US-052 | AC-CALL-002 | WS `call.reject` | calls | `calls::signaling` | unit, integration | Phase 13 |
| BR-004 | CALL-006 | US-053 | — | `GET /api/v1/calls/history` | calls, call_participants | `calls::history` | unit, integration | Phase 13 |

---

*Next: [architecture-overview.md](../02-architecture/architecture-overview.md)*
