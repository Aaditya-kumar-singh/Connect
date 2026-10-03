# Business Requirements — YBM Connect

| Field             | Value                                      |
|-------------------|--------------------------------------------|
| **Document ID**   | `DOC-REQ-001`                              |
| **Version**       | `1.0.0`                                    |
| **Status**        | `APPROVED`                                 |
| **Owner**         | Engineering Lead                           |
| **Last Updated**  | 2026-10-02                                 |
| **Related Docs**  | DOC-OVR-001, DOC-REQ-002, DOC-REQ-003     |
| **Related ADRs**  | ADR-011, ADR-013                           |

---

## Business Context

YBM Connect is a portfolio/learning project that demonstrates the ability to design and build a complex distributed system. The "business" requirements are framed as if building a product, because production-quality thinking from day one prevents architectural shortcuts that become expensive to fix later.

## Business Requirements

### BR-001: Cross-Platform Instant Messaging
**Requirement:** Users must be able to send and receive text messages in real-time across web and Android platforms.
**Business reason:** Core value proposition. A messaging platform that cannot deliver messages instantly has no value.
**Success metric:** p95 message delivery < 1 second within the same region.
**Priority:** CRITICAL

### BR-002: Reliable Message Delivery
**Requirement:** Every message sent must eventually reach its intended recipient(s), even if the recipient is temporarily offline.
**Business reason:** Users lose trust in a messaging platform that drops messages. Message reliability is non-negotiable.
**Success metric:** 0% message loss under normal operation; offline messages delivered within 5 seconds of reconnection.
**Priority:** CRITICAL

### BR-003: Rich Media Communication
**Requirement:** Users must be able to share images, videos, documents, and voice messages within conversations.
**Business reason:** Text-only messaging is insufficient for modern communication. Media sharing is expected functionality.
**Success metric:** Media upload completes within 10 seconds for files < 5MB.
**Priority:** HIGH

### BR-004: Voice and Video Calling
**Requirement:** Users must be able to initiate 1-to-1 voice and video calls with other users on any platform.
**Business reason:** Calls reduce the need for users to switch to another application. Increases platform stickiness.
**Success metric:** Call connects in < 8 seconds; audio quality comparable to standard VoIP.
**Priority:** HIGH

### BR-005: Secure Authentication
**Requirement:** Users must be able to register with email/password, verify their email with OTP, and log in securely from multiple devices.
**Business reason:** Security breaches destroy trust. Authentication must be robust from launch.
**Success metric:** No unauthorized access in security testing; brute-force login blocked.
**Priority:** CRITICAL

### BR-006: Group Communication
**Requirement:** Users must be able to create groups, manage membership, assign roles, and communicate within groups.
**Business reason:** Group messaging is a core feature of modern messaging platforms. Users expect it.
**Success metric:** Groups support up to 256 members with sub-second message delivery.
**Priority:** HIGH

### BR-007: User Awareness
**Requirement:** Users must see who is online, who was last seen when, and who is currently typing.
**Business reason:** Presence and typing indicators create a feeling of real-time connection and reduce communication friction.
**Success metric:** Presence updates within 5 seconds; typing indicators within 1 second.
**Priority:** MEDIUM

### BR-008: Notification Delivery
**Requirement:** Users must receive push notifications for new messages and calls when the application is not in the foreground.
**Business reason:** Without notifications, users miss messages and the platform feels unreliable.
**Success metric:** Notifications delivered within 5 seconds of message receipt by server.
**Priority:** HIGH

### BR-009: Multi-Device Support
**Requirement:** A single user account must support multiple simultaneous devices (e.g., phone + browser). Messages must sync across all devices.
**Business reason:** Users switch between devices. Requiring re-login or missing messages on other devices creates friction.
**Success metric:** Messages appear on all active devices within 2 seconds.
**Priority:** HIGH

### BR-010: System Reliability
**Requirement:** The system must maintain availability during component failures and recover without manual intervention where possible.
**Business reason:** Downtime equals user loss. Even a portfolio project should demonstrate production-grade reliability patterns.
**Success metric:** System recovers from Redis restart within 30 seconds. No message loss during recovery.
**Priority:** HIGH

### BR-011: Developer Onboarding
**Requirement:** A new developer (human or AI) must be able to understand the architecture, set up the development environment, and begin contributing within 15 minutes.
**Business reason:** Fast onboarding accelerates development velocity and reduces bus factor.
**Success metric:** Clone-to-running in < 15 minutes with documented steps.
**Priority:** MEDIUM

---

*Next: [functional-requirements.md](functional-requirements.md)*
