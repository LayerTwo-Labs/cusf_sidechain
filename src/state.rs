use heed::types::*;
use heed::{Database, RoTxn, RwTxn};
use miette::Result;

use crate::types::{OutPoint, Output, Transaction};

struct State {
    utxos: Database<SerdeBincode<OutPoint>, SerdeBincode<Output>>,
}

impl State {
    const NUM_DBS: usize = 1;

    pub fn validate(&self, transactions: &[Transaction]) -> Result<u64> {
        todo!();
    }

    /// Performs no validation, assumes that all transactions are valid.
    pub fn connect(&self, transactions: &[Transaction]) -> Result<()> {
        todo!();
    }

    /// Performs no validation, assumes that all transactions are valid.
    pub fn disconnect(&self, transactions: &[Transaction]) -> Result<()> {
        todo!();
    }
}
