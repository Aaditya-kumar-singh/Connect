# Terminology — YBM Connect

| Field             | Value                                      |
|-------------------|--------------------------------------------|
| **Document ID**   | `DOC-OVR-005`                              |
| **Version**       | `1.0.0`                                    |
| **Status**        | `APPROVED`                                 |
| **Owner**         | Engineering Lead                           |
| **Last Updated**  | 2026-10-02                                 |
| **Related Docs**  | DOC-OVR-006 (Glossary)                     |

---

## Conventions Used in This Documentation

### Requirement IDs
Every requirement uses a prefix indicating its domain:

| Prefix   | Domain                  | Example     |
|----------|-------------------------|-------------|
| `AUTH-`  | Authentication          | AUTH-001    |
| `MSG-`   | Messaging               | MSG-001     |
| `GRP-`   | Groups                  | GRP-001     |
| `MEDIA-` | Media                   | MEDIA-001   |
| `CALL-`  | Calls                   | CALL-001    |
| `PRES-`  | Presence                | PRES-001    |
| `SEC-`   | Security                | SEC-001     |
| `PERF-`  | Performance             | PERF-001    |
| `OBS-`   | Observability           | OBS-001     |
| `DEV-`   | Development/DevOps      | DEV-001     |
| `NFR-`   | Non-functional          | NFR-001     |
| `UI-`    | User Interface          | UI-001      |
| `NOTIF-` | Notifications           | NOTIF-001   |
| `DB-`    | Database                | DB-001      |

### Document IDs
Documents use the format `DOC-{SECTION}-{NUMBER}`:
- `DOC-OVR-001` = Overview section, document 1
- `DOC-ARCH-001` = Architecture section, document 1

### ADR IDs
Architecture Decision Records use `ADR-{NUMBER}`:
- `ADR-001` = First architecture decision

### Diagrams
All diagrams use Mermaid syntax unless otherwise noted. Diagrams are embedded directly in markdown documents.

### Status Labels

| Status       | Meaning                                           |
|--------------|---------------------------------------------------|
| `DRAFT`      | Under development, may change significantly       |
| `REVIEW`     | Complete but awaiting review                       |
| `APPROVED`   | Reviewed and accepted as authoritative             |
| `DEPRECATED` | Superseded by a newer document                     |

### Severity/Priority

| Level      | Meaning                                            |
|------------|----------------------------------------------------|
| `CRITICAL` | System is unusable; immediate action required       |
| `HIGH`     | Major feature broken; workaround may exist          |
| `MEDIUM`   | Feature degraded; acceptable short-term             |
| `LOW`      | Minor issue; cosmetic or non-urgent                 |

### Domain Terms

These terms have specific meanings in YBM Connect:

| Term              | Definition                                                                  |
|-------------------|-----------------------------------------------------------------------------|
| **Conversation**  | A communication channel between two or more users. Can be 1-to-1 or group. |
| **Message**       | A unit of content (text, media reference) sent within a conversation.       |
| **Device**        | A specific browser or mobile app instance registered to a user.             |
| **Session**       | An authenticated login context on a specific device.                        |
| **Presence**      | A user's current online/offline state.                                      |
| **Receipt**       | A delivery or read acknowledgement for a message.                           |
| **Reaction**      | An emoji response attached to a specific message.                           |
| **Worker**        | A supervised background task handling a specific responsibility.            |
| **Supervisor**    | A task that monitors and restarts workers on failure.                        |
| **Circuit breaker** | A pattern that stops calling a failing service to allow recovery.         |

---

*Next: [glossary.md](glossary.md)*
