use heed::types::*;
use heed::{Database, RoTxn, RwTxn};
use miette::{miette, IntoDiagnostic, Result};

use crate::types::{OutPoint, Output, Transaction, ADDRESS_LENGTH};

pub struct Utxos {
    utxos: Database<SerdeBincode<OutPoint>, SerdeBincode<Output>>,
    refundable_withdrawals: Database<SerdeBincode<OutPoint>, Unit>,
    locked_withdrawals: Database<SerdeBincode<OutPoint>, Unit>,
}

impl Utxos {
    const NUM_DBS: usize = 3;

    pub fn validate(&self, txn: &RoTxn, transactions: &[Transaction]) -> Result<u64> {
        todo!();
    }

    /// Performs no validation, assumes that all transactions are valid.
    pub fn connect(&self, txn: &mut RwTxn, transactions: &[Transaction]) -> Result<()> {
        todo!();
    }

    /// Performs no validation, assumes that all transactions are valid.
    pub fn disconnect(&self, txn: &mut RwTxn, transactions: &[Transaction]) -> Result<()> {
        todo!();
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
