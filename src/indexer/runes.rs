use anyhow::Result;
use bitcoin::{Transaction, TxOut};
use tracing::debug;

use crate::db::models::{RuneOperation, RunesData};

pub struct RunesProcessor;

impl RunesProcessor {
    pub fn new() -> Self {
        Self
    }

    pub fn process_transaction(&self, tx: &Transaction) -> Result<Option<RunesData>> {
        // Check for Runes protocol in OP_RETURN outputs
        for output in &tx.output {
            if let Some(runes_data) = self.parse_op_return(&output)? {
                return Ok(Some(runes_data));
            }
        }

        // Check for Runes in witness data (for taproot transactions)
        if tx.input.iter().any(|input| !input.witness.is_empty()) {
            if let Some(runes_data) = self.parse_witness_data(tx)? {
                return Ok(Some(runes_data));
            }
        }

        Ok(None)
    }

    fn parse_op_return(&self, output: &TxOut) -> Result<Option<RunesData>> {
        let script = &output.script_pubkey;

        // Check if this is an OP_RETURN output
        if !script.is_op_return() {
            return Ok(None);
        }

        // Get the data after OP_RETURN
        let data = script.as_bytes();
        if data.len() < 2 {
            return Ok(None);
        }

        // Skip OP_RETURN byte and length byte
        let payload = &data[2..];

        // Check for Runes magic bytes (example: "RUNE")
        if payload.len() >= 4 && &payload[0..4] == b"RUNE" {
            return self.parse_runes_payload(&payload[4..]);
        }

        Ok(None)
    }

    fn parse_witness_data(&self, tx: &Transaction) -> Result<Option<RunesData>> {
        // Parse Runes data from taproot witness
        // This is simplified - actual implementation would parse the witness stack

        for input in &tx.input {
            if input.witness.is_empty() {
                continue;
            }

            // Look for Runes data in witness
            for witness_element in input.witness.iter() {
                if witness_element.len() >= 4 && &witness_element[0..4] == b"RUNE" {
                    return self.parse_runes_payload(&witness_element[4..]);
                }
            }
        }

        Ok(None)
    }

    fn parse_runes_payload(&self, payload: &[u8]) -> Result<Option<RunesData>> {
        // This is a simplified parser - actual Runes protocol is more complex

        if payload.is_empty() {
            return Ok(None);
        }

        // Parse operation type (first byte)
        let operation = match payload[0] {
            0x00 => RuneOperation::Etch,
            0x01 => RuneOperation::Mint,
            0x02 => RuneOperation::Transfer,
            0x03 => RuneOperation::Burn,
            _ => return Ok(None),
        };

        // Parse remaining data based on operation
        let runes_data = match operation {
            RuneOperation::Etch => {
                // Parse new rune creation
                RunesData {
                    rune_id: Some(format!("RUNE_{}", hex::encode(&payload[1..9]))),
                    operation,
                    amount: None,
                    from_address: None,
                    to_address: None,
                    metadata: Some(serde_json::json!({
                        "symbol": "RUNE",
                        "decimals": 8,
                    })),
                }
            }
            RuneOperation::Mint => {
                // Parse minting operation
                let amount = if payload.len() >= 9 {
                    Some(u64::from_le_bytes(payload[1..9].try_into().unwrap_or_default()) as u128)
                } else {
                    None
                };

                RunesData {
                    rune_id: Some(format!("RUNE_{}", hex::encode(&payload[9..17]))),
                    operation,
                    amount,
                    from_address: None,
                    to_address: None,
                    metadata: None,
                }
            }
            RuneOperation::Transfer | RuneOperation::Burn => {
                // Parse transfer/burn operation
                let amount = if payload.len() >= 9 {
                    Some(u64::from_le_bytes(payload[1..9].try_into().unwrap_or_default()) as u128)
                } else {
                    None
                };

                RunesData {
                    rune_id: Some(format!("RUNE_{}", hex::encode(&payload[9..17]))),
                    operation,
                    amount,
                    from_address: None, // Would extract from inputs
                    to_address: None,   // Would extract from outputs
                    metadata: None,
                }
            }
        };

        debug!("Parsed Runes operation: {:?}", runes_data.operation);
        Ok(Some(runes_data))
    }
}
