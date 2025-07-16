# Starknet Bitcoin Indexer

A BtcTx indexer with support for Runes and Starknet bridge operations.

## Features

- ✅ Real-time Bitcoin block and transaction indexing
- ✅ Runes protocol transaction detection and parsing
- ✅ RESTful API for querying indexed data
- ✅ GraphQL endpoint for flexible queries
- ✅ TimescaleDB for efficient time-series data storage
- ✅ Ready for Starknet bridge event integration

## Quick Start

1. **Clone and setup:**
   ```bash
   git clone
   cd BitcoinTxIndexer
   cp .env.example .env
   ```
2. **Configure .env:**
- Set Bitcoin node RPC credentials
- Configure database connection
- Adjust indexer settings

3. **Start Services:**
```bash
docker-compose up -d
```

4. **Run Migrations:**
```bash
psql $DATABASE_URL < migrations/001_initial_schema.sql
```

5. **Start Indexer:**
```bash
cargo run --release
```
## API Endpoints

**REST API**
- GET /health - Health check
- GET /blocks/:height - Get block by height
- GET /transactions/:txid - Get transaction by ID
- GET /runes/transactions?limit=50&offset=0 - List Runes transactions
- GET /stats - Get indexer statistics

**GraphQL**
- Endpoint: POST /graphql
- Playground: GET /graphql
## Example Query

```bash
query {
  runesTransactions(limit: 10) {
    txid
    operation
    amount
    runeId
    timestamp
  }
}
```
## Architecture

- Bitcoin Indexer: Should connect to Bitcoin Core node and processes blocks
- Runes Processor: Should detect and parse Runes transactions
- Database: PostgreSQL with TimescaleDB for efficient time-series storage
- API Layer: REST and GraphQL endpoints for data access

## Future Enhancements

 - Starknet bridge event monitoring
 - Additional metaprotocol support (Ordinals, BRC-20)
 - Horizontal scaling with multiple indexer instances
 - Prometheus metrics integration

## Note

This is still experimental and should still not be used in production yet!

## License

**MIT**
