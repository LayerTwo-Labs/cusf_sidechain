use std::collections::HashSet;

use cusf_sidechain_types::{OutPoint, Output, Transaction, ADDRESS_LENGTH, HASH_LENGTH};
use heed::{types::*, Env};
use heed::{Database, RoTxn, RwTxn};
use miette::{miette, IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};

/// Unit key. LMDB can't use zero-sized keys, so this encodes to a single byte
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub struct UnitKey;

impl<'de> Deserialize<'de> for UnitKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Deserialize any byte (ignoring it) and return UnitKey
        let _ = u8::deserialize(deserializer)?;
        Ok(UnitKey)
    }
}

impl Serialize for UnitKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Always serialize to the same arbitrary byte
        serializer.serialize_u8(0x00)
    }
}

#[derive(Clone)]
pub struct Utxos {
    utxos: Database<SerdeBincode<OutPoint>, SerdeBincode<Output>>,
    transaction_number: Database<SerdeBincode<UnitKey>, SerdeBincode<u64>>,
    main_block_height: Database<SerdeBincode<UnitKey>, SerdeBincode<u32>>,
    main_chain_tip: Database<SerdeBincode<UnitKey>, SerdeBincode<[u8; HASH_LENGTH]>>,
    side_block_height: Database<SerdeBincode<UnitKey>, SerdeBincode<u32>>,
    refundable_withdrawals: Database<SerdeBincode<OutPoint>, Unit>,
    locked_withdrawals: Database<SerdeBincode<OutPoint>, Unit>,
}

impl Utxos {
    pub const NUM_DBS: u32 = 7;

    pub fn new(env: &Env) -> Result<Self> {
        let utxos = env.create_database(Some("utxos")).into_diagnostic()?;
        let transaction_number = env
            .create_database(Some("transaction_number"))
            .into_diagnostic()?;
        let main_block_height = env
            .create_database(Some("main_block_height"))
            .into_diagnostic()?;
        let main_chain_tip = env
            .create_database(Some("main_chain_tip"))
            .into_diagnostic()?;
        let side_block_height = env
            .create_database(Some("side_block_height"))
            .into_diagnostic()?;
        let refundable_withdrawals = env
            .create_database(Some("utxos_refundable_withdrawals"))
            .into_diagnostic()?;
        let locked_withdrawals = env
            .create_database(Some("utxos_locked_withdrawals"))
            .into_diagnostic()?;
        Ok(Self {
            utxos,
            transaction_number,
            main_block_height,
            main_chain_tip,
            side_block_height,
            refundable_withdrawals,
            locked_withdrawals,
        })
    }

    pub fn get_main_block_height(&self, txn: &RoTxn) -> Result<u32> {
        let height = self
            .main_block_height
            .get(txn, &UnitKey)
            .into_diagnostic()?
            .unwrap_or(0);
        Ok(height)
    }

    pub fn set_main_block_height(&self, txn: &mut RwTxn, height: u32) -> Result<()> {
        self.main_block_height
            .put(txn, &UnitKey, &height)
            .into_diagnostic()?;
        Ok(())
    }

    pub fn get_main_chain_tip(&self, txn: &RoTxn) -> Result<[u8; HASH_LENGTH]> {
        let height = self
            .main_chain_tip
            .get(txn, &UnitKey)
            .into_diagnostic()?
            .unwrap_or([0; HASH_LENGTH]);
        Ok(height)
    }

    pub fn set_main_chain_tip(
        &self,
        txn: &mut RwTxn,
        block_hash: &[u8; HASH_LENGTH],
    ) -> Result<()> {
        self.main_chain_tip
            .put(txn, &UnitKey, &block_hash)
            .into_diagnostic()?;
        Ok(())
    }

    pub fn get_side_block_height(&self, txn: &RoTxn) -> Result<u32> {
        let height = self
            .side_block_height
            .get(txn, &UnitKey)
            .into_diagnostic()?
            .unwrap_or(0);
        Ok(height)
    }

