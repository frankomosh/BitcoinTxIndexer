use async_graphql::{
    http::GraphiQLSource, Context, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::Extension,
    response::{Html, IntoResponse},
};
use chrono::{DateTime, Utc};

use crate::{
    api::ApiState,
    db::models::{BlockModel, TransactionModel, RunesTransactionModel},
};

pub type AppSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn block(&self, ctx: &Context<'_>, height: i64) -> async_graphql::Result<Option<Block>> {
        let state = ctx.data::<ApiState>()?;
        let block = state.db.get_block_by_height(height as u64).await?;
        Ok(block.map(Block::from))
    }
    
    async fn transaction(&self, ctx: &Context<'_>, txid: String) -> async_graphql::Result<Option<Transaction>> {
        let state = ctx.data::<ApiState>()?;
        let tx = state.db.get_transaction(&txid).await?;
        Ok(tx.map(Transaction::from))
    }
    
    async fn runes_transactions(
        &self,
        ctx: &Context<'_>,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> async_graphql::Result<Vec<RunesTransaction>> {
        let state = ctx.data::<ApiState>()?;
        let limit = limit.unwrap_or(50) as i64;
        let offset = offset.unwrap_or(0) as i64;
        
        let txs = state.db.get_runes_transactions(limit, offset).await?;
        Ok(txs.into_iter().map(RunesTransaction::from).collect())
    }
    
    async fn stats(&self, ctx: &Context<'_>) -> async_graphql::Result<Stats> {
        let state = ctx.data::<ApiState>()?;
        let last_block = state.db.get_last_block_height().await?;
        
        Ok(Stats {
            last_indexed_block: last_block.map(|h| h as i64),
            total_transactions: 0,
            total_runes_transactions: 0,
        })
    }
}

#[derive(SimpleObject)]
struct Block {
    height: i64,
    hash: String,
    prev_hash: String,
    timestamp: DateTime<Utc>,
    merkle_root: String,
}

impl From<BlockModel> for Block {
    fn from(model: BlockModel) -> Self {
        Self {
            height: model.height,
            hash: model.hash,
            prev_hash: model.prev_hash,
            timestamp: model.timestamp,
            merkle_root: model.merkle_root,
        }
    }
}

#[derive(SimpleObject)]
struct Transaction {
    txid: String,
    block_height: i64,
    block_hash: String,
    version: i32,
    locktime: i64,
    size: i32,
    weight: i32,
    fee: Option<i64>,
    timestamp: DateTime<Utc>,
}

impl From<TransactionModel> for Transaction {
    fn from(model: TransactionModel) -> Self {
        Self {
            txid: model.txid,
            block_height: model.block_height,
            block_hash: model.block_hash,
            version: model.version,
            locktime: model.locktime,
            size: model.size,
            weight: model.weight,
            fee: model.fee,
            timestamp: model.timestamp,
        }
    }
}

#[derive(SimpleObject)]
struct RunesTransaction {
    id: i32,
    txid: String,
    block_height: i64,
    rune_id: Option<String>,
    operation: String,
    amount: Option<String>,
    from_address: Option<String>,
    to_address: Option<String>,
    metadata: Option<serde_json::Value>,
    timestamp: DateTime<Utc>,
}

impl From<RunesTransactionModel> for RunesTransaction {
    fn from(model: RunesTransactionModel) -> Self {
        Self {
            id: model.id,
            txid: model.txid,
            block_height: model.block_height,
            rune_id: model.rune_id,
            operation: model.operation,
            amount: model.amount.map(|a| a.to_string()),
            from_address: model.from_address,
            to_address: model.to_address,
            metadata: model.metadata,
            timestamp: model.timestamp,
        }
    }
}

#[derive(SimpleObject)]
struct Stats {
    last_indexed_block: Option<i64>,
    total_transactions: i64,
    total_runes_transactions: i64,
}

pub async fn graphql_handler(
    Extension(state): Extension<ApiState>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(state)
        .finish();
    
    schema.execute(req.into_inner()).await.into()
}

pub async fn graphql_playground() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}