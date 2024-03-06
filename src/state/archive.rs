use heed::types::*;
use heed::{Database, RoTxn, RwTxn};
use miette::Result;

use crate::types::{Header, Transaction};

pub struct Archive {
    /// Transaction number -> Transaction
    pub transactions: Database<SerdeBincode<u64>, SerdeBincode<Transaction>>,
    /// Block number -> (Header, (Transactions range))
    pub headers: Database<SerdeBincode<u32>, SerdeBincode<(Header, (u64, u64))>>,
}

impl Archive {
    const NUM_DBS: usize = 2;

    pub fn connect(
        &self,
        txn: &mut RwTxn,
        header: &Header,
        transactions: &[Transaction],
    ) -> Result<()> {
        todo!();
    }

    // Disconnect number latest blocks.
    pub fn disconnect(&self, txn: &mut RwTxn, number: u32) -> Result<()> {
        todo!();
    }
}
