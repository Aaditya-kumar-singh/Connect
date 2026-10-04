pub mod connection;
pub mod handlers;
pub mod manager;
pub mod protocol;
pub mod pubsub;

pub const REGISTRY_TTL_SECONDS: u64 = 300;
pub const HEARTBEAT_TIMEOUT_SECONDS: u64 = 60;
