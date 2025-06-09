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

impl From<IndexerError> for axum::response::Response {
    fn from(err: IndexerError) -> Self {
        use axum::http::StatusCode;
        use axum::response::IntoResponse;
        
        let (status, message) = match err {
            IndexerError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
            IndexerError::Api(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };
        
        (status, message).into_response()
    }
}