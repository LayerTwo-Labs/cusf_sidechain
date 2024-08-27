use heed::{types::*, Env};
use heed::{Database, RoTxn, RwTxn};
use miette::{IntoDiagnostic, Result};

use crate::types::{Hashable, Output, Transaction, HASH_LENGTH};

#[derive(Clone)]
pub struct Mempool {
    // Transaction hash -> (Transaction, Fee)
    transactions: Database<SerdeBincode<[u8; HASH_LENGTH]>, SerdeBincode<(Transaction, u64)>>,
    // Fee -> (hash, size)
    fee_to_hashes_sizes: Database<SerdeBincode<u64>, SerdeBincode<Vec<([u8; HASH_LENGTH], u32)>>>,
}

impl Mempool {
    pub const NUM_DBS: u32 = 2;

    pub fn new(env: &Env) -> Result<Self> {
        let transactions = env
            .create_database(Some("mempool_transactions"))
            .into_diagnostic()?;
        let fee_to_hashes_sizes = env
            .create_database(Some("mempool_fee_to_hashes_sizes"))
            .into_diagnostic()?;
        Ok(Self {
            transactions,
            fee_to_hashes_sizes,
        })
    }

    pub fn add(&self, txn: &mut RwTxn, transactions_fees: &[(Transaction, u64)]) -> Result<()> {
        for (transaction, fee) in transactions_fees {
            let hash = transaction.hash();
            let size = bincode::serialize(transaction).unwrap().len();
        }
        todo!();
    }

    pub fn remove(&self, txn: &mut RwTxn, transaction_hashes: &[[u8; HASH_LENGTH]]) -> Result<()> {
        for hash in transaction_hashes {}
        todo!();
    }
}
