use prometheus::{
    Encoder, HistogramOpts, HistogramVec, IntCounter, IntCounterVec, IntGauge, Opts, Registry,
    TextEncoder,
};
use sqlx::PgPool;
use std::time::Duration;

pub struct Metrics {
    registry: Registry,
    http_requests_total: IntCounterVec,
    http_request_duration_seconds: HistogramVec,
    ws_connections_active: IntGauge,
    ws_connections_total: IntCounter,
    ws_messages_received_total: IntCounterVec,
    ws_messages_sent_total: IntCounterVec,
    db_pool_connections_active: IntGauge,
    db_pool_connections_idle: IntGauge,
    db_pool_connections_max: IntGauge,
}

impl Metrics {
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Registry::new();

        let http_requests_total = IntCounterVec::new(
            Opts::new("http_requests_total", "Total HTTP requests"),
            &["method", "path", "status"],
        )?;
        let http_request_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "http_request_duration_seconds",
                "HTTP request duration in seconds",
            ),
            &["method", "path"],
        )?;
        let ws_connections_active = IntGauge::with_opts(Opts::new(
            "ws_connections_active",
            "Active WebSocket connections",
        ))?;
        let ws_connections_total = IntCounter::with_opts(Opts::new(
            "ws_connections_total",
            "Total WebSocket connections",
        ))?;
        let ws_messages_received_total = IntCounterVec::new(
            Opts::new(
                "ws_messages_received_total",
                "Total WebSocket text messages received",
            ),
            &["type"],
        )?;
        let ws_messages_sent_total = IntCounterVec::new(
            Opts::new(
                "ws_messages_sent_total",
                "Total WebSocket text messages sent",
            ),
            &["type"],
        )?;
        let db_pool_connections_active = IntGauge::with_opts(Opts::new(
            "db_pool_connections_active",
            "Active PostgreSQL pool connections",
        ))?;
        let db_pool_connections_idle = IntGauge::with_opts(Opts::new(
            "db_pool_connections_idle",
            "Idle PostgreSQL pool connections",
        ))?;
        let db_pool_connections_max = IntGauge::with_opts(Opts::new(
            "db_pool_connections_max",
            "Maximum PostgreSQL pool connections",
        ))?;

        registry.register(Box::new(http_requests_total.clone()))?;
        registry.register(Box::new(http_request_duration_seconds.clone()))?;
        registry.register(Box::new(ws_connections_active.clone()))?;
        registry.register(Box::new(ws_connections_total.clone()))?;
        registry.register(Box::new(ws_messages_received_total.clone()))?;
        registry.register(Box::new(ws_messages_sent_total.clone()))?;
        registry.register(Box::new(db_pool_connections_active.clone()))?;
        registry.register(Box::new(db_pool_connections_idle.clone()))?;
        registry.register(Box::new(db_pool_connections_max.clone()))?;

        Ok(Self {
            registry,
            http_requests_total,
            http_request_duration_seconds,
            ws_connections_active,
            ws_connections_total,
            ws_messages_received_total,
            ws_messages_sent_total,
            db_pool_connections_active,
            db_pool_connections_idle,
            db_pool_connections_max,
        })
    }

    pub fn observe_http(&self, method: &str, path: &str, status: u16, duration: Duration) {
        let path = normalize_path(path);
        let status = status.to_string();
        self.http_requests_total
            .with_label_values(&[method, path, &status])
            .inc();
        self.http_request_duration_seconds
            .with_label_values(&[method, path])
            .observe(duration.as_secs_f64());
    }

    pub fn ws_connected(&self) {
        self.ws_connections_total.inc();
        self.ws_connections_active.inc();
    }

    pub fn ws_disconnected(&self) {
        self.ws_connections_active.dec();
    }

    pub fn ws_message_received(&self, event_type: &str) {
        let label = bounded_ws_event_label(event_type);
        self.ws_messages_received_total
            .with_label_values(&[label])
            .inc();
    }

    pub fn ws_message_sent(&self, event_type: &str) {
        let label = bounded_ws_event_label(event_type);
        self.ws_messages_sent_total
            .with_label_values(&[label])
            .inc();
    }

    pub fn render(&self, pool: &PgPool) -> Result<String, prometheus::Error> {
        self.db_pool_connections_active
            .set((pool.size() as i64 - pool.num_idle() as i64).max(0));
        self.db_pool_connections_idle.set(pool.num_idle() as i64);
        self.db_pool_connections_max
            .set(pool.options().get_max_connections() as i64);

        let families = self.registry.gather();
        let mut buffer = Vec::new();
        TextEncoder::new().encode(&families, &mut buffer)?;
        Ok(String::from_utf8_lossy(&buffer).into_owned())
    }
}

fn normalize_path(path: &str) -> &str {
    match path {
        "/health" | "/ready" | "/metrics" | "/ws/connect" => path,
        "/api/v1/auth/register"
        | "/api/v1/auth/verify-email"
        | "/api/v1/auth/login"
        | "/api/v1/auth/refresh"
        | "/api/v1/auth/logout"
        | "/api/v1/auth/logout-all"
        | "/api/v1/auth/forgot-password"
        | "/api/v1/auth/reset-password"
        | "/api/v1/users/me"
        | "/api/v1/users/search"
        | "/api/v1/devices"
        | "/api/v1/sessions"
        | "/api/v1/contacts"
        | "/api/v1/blocks"
        | "/api/v1/conversations"
        | "/api/v1/messages/forward"
        | "/api/v1/media/upload"
        | "/api/v1/calls/history"
        | "/api/v1/calls/ice-config"
        | "/api/v1/groups" => path,
        _ if path.starts_with("/api/v1/") => {
            if path
                .split('/')
                .any(|segment| uuid::Uuid::parse_str(segment).is_ok())
            {
                "/api/v1/{id}"
            } else {
                "/api/v1/{route}"
            }
        }
        _ => "/other",
    }
}

fn bounded_ws_event_label(event_type: &str) -> &str {
    match event_type {
        "auth.ping" | "message.send" | "message.edit" | "message.delete" | "message.react"
        | "message.delivered" | "message.read" | "typing.start" | "typing.stop" | "call.offer"
        | "call.answer" | "call.connected" | "call.ice_candidate" | "call.reject" | "call.end" => {
            event_type
        }
        _ => "other",
    }
}

#[cfg(test)]
mod tests {
    use super::{bounded_ws_event_label, normalize_path};

    #[test]
    fn normalizes_dynamic_uuid_paths() {
        assert_eq!(
            normalize_path("/api/v1/users/550e8400-e29b-41d4-a716-446655440000"),
            "/api/v1/{id}"
        );
    }

    #[test]
    fn collapses_unknown_api_paths() {
        assert_eq!(
            normalize_path("/api/v1/attacker-controlled"),
            "/api/v1/{route}"
        );
        assert_eq!(normalize_path("/not-an-api-route"), "/other");
    }

    #[test]
    fn bounds_unknown_websocket_event_labels() {
        assert_eq!(bounded_ws_event_label("message.send"), "message.send");
        assert_eq!(bounded_ws_event_label("attacker-controlled"), "other");
    }
}
