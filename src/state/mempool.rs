use heed::types::*;
use heed::{Database, RoTxn, RwTxn};
use miette::Result;

use crate::types::{Hashable, Output, Transaction, HASH_LENGTH};

pub struct Mempool {
    // Transaction hash -> (Transaction, Fee)
    transactions: Database<SerdeBincode<[u8; HASH_LENGTH]>, SerdeBincode<(Transaction, u64)>>,
    // Fee -> (hash, size)
    fee_to_hashes_sizes: Database<SerdeBincode<u64>, SerdeBincode<Vec<([u8; HASH_LENGTH], u32)>>>,
}

impl Mempool {
    const NUM_DBS: usize = 1;

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
