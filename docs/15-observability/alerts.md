# YBM Connect — Alert Rules

Prometheus alert rules are stored under `observability/prometheus/rules/`.

Current rules cover:

- HTTP 5xx error rate above 5% for 5 minutes.
- HTTP p95 latency above 2 seconds for 5 minutes.
- PostgreSQL pool active connections above 90% of configured maximum for 5 minutes.

These rules are evaluated by Prometheus. Alert delivery through Alertmanager or another notification provider is intentionally deferred.
