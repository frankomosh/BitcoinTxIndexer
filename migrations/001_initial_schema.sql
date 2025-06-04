-- Enable TimescaleDB extension
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- Blocks table
CREATE TABLE blocks (
    height BIGINT PRIMARY KEY,
    hash VARCHAR(64) NOT NULL UNIQUE,
    prev_hash VARCHAR(64) NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    merkle_root VARCHAR(64) NOT NULL,
    indexed_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Transactions table
CREATE TABLE transactions (
    txid VARCHAR(64) PRIMARY KEY,
    block_height BIGINT NOT NULL REFERENCES blocks(height),
    block_hash VARCHAR(64) NOT NULL,
    version INT NOT NULL,
    locktime BIGINT NOT NULL,
    size INT NOT NULL,
    weight INT NOT NULL,
    fee BIGINT,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    indexed_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Outputs table (for UTXO tracking)
CREATE TABLE outputs (
    txid VARCHAR(64) NOT NULL,
    vout INT NOT NULL,
    value BIGINT NOT NULL,
    script_pubkey TEXT NOT NULL,
    address VARCHAR(100),
    spent BOOLEAN DEFAULT FALSE,
    spending_txid VARCHAR(64),
    spending_vin INT,
    PRIMARY KEY (txid, vout),
    FOREIGN KEY (txid) REFERENCES transactions(txid)
);

-- Runes transactions table
CREATE TABLE runes_transactions (
    id SERIAL PRIMARY KEY,
    txid VARCHAR(64) NOT NULL REFERENCES transactions(txid),
    block_height BIGINT NOT NULL,
    rune_id VARCHAR(64),
    operation VARCHAR(20) NOT NULL, -- 'mint', 'transfer', 'burn'
    amount NUMERIC,
    from_address VARCHAR(100),
    to_address VARCHAR(100),
    metadata JSONB,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    indexed_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Bridge events table (for future Starknet integration)
CREATE TABLE bridge_events (
    id SERIAL PRIMARY KEY,
    event_type VARCHAR(50) NOT NULL, -- 'deposit', 'withdrawal'
    txid VARCHAR(64),
    starknet_tx_hash VARCHAR(64),
    bitcoin_address VARCHAR(100),
    starknet_address VARCHAR(66),
    amount BIGINT,
    asset VARCHAR(64),
    status VARCHAR(20) DEFAULT 'pending',
    metadata JSONB,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    indexed_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_blocks_timestamp ON blocks(timestamp DESC);
CREATE INDEX idx_transactions_block_height ON transactions(block_height);
CREATE INDEX idx_outputs_address ON outputs(address);
CREATE INDEX idx_outputs_spent ON outputs(spent);
CREATE INDEX idx_runes_transactions_rune_id ON runes_transactions(rune_id);
CREATE INDEX idx_runes_transactions_timestamp ON runes_transactions(timestamp DESC);
CREATE INDEX idx_bridge_events_status ON bridge_events(status);

-- Convert tables to hypertables for time-series optimization
SELECT create_hypertable('blocks', 'timestamp', chunk_time_interval => INTERVAL '1 day', if_not_exists => TRUE);
SELECT create_hypertable('transactions', 'timestamp', chunk_time_interval => INTERVAL '1 day', if_not_exists => TRUE);
SELECT create_hypertable('runes_transactions', 'timestamp', chunk_time_interval => INTERVAL '1 day', if_not_exists => TRUE);
SELECT create_hypertable('bridge_events', 'timestamp', chunk_time_interval => INTERVAL '1 day', if_not_exists => TRUE);