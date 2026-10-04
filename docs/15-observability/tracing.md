# YBM Connect — Request Tracing

The Rust backend uses `tracing` and `tracing-subscriber` with JSON output.

Each HTTP request has:

- method
- path without the query string
- request ID
- response status and timing through the HTTP trace layer

The request ID is generated or preserved by the existing `x-request-id` middleware and propagated to the response.

Phase 16 does not add an OpenTelemetry collector or external trace backend. Distributed trace export remains a later hardening task.
