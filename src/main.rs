use anyhow::Result;
use starknet_btc_indexer::{
    api::{create_api_router, ApiState},
    config::Config,
    db::Database,
    indexer::BitcoinIndexer,
};
use std::sync::Arc;
use tokio::signal;
use tracing::{info, error};

// Use Server from Hyper 0.14 as it is compatible with Axum 0.6
use hyper::Server;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let config = Config::from_env()?;
    info!("Configuration loaded");

    let db = Database::new(&config.database_url).await?;
    db.run_migrations().await?;
    info!("Database initialized");

    let db = Arc::new(db);
    let config = Arc::new(config);

    let indexer = BitcoinIndexer::new(db.clone(), config.clone()).await?;
    let indexer_handle = tokio::spawn(async move {
        if let Err(e) = indexer.start().await {
            error!("Indexer error: {}", e);
        }
    });

    let api_state = ApiState {
        db: db.clone(),
        config: config.clone(),
    };

    let app = create_api_router(api_state);
    let addr = format!("{}:{}", config.api_host, config.api_port).parse()?;

    info!("API server listening on {}", addr);

    // Works with Axum 0.6 + Hyper 0.14
    Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    indexer_handle.abort();
    info!("Shutting down gracefully");

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
