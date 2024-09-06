use heed::{types::*, Env, RoTxn};
use heed::{Database, RwTxn};
use miette::{miette, IntoDiagnostic, Result};

use cusf_sidechain_types::{Hashable, Header, Output, Transaction, HASH_LENGTH};

#[derive(Clone)]
pub struct Archive {
    /// Block number -> (Header, (Transactions range))
    pub headers: Database<SerdeBincode<u32>, SerdeBincode<(Header, (u64, u64))>>,
    /// Block number -> Coinbase
    pub coinbases: Database<SerdeBincode<u32>, SerdeBincode<Vec<Output>>>,
    /// Transaction sequence number -> Transaction
    pub transactions: Database<SerdeBincode<u64>, SerdeBincode<Transaction>>,
    pub bmm_hashes: Database<SerdeBincode<[u8; HASH_LENGTH]>, Unit>,
}

impl Archive {
    pub const NUM_DBS: u32 = 4;

    pub fn new(env: &Env) -> Result<Self> {
        let transactions = env
            .create_database(Some("archive_transactions"))
            .into_diagnostic()?;
        let coinbases = env.create_database(Some("coinbase")).into_diagnostic()?;
        let headers = env
            .create_database(Some("archive_headers"))
            .into_diagnostic()?;
        let bmm_hashes = env.create_database(Some("bmm_hashes")).into_diagnostic()?;
        Ok(Self {
            headers,
            coinbases,
            transactions,
            bmm_hashes,
        })
    }

    pub fn validate_header(&self, txn: &RoTxn, header: &Header) -> Result<()> {
        let block_hash = header.hash();
        self.bmm_hashes
            .get(txn, &block_hash)
            .into_diagnostic()?
            .ok_or(miette!("block header wasn't blind merge mined"))?;
        let prev_block_hash = match self.get_chain_tip(txn)? {
            Some((_block_number, (header, (_, _)))) => header.hash(),
            None => [0; HASH_LENGTH],
        };
        if header.prev_side_block_hash != prev_block_hash {
            return Err(miette!("wrong prev_side_block_hash"));
        }
        Ok(())
    }

    pub fn add_bmm_hashes(&self, txn: &mut RwTxn, bmm_hashes: &[[u8; HASH_LENGTH]]) -> Result<()> {
        for bmm_hash in bmm_hashes {
            self.bmm_hashes.put(txn, bmm_hash, &()).into_diagnostic()?;
        }
        Ok(())
    }

    pub fn get_chain_tip(&self, txn: &RoTxn) -> Result<Option<(u32, (Header, (u64, u64)))>> {
        Ok(self.headers.last(txn).into_diagnostic()?)
    }

    pub fn get_coinbase(&self, txn: &RoTxn, block_number: u32) -> Result<Option<Vec<Output>>> {
        Ok(self.coinbases.get(txn, &block_number).into_diagnostic()?)
    }

    pub fn connect(
        &self,
        txn: &mut RwTxn,
        header: Header,
        coinbase: &[Output],
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
            // 0th block is Genesis.
            None => 0 + 1,
        };
        self.coinbases
            .put(txn, &block_number, &coinbase.to_vec())
            .into_diagnostic()?;
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
