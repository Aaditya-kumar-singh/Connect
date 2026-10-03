# Glossary — YBM Connect

| Field             | Value                                      |
|-------------------|--------------------------------------------|
| **Document ID**   | `DOC-OVR-006`                              |
| **Version**       | `1.0.0`                                    |
| **Status**        | `APPROVED`                                 |
| **Owner**         | Engineering Lead                           |
| **Last Updated**  | 2026-10-02                                 |
| **Related Docs**  | DOC-OVR-005, DOC-REF-001                   |

---

| Term | Definition |
|------|-----------|
| **ACK** | Acknowledgement. A response confirming receipt of a message or event. |
| **ADR** | Architecture Decision Record. A document capturing a significant architectural choice. |
| **Argon2id** | A memory-hard password hashing algorithm. The recommended variant combining Argon2i (side-channel resistance) and Argon2d (GPU resistance). |
| **Axum** | A Rust web framework built on Tokio and Tower. Used as the HTTP/WebSocket server. |
| **Backoff** | A strategy for increasing delay between retry attempts, typically exponential. |
| **Circuit Breaker** | A pattern that monitors failures to an external service and stops sending requests when the failure rate exceeds a threshold. States: Closed (normal), Open (blocking), Half-Open (testing recovery). |
| **Client Message ID** | A UUID generated client-side before sending a message. Used for idempotency and deduplication. |
| **coturn** | An open-source TURN/STUN server for WebRTC NAT traversal. |
| **CSRF** | Cross-Site Request Forgery. An attack where a malicious site tricks a user's browser into making requests to a different site. |
| **DTLS** | Datagram Transport Layer Security. TLS for UDP, used by WebRTC media. |
| **Durable Object** | A Cloudflare Workers feature providing single-instance coordination with persistent storage. |
| **E2EE** | End-to-End Encryption. Encryption where only communicating parties can read messages. |
| **FCM** | Firebase Cloud Messaging. Google's push notification service for Android and web. |
| **ICE** | Interactive Connectivity Establishment. A WebRTC framework for finding the best path between peers. |
| **IDOR** | Insecure Direct Object Reference. An access control vulnerability where object IDs are exposed without authorization checks. |
| **Idempotency** | The property that performing an operation multiple times produces the same result as performing it once. |
| **Jitter** | Random variation added to retry delays to prevent thundering herd problems. |
| **JWT** | JSON Web Token. A compact, URL-safe token format (not used as primary session tokens in YBM Connect — see ADR-014). |
| **Modular Monolith** | An architecture where all components run in a single process but are organized into modules with clear boundaries and contracts. |
| **NAT** | Network Address Translation. A router function that maps private IP addresses to public ones. WebRTC must traverse NATs. |
| **OTP** | One-Time Password. A temporary code sent via email for verification. |
| **P2P** | Peer-to-Peer. Direct communication between two devices without server relay. |
| **Pub/Sub** | Publish/Subscribe. A messaging pattern where senders publish to channels and receivers subscribe. Redis provides this. |
| **R2** | Cloudflare's S3-compatible object storage service with zero egress fees. |
| **Redis Streams** | A Redis data structure for append-only log processing, similar to Kafka topics. |
| **RPO** | Recovery Point Objective. Maximum acceptable data loss measured in time. |
| **RTO** | Recovery Time Objective. Maximum acceptable downtime. |
| **SDP** | Session Description Protocol. Describes multimedia communication sessions for WebRTC. |
| **SFU** | Selective Forwarding Unit. A server that receives media streams and forwards them to other participants (used in group calls). |
| **SRTP** | Secure Real-time Transport Protocol. Encrypted RTP for WebRTC media. |
| **SSRF** | Server-Side Request Forgery. An attack where the server is tricked into making requests to unintended destinations. |
| **STUN** | Session Traversal Utilities for NAT. A protocol that discovers a client's public IP and port. |
| **Supervision Tree** | A hierarchy of supervisors and workers where each supervisor monitors its children and restarts them according to a strategy. |
| **Tokio** | An asynchronous runtime for Rust, providing task scheduling, I/O, timers, and synchronization primitives. |
| **Tower** | A Rust library of modular, reusable components for building network services. Axum is built on Tower. |
| **TURN** | Traversal Using Relays around NAT. A protocol that relays media through a server when P2P is impossible. |
| **WAF** | Web Application Firewall. Cloudflare provides WAF at the edge. |
| **WebRTC** | Web Real-Time Communication. A set of APIs and protocols for P2P audio, video, and data communication. |
| **WebSocket** | A protocol providing full-duplex communication over a single TCP connection. |
| **Worker (Cloudflare)** | Cloudflare's serverless edge compute platform. Not to be confused with YBM Connect's internal worker tasks. |
| **Worker (YBM Connect)** | A supervised Tokio task that handles a specific background responsibility (e.g., presence, notifications). |
| **WSS** | WebSocket Secure. WebSocket over TLS. |
| **XSS** | Cross-Site Scripting. An attack where malicious scripts are injected into trusted web pages. |

---

*Next: [success-criteria.md](success-criteria.md)*
