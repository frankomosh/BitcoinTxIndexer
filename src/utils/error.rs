use thiserror::Error;

#[derive(Error, Debug)]
pub enum IndexerError {
    #[error("Bitcoin RPC error: {0}")]
    BitcoinRpc(#[from] bitcoincore_rpc::Error),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid rune data: {0}")]
    InvalidRuneData(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("Generic error: {0}")]
    Generic(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, IndexerError>;

impl axum::response::IntoResponse for IndexerError {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;

        let (status, message) = match self {
            IndexerError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            ),
            IndexerError::Api(msg) => (StatusCode::BAD_REQUEST, msg),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };

        (status, message).into_response()
    }
}
