# Project Goals — YBM Connect

| Field             | Value                                      |
|-------------------|--------------------------------------------|
| **Document ID**   | `DOC-OVR-002`                              |
| **Version**       | `1.0.0`                                    |
| **Status**        | `APPROVED`                                 |
| **Owner**         | Engineering Lead                           |
| **Last Updated**  | 2026-10-02                                 |
| **Related Docs**  | DOC-OVR-001, DOC-OVR-003, DOC-PROD-001    |
| **Related ADRs**  | ADR-001, ADR-011, ADR-013                  |

---

## Primary Goals

### GOAL-001: Real-Time Cross-Platform Communication
Build a system where users on any supported platform (web, Android) can exchange text messages, media, and conduct voice/video calls in real-time with sub-second delivery latency.

### GOAL-002: Production-Grade Architecture
Design internal architecture (module boundaries, data flow, error handling, security) at production quality from day one, even while targeting portfolio/learning scope for initial deployment.

### GOAL-003: Incremental Buildability
Structure the project so that each feature (auth → messaging → calls) can be built, tested, and deployed independently without requiring the entire system to be complete.

### GOAL-004: Developer Experience
Maintain documentation, tooling, and local development setup such that a developer can understand the architecture, set up the environment, and begin contributing within 15 minutes.

### GOAL-005: Fault Tolerance
Ensure the system degrades gracefully under failure conditions (database outage, Redis failure, network partition) rather than failing catastrophically. No single component failure should cause complete system unavailability.

### GOAL-006: Security by Default
Ship every feature with authentication, authorization, input validation, rate limiting, and secure defaults. Security is not a phase — it is a property of every component.

### GOAL-007: Observability
Every component produces structured logs, metrics, and traces. Any production issue can be diagnosed by examining telemetry without requiring code changes or redeployment.

### GOAL-008: Scalability Path
While the initial deployment targets a single server, the architecture must not contain decisions that prevent horizontal scaling. When scaling is needed, it should require infrastructure changes, not architecture rewrites.

## Secondary Goals

### GOAL-009: Portfolio & Learning Value
The project demonstrates mastery of: Rust async systems, real-time protocols, relational database design, distributed system patterns, WebRTC, security engineering, and operational practices.

### GOAL-010: Documentation as Engineering Artifact
The documentation system itself serves as a reference implementation of how to document a complex system — useful for future projects and team onboarding.

### GOAL-011: Cost Efficiency
Leverage free tiers (Cloudflare, managed databases) for development and staging. Production costs should be predictable and proportional to usage.

## Explicit Non-Goals (at this stage)

See [non-goals.md](non-goals.md) for the full list. Key exclusions:

- End-to-end encryption (E2EE) — deferred to V2
- Group video/audio calls — V2
- Status/stories feature — V2
- Desktop native apps — future
- Payment/commerce integration — out of scope
- Blockchain/cryptocurrency — out of scope

---

*Next: [project-scope.md](project-scope.md) · [non-goals.md](non-goals.md)*
