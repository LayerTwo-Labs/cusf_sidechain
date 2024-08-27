use heed::{types::*, Env};
use heed::{Database, RwTxn};
use miette::{IntoDiagnostic, Result};

use crate::types::{Header, Transaction};

#[derive(Clone)]
pub struct Archive {
    /// Transaction number -> Transaction
    pub transactions: Database<SerdeBincode<u64>, SerdeBincode<Transaction>>,
    /// Block number -> (Header, (Transactions range))
    pub headers: Database<SerdeBincode<u32>, SerdeBincode<(Header, (u64, u64))>>,
}

impl Archive {
    pub const NUM_DBS: u32 = 2;

    pub fn new(env: &Env) -> Result<Self> {
        let transactions = env
            .create_database(Some("archive_transactions"))
            .into_diagnostic()?;
        let headers = env
            .create_database(Some("archive_headers"))
            .into_diagnostic()?;
        Ok(Self {
            transactions,
            headers,
        })
    }

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
