use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BlockModel {
    pub height: i64,
    pub hash: String,
    pub prev_hash: String,
    pub timestamp: DateTime<Utc>,
    pub merkle_root: String,
    pub indexed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TransactionModel {
    pub txid: String,
    pub block_height: i64,
    pub block_hash: String,
    pub version: i32,
    pub locktime: i64,
    pub size: i32,
    pub weight: i32,
    pub fee: Option<i64>,
    pub timestamp: DateTime<Utc>,
    pub indexed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OutputModel {
    pub txid: String,
    pub vout: i32,
    pub value: i64,
    pub script_pubkey: String,
    pub address: Option<String>,
    pub spent: bool,
    pub spending_txid: Option<String>,
    pub spending_vin: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RunesTransactionModel {
    pub id: i32,
    pub txid: String,
    pub block_height: i64,
    pub rune_id: Option<String>,
    pub operation: String,
    pub amount: Option<sqlx::types::Decimal>,
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub timestamp: DateTime<Utc>,
    pub indexed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunesData {
    pub rune_id: Option<String>,
    pub operation: RuneOperation,
    pub amount: Option<u128>,
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuneOperation {
    Mint,
    Transfer,
    Burn,
    Etch,
}

impl ToString for RuneOperation {
    fn to_string(&self) -> String {
        match self {
            RuneOperation::Mint => "mint".to_string(),
            RuneOperation::Transfer => "transfer".to_string(),
            RuneOperation::Burn => "burn".to_string(),
            RuneOperation::Etch => "etch".to_string(),
        }
    }
}