    pub fn is_empty(&self, txn: &RoTxn) -> Result<bool> {
        self.utxos.is_empty(txn).into_diagnostic()
    }

    pub fn add_utxo(&self, txn: &mut RwTxn, outpoint: &OutPoint, output: &Output) -> Result<()> {
        self.utxos.put(txn, outpoint, output).into_diagnostic()?;
        Ok(())
    }

    pub fn remove_utxo(&self, txn: &mut RwTxn, outpoint: &OutPoint) -> Result<()> {
        self.utxos.delete(txn, outpoint).into_diagnostic()?;
        Ok(())
    }

    pub fn validate(&self, txn: &RoTxn, transactions: &[Transaction]) -> Result<bool> {
        let mut spent_utxos = HashSet::new();
        for transaction in transactions {
            let mut value_in = 0;
            for input in &transaction.inputs {
                if spent_utxos.contains(input) {
                    return Ok(false);
                }
                let spent_utxo = self.utxos.get(txn, input).into_diagnostic()?;
                let value = match spent_utxo {
                    Some(spent_utxo) => spent_utxo.total_value(),
                    None => {
                        return Ok(false);
                    }
                };
                value_in += value;
                spent_utxos.insert(input);
            }
            let value_out = transaction.value_out();
            if value_out > value_in {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Performs no validation, assumes that all transactions are valid.
    pub fn connect(&self, txn: &mut RwTxn, transactions: &[Transaction]) -> Result<()> {
        let transaction_number = self
            .transaction_number
            .get(txn, &UnitKey)
            .into_diagnostic()?;
        let mut transaction_number = match transaction_number {
            Some(transaction_number) => transaction_number + 1,
            None => 0,
        };
        for transaction in transactions {
            for input in &transaction.inputs {
                self.utxos.delete(txn, &input).into_diagnostic()?;
            }
            for (output_number, output) in transaction.outputs.iter().enumerate() {
                let outpoint = OutPoint::Regular {
                    transaction_number,
                    output_number: output_number as u8,
                };
                self.utxos.put(txn, &outpoint, &output).into_diagnostic()?;
                if matches!(output, Output::Withdrawal { .. }) {
                    self.refundable_withdrawals
                        .put(txn, &outpoint, &())
                        .into_diagnostic()?;
                }
            }
            transaction_number += 1;
        }
        let side_block_height = self
            .side_block_height
            .get(txn, &UnitKey)
            .into_diagnostic()?;
        let side_block_height = match side_block_height {
            Some(side_block_height) => side_block_height + 1,
            None => 0,
        };
        self.side_block_height
            .put(txn, &UnitKey, &side_block_height)
            .into_diagnostic()?;
        Ok(())
    }

    /// Performs no validation, assumes that all transactions are valid.
    pub fn disconnect(&self, txn: &mut RwTxn, transactions: &[Transaction]) -> Result<()> {
        todo!();
    }

    pub fn get_transaction_fee(&self, txn: &RoTxn, transaction: &Transaction) -> Result<u64> {
        let mut value_in = 0;
        for input in &transaction.inputs {
            let spent_utxo = self
                .utxos
                .get(&txn, &input)
                .into_diagnostic()?
                .ok_or(miette!("transaction is invalid"))?;
            value_in += spent_utxo.total_value();
        }
        let value_out = transaction.value_out();
        if value_in < value_out {
            return Err(miette!("transaction is invalid"));
        }
        Ok(value_in - value_out)
    }

    pub fn extract_input_addresses(
        &self,
        txn: &RoTxn,
        transactions: &[Transaction],
    ) -> Result<Vec<[u8; ADDRESS_LENGTH]>> {
        let mut addresses = vec![];
        for transaction in transactions {
            for input in &transaction.inputs {
                let output = self
                    .utxos
                    .get(txn, input)
                    .into_diagnostic()?
                    .ok_or(miette!("input {input} doesn't exist"))?;
                let address = output.address();
                addresses.push(address);
            }
        }
        Ok(addresses)
    }
}
