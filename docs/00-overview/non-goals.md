# Non-Goals — YBM Connect

| Field             | Value                                      |
|-------------------|--------------------------------------------|
| **Document ID**   | `DOC-OVR-004`                              |
| **Version**       | `1.0.0`                                    |
| **Status**        | `APPROVED`                                 |
| **Owner**         | Engineering Lead                           |
| **Last Updated**  | 2026-10-02                                 |
| **Related Docs**  | DOC-OVR-001, DOC-OVR-003                  |

---

## Purpose

Non-goals are explicitly excluded features, approaches, or properties. Documenting them prevents scope creep and clarifies architectural decisions.

## Non-Goals

### NG-001: End-to-End Encryption (E2EE)
**What:** Signal Protocol or equivalent client-side encryption where the server cannot read message content.
**Why excluded:** E2EE fundamentally changes key management, device verification, group messaging protocols, and backup strategy. It is a feature-sized project on its own. Including it from day one would triple initial complexity.
**Reconsider when:** Core messaging is stable, multi-device sync works reliably, and key management UX has been designed.
**V2 candidate:** Yes.

### NG-002: Group Audio/Video Calls
**What:** Multi-party calls using an SFU (Selective Forwarding Unit).
**Why excluded:** Group calls require a media server (SFU), which is a separate infrastructure component with different scaling, CPU, and bandwidth characteristics. 1-to-1 calls use P2P WebRTC; group calls require server-side media routing.
**Reconsider when:** 1-to-1 calls are stable and there is demand for group calling.
**V2 candidate:** Yes.

### NG-003: iOS Application
**What:** Native or React Native iOS client.
**Why excluded:** React Native/Expo supports iOS, but testing, signing, and App Store deployment add significant overhead. Android is prioritized for initial mobile development.
**Reconsider when:** Android app is stable and feature-complete.
**V2 candidate:** Yes.

### NG-004: Desktop Native Application
**What:** Electron, Tauri, or native desktop clients.
**Why excluded:** The web application, accessed via a desktop browser, provides the desktop experience. A native wrapper adds deployment complexity without significant user-facing benefit at this stage.
**Reconsider when:** Users need OS-level integration (system tray, native notifications) beyond what the web app provides.

### NG-005: Status/Stories
**What:** Ephemeral content (images, text, video) visible for 24 hours.
**Why excluded:** A separate content type with its own storage, expiration, privacy controls, and UI. Not part of core messaging.
**Reconsider when:** Core platform is feature-complete and there is product demand.

### NG-006: Self-Destructing Messages
**What:** Messages that automatically delete after a set time.
**Why excluded:** Requires client-side enforcement (which is inherently bypassable), server-side scheduling, and changes to the message lifecycle state machine. Adds complexity to backup/recovery.
**Reconsider when:** After E2EE is implemented.

### NG-007: WhatsApp Internal Architecture Replication
**What:** Attempting to copy WhatsApp's internal Erlang/BEAM architecture, custom protocol, or proprietary optimizations.
**Why excluded:** WhatsApp's internals are not publicly documented in sufficient detail. Cargo-culting assumed internals leads to poor engineering decisions. Our architecture is independently designed using public distributed-systems principles.
**Reconsider when:** Never. Design independently.

### NG-008: Blockchain/Cryptocurrency
**What:** Any blockchain-based features.
**Why excluded:** No engineering requirement. Adds complexity without value for a messaging platform.
**Reconsider when:** Never for core platform.

### NG-009: AI-Powered Features
**What:** AI chatbots, smart reply, message summarization.
**Why excluded:** Not core to the communication platform. Can be layered on later without architectural changes.
**Reconsider when:** Core platform is stable and there is product interest.

### NG-010: Multi-Region Deployment
**What:** Active-active or active-passive multi-region deployment.
**Why excluded:** Requires distributed database (CockroachDB/Spanner), global load balancing, and conflict resolution. Enormous complexity for a portfolio project.
**Reconsider when:** Single-region deployment reaches capacity limits or latency requirements demand geographic distribution.

### NG-011: Federation
**What:** Interoperability with other messaging platforms (Matrix, XMPP).
**Why excluded:** Federation protocols add protocol translation, identity management, and trust layers. Not aligned with project goals.
**Reconsider when:** Not planned.

### NG-012: Admin Dashboard UI
**What:** Web-based admin panel for user management, system monitoring.
**Why excluded:** CLI tools and direct API access suffice initially. A dashboard is a separate frontend project.
**Reconsider when:** When operational needs justify a dedicated UI.

---

*Next: [terminology.md](terminology.md) · [glossary.md](glossary.md)*
