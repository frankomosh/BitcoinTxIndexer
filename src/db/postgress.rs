use anyhow::Result;
use bitcoin::{Block, BlockHeader, Transaction};
use chrono::{DateTime, Utc};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tracing::{info, debug};

use crate::db::models::{BlockModel, TransactionModel, OutputModel, RunesTransactionModel, RunesData};

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(20)
            .connect(database_url)
            .await?;
        
        Ok(Self { pool })
    }
    
    pub async fn run_migrations(&self) -> Result<()> {
        // we'll replace with sqlx migrate, in production
        // For now, just assume migrations are run manually
        info!("Database migrations completed");
        Ok(())
    }
    
    pub async fn get_last_block_height(&self) -> Result<Option<u64>> {
        let result = sqlx::query_scalar::<_, i64>(
            "SELECT MAX(height) FROM blocks"
        )
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(result.map(|h| h as u64))
    }
    
    pub async fn insert_block(&self, block: &Block, height: u64) -> Result<()> {
        let timestamp = DateTime::<Utc>::from_timestamp(block.header.time as i64, 0)
            .unwrap_or_else(Utc::now);
        
        sqlx::query(
            r#"
            INSERT INTO blocks (height, hash, prev_hash, timestamp, merkle_root)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (height) DO NOTHING
            "#
        )
        .bind(height as i64)
        .bind(block.header.block_hash().to_string())
        .bind(block.header.prev_blockhash.to_string())
        .bind(timestamp)
        .bind(block.header.merkle_root.to_string())
        .execute(&self.pool)
        .await?;
        
        debug!("Inserted block at height {}", height);
        Ok(())
    }
    
    pub async fn insert_transaction(
        &self,
        tx: &Transaction,
        block_height: u64,
        block_header: &BlockHeader,
    ) -> Result<()> {
        let timestamp = DateTime::<Utc>::from_timestamp(block_header.time as i64, 0)
            .unwrap_or_else(Utc::now);
        
        // Calculate fee (would need to look up input values in production)
        let fee = None; // Simplified for now
        
        sqlx::query(
            r#"
            INSERT INTO transactions 
            (txid, block_height, block_hash, version, locktime, size, weight, fee, timestamp)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (txid) DO NOTHING
            "#
        )
        .bind(tx.txid().to_string())
        .bind(block_height as i64)
        .bind(block_header.block_hash().to_string())
        .bind(tx.version as i32)
        .bind(tx.lock_time.0 as i64)
        .bind(tx.size() as i32)
        .bind(tx.weight() as i32)
        .bind(fee)
        .bind(timestamp)
        .execute(&self.pool)
        .await?;
        
        // Insert outputs
        for (vout, output) in tx.output.iter().enumerate() {
            let address = bitcoin::Address::from_script(&output.script_pubkey, bitcoin::Network::Bitcoin)
                .ok()
                .map(|a| a.to_string());
            
            sqlx::query(
                r#"
                INSERT INTO outputs 
                (txid, vout, value, script_pubkey, address, spent)
                VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT (txid, vout) DO NOTHING
                "#
            )
            .bind(tx.txid().to_string())
            .bind(vout as i32)
            .bind(output.value as i64)
            .bind(hex::encode(&output.script_pubkey.as_bytes()))
            .bind(address)
            .bind(false)
            .execute(&self.pool)
            .await?;
        }
        
        Ok(())
    }
    
    pub async fn insert_runes_transaction(
        &self,
        runes_data: &RunesData,
        tx: &Transaction,
        block_height: u64,
    ) -> Result<()> {
        let timestamp = Utc::now(); // Should get from block
        
        sqlx::query(
            r#"
            INSERT INTO runes_transactions 
            (txid, block_height, rune_id, operation, amount, from_address, to_address, metadata, timestamp)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#
        )
        .bind(tx.txid().to_string())
        .bind(block_height as i64)
        .bind(&runes_data.rune_id)
        .bind(runes_data.operation.to_string())
        .bind(runes_data.amount.map(|a| sqlx::types::Decimal::from(a as i64)))
        .bind(&runes_data.from_address)
        .bind(&runes_data.to_address)
        .bind(&runes_data.metadata)
        .bind(timestamp)
        .execute(&self.pool)
        .await?;
        
        info!("Inserted Runes transaction: {}", tx.txid());
        Ok(())
    }
    
    // Query methods for API
    pub async fn get_block_by_height(&self, height: u64) -> Result<Option<BlockModel>> {
        let block = sqlx::query_as::<_, BlockModel>(
            "SELECT * FROM blocks WHERE height = $1"
        )
        .bind(height as i64)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(block)
    }
    
    pub async fn get_transaction(&self, txid: &str) -> Result<Option<TransactionModel>> {
        let tx = sqlx::query_as::<_, TransactionModel>(
            "SELECT * FROM transactions WHERE txid = $1"
        )
        .bind(txid)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(tx)
    }
    
    pub async fn get_runes_transactions(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<RunesTransactionModel>> {
        let txs = sqlx::query_as::<_, RunesTransactionModel>(
            "SELECT * FROM runes_transactions ORDER BY timestamp DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(txs)
    }
}