use anyhow::Result;
use bitcoin::{Block, Transaction};
use bitcoincore_rpc::{Auth, Client, RpcApi};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info};

use crate::{config::Config, db::Database, indexer::runes::RunesProcessor};

pub struct BitcoinIndexer {
    client: Client,
    db: Arc<Database>,
    config: Arc<Config>,
    runes_processor: RunesProcessor,
}

impl BitcoinIndexer {
    pub async fn new(db: Arc<Database>, config: Arc<Config>) -> Result<Self> {
        let auth = Auth::UserPass(
            config.bitcoin_rpc_user.clone(),
            config.bitcoin_rpc_pass.clone(),
        );

        let client = Client::new(&config.bitcoin_rpc_url, auth)?;
        let runes_processor = RunesProcessor::new();

        Ok(Self {
            client,
            db,
            config,
            runes_processor,
        })
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting Bitcoin indexer");

        // Get the last indexed height from database
        let mut current_height = self
            .db
            .get_last_block_height()
            .await?
            .unwrap_or(self.config.indexer_start_height);

        loop {
            // Get the latest block height from Bitcoin node
            let latest_height = self.client.get_block_count()? as u64;

            if current_height >= latest_height {
                debug!("Caught up to latest block {}", latest_height);
                sleep(Duration::from_secs(self.config.indexer_poll_interval_secs)).await;
                continue;
            }

            // Process blocks in batches
            let end_height =
                (current_height + self.config.indexer_batch_size as u64).min(latest_height);

            for height in current_height..=end_height {
                if let Err(e) = self.process_block(height).await {
                    error!("Error processing block {}: {}", height, e);
                    // Continue with next block
                }
            }

            current_height = end_height + 1;
            info!("Indexed up to block {}", end_height);
        }
    }

    async fn process_block(&self, height: u64) -> Result<()> {
        // Get block hash
        let block_hash = self.client.get_block_hash(height)?;

        // Get full block
        let block = self.client.get_block(&block_hash)?;

        // Store block in database
        self.db.insert_block(&block, height).await?;

        // Process transactions
        for tx in &block.txdata {
            self.process_transaction(tx, height, &block).await?;
        }

        Ok(())
    }

    async fn process_transaction(
        &self,
        tx: &Transaction,
        block_height: u64,
        block: &Block,
    ) -> Result<()> {
        // Store transaction
        self.db
            .insert_transaction(tx, block_height, &block.header)
            .await?;

        // Check if this is a Runes transaction
        if let Some(runes_data) = self.runes_processor.process_transaction(tx)? {
            self.db
                .insert_runes_transaction(&runes_data, tx, block_height)
                .await?;
        }

        Ok(())
    }
}
