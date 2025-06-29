use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::api::ApiState;

#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
    version: String,
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

pub async fn get_block(
    Path(height): Path<u64>,
    Extension(state): Extension<ApiState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.db.get_block_by_height(height).await {
        Ok(Some(block)) => Ok(Json(serde_json::to_value(block).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Database error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_transaction(
    Path(txid): Path<String>,
    Extension(state): Extension<ApiState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.db.get_transaction(&txid).await {
        Ok(Some(tx)) => Ok(Json(serde_json::to_value(tx).unwrap())),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Database error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}

fn default_limit() -> i64 {
    50
}

pub async fn get_runes_transactions(
    Query(params): Query<PaginationParams>,
    Extension(state): Extension<ApiState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state
        .db
        .get_runes_transactions(params.limit, params.offset)
        .await
    {
        Ok(txs) => Ok(Json(serde_json::json!({
            "transactions": txs,
            "pagination": {
                "limit": params.limit,
                "offset": params.offset,
                "count": txs.len(),
            }
        }))),
        Err(e) => {
            error!("Database error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Serialize)]
pub struct Stats {
    pub last_indexed_block: Option<u64>,
    pub total_transactions: i64,
    pub total_runes_transactions: i64,
}

pub async fn get_stats(Extension(state): Extension<ApiState>) -> Result<Json<Stats>, StatusCode> {
    // In production, these would be actual queries
    let last_block = state
        .db
        .get_last_block_height()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(Stats {
        last_indexed_block: last_block,
        total_transactions: 0,       // Implement count query
        total_runes_transactions: 0, // Implement count query
    }))
}
