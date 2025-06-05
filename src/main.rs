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

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Load configuration
    let config = Config::from_env()?;
    info!("Configuration loaded");
    
    // Initialize database
    let db = Database::new(&config.database_url).await?;
    db.run_migrations().await?;
    info!("Database initialized");
    
    // Create shared state
    let db = Arc::new(db);
    let config = Arc::new(config);
    
    // Start the indexer
    let indexer = BitcoinIndexer::new(db.clone(), config.clone()).await?;
    let indexer_handle = tokio::spawn(async move {
        if let Err(e) = indexer.start().await {
            error!("Indexer error: {}", e);
        }
    });
    
    // Start the API server
    let api_state = ApiState {
        db: db.clone(),
        config: config.clone(),
    };
    
    let app = create_api_router(api_state);
    let addr = format!("{}:{}", config.api_host, config.api_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    info!("API server listening on {}", addr);
    
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    
    // Clean shutdown
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