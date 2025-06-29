pub mod graphql;
pub mod rest;

use axum::{routing::get, Extension, Router};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

use crate::{config::Config, db::Database};

#[derive(Clone)]
pub struct ApiState {
    pub db: Arc<Database>,
    pub config: Arc<Config>,
}

pub fn create_api_router(state: ApiState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // REST endpoints
        .route("/health", get(rest::health))
        .route("/blocks/:height", get(rest::get_block))
        .route("/transactions/:txid", get(rest::get_transaction))
        .route("/runes/transactions", get(rest::get_runes_transactions))
        .route("/stats", get(rest::get_stats))
        // GraphQL endpoint
        .route(
            "/graphql",
            get(graphql::graphql_playground).post(graphql::graphql_handler),
        )
        // Add state
        .layer(Extension(state))
        .layer(cors)
}
