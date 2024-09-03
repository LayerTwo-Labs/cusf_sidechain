use bitcoin::block;
use heed::{types::*, Env};
use heed::{Database, RwTxn};
use miette::{IntoDiagnostic, Result};

use cusf_sidechain_types::{Header, Transaction};

#[derive(Clone)]
pub struct Archive {
    /// Transaction sequence number -> Transaction
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
        header: Header,
        transactions: &[Transaction],
    ) -> Result<()> {
        let last_transaction = self.transactions.last(txn).into_diagnostic()?;
        let mut transaction_number = match last_transaction {
            Some((transaction_number, _transaction)) => transaction_number + 1,
            None => 0,
        };
        let transaction_range_start = transaction_number;
        for transaction in transactions {
            self.transactions
                .put(txn, &transaction_number, transaction)
                .into_diagnostic()?;
            transaction_number += 1;
        }
        let transaction_range_end = transaction_number;
        let last_header = self.headers.last(txn).into_diagnostic()?;
        let block_number = match last_header {
            Some((block_number, _header)) => block_number + 1,
            None => 0,
        };
        self.headers
            .put(
                txn,
                &block_number,
                &(header, (transaction_range_start, transaction_range_end)),
            )
            .into_diagnostic()?;
        Ok(())
    }

    // Disconnect number latest blocks.
    pub fn disconnect(&self, txn: &mut RwTxn, number: u32) -> Result<()> {
        todo!();
    }
}
