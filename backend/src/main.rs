use dotenvy::dotenv;
use std::{future::IntoFuture, net::SocketAddr};
use tokio::sync::oneshot;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use ybm_connect::{
    app_state::AppState,
    config::Config,
    infrastructure::{database, redis_client, reliability_events},
    router,
    websocket::pubsub,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let config = Config::from_env()?;

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&config.rust_log))
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        host = %config.server_host,
        port = %config.server_port,
        "Initializing YBM Connect server"
    );

    let db_pool = database::create_pool(
        &config.database_url,
        config.database_max_connections,
        config.database_min_connections,
    )?;
    tracing::info!("PostgreSQL connection pool initialized");

    let redis = redis_client::create_client(&config.redis_url)?;
    tracing::info!("Redis client initialized");

    let state = AppState::new(config.clone(), db_pool, redis);
    let app = router::build(state.clone());

    let (pubsub_shutdown_tx, pubsub_shutdown_rx) = oneshot::channel::<()>();
    tokio::spawn(pubsub::run(state.clone(), pubsub_shutdown_rx));

    let addr: SocketAddr = config.server_addr().parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(addr = %addr, instance_id = %state.instance_id(), "YBM Connect server listening");
    reliability_events::publish(&state, "startup").await;

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let server = axum::serve(listener, app).with_graceful_shutdown(async move {
        let _ = shutdown_rx.await;
    });
    let server_handle = tokio::spawn(server.into_future());

    wait_for_shutdown_signal().await;
    reliability_events::publish(&state, "shutdown").await;
    let _ = pubsub_shutdown_tx.send(());
    let _ = shutdown_tx.send(());

    match tokio::time::timeout(std::time::Duration::from_secs(10), server_handle).await {
        Ok(join_result) => join_result??,
        Err(_) => {
            tracing::error!("Graceful shutdown exceeded 10 seconds; forcing termination");
        }
    }

    tracing::info!("Server shut down gracefully");
    Ok(())
}

async fn wait_for_shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C signal handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => tracing::info!("Received SIGINT (Ctrl+C), initiating graceful shutdown"),
        _ = terminate => tracing::info!("Received SIGTERM, initiating graceful shutdown"),
    }
}
