# YBM Connect — Metrics

Phase 16 exposes Prometheus-compatible metrics from the Rust backend at `GET /metrics`.

## Implemented

- `http_requests_total{method,path,status}`
- `http_request_duration_seconds{method,path}`
- `ws_connections_active`
- `ws_connections_total`
- `ws_messages_received_total{type}`
- `ws_messages_sent_total{type}`
- `db_pool_connections_active`
- `db_pool_connections_idle`
- `db_pool_connections_max`

Metric labels are deliberately bounded. Query strings, user IDs, tokens, message bodies, and arbitrary WebSocket event names are not exported as labels.

## Collection

Prometheus is configured in `observability/prometheus/prometheus.yml` and can be started with:

`docker compose --profile observability up -d prometheus grafana`

The optional Grafana dashboard is provisioned from `observability/grafana/dashboards/`.
