# YBM Connect Reliability Service

Erlang/OTP 27 sidecar for infrastructure-level fault tolerance.

## Responsibilities

- OTP supervision
- PostgreSQL/Redis/R2 health monitoring
- Circuit breakers
- Recovery coordination
- Aggregated reliability health

The service contains no application/business logic and is advisory. The Rust backend remains operational if this service is unavailable.

## Development

Requirements:

- Erlang/OTP 27+
- rebar3
- Redis

Commands:

    rebar3 compile
    rebar3 eunit
    rebar3 ct
    rebar3 shell

## Configuration

- REDIS_HOST
- REDIS_PORT
- REDIS_PASSWORD
- PG_HOST
- PG_PORT
- R2_ENDPOINT
- HEALTH_PORT
- PROBE_INTERVAL_PG
- PROBE_INTERVAL_REDIS
- PROBE_INTERVAL_STORAGE

The Rust/Erlang boundary uses Redis Pub/Sub channels:

- ybm:reliability:events
- ybm:reliability:status
