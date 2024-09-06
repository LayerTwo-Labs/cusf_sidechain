use cusf_sidechain_types::{Hashable, Header, Transaction, BLOCK_SIZE_LIMIT, HASH_LENGTH};
use heed::{types::*, Env, RoTxn};
use heed::{Database, RwTxn};
use miette::{miette, IntoDiagnostic, Result};
use rayon::prelude::IntoParallelRefMutIterator;
use std::collections::HashSet;
use std::time::SystemTime;

use super::utxos::UnitKey;

#[derive(Clone)]
pub struct Mempool {
    pending_transactions: Database<SerdeBincode<[u8; HASH_LENGTH]>, Unit>,
    // Transaction hash -> (transaction, fee, unix timestamp)
    hash_to_transaction_fee_timestamp:
        Database<SerdeBincode<[u8; HASH_LENGTH]>, SerdeBincode<(Transaction, u64, u64)>>,
    // Fee -> (hash, size, unix timestampe time)
    fee_to_hashes_sizes_timestamps:
        Database<SerdeBincode<u64>, SerdeBincode<Vec<([u8; HASH_LENGTH], u32, u64)>>>,
}

impl Mempool {
    pub const NUM_DBS: u32 = 2;

    pub fn new(env: &Env) -> Result<Self> {
        let pending_transactions = env
            .create_database(Some("pending_transactions"))
            .into_diagnostic()?;
        let hash_to_transaction_fee_timestamp = env
            .create_database(Some("mempool_hash_to_transaction_fee_timestamps"))
            .into_diagnostic()?;
        let fee_to_hashes_sizes_timestamps = env
            .create_database(Some("mempool_fee_to_hashes_sizes_timestamps"))
            .into_diagnostic()?;
        Ok(Self {
            pending_transactions,
            hash_to_transaction_fee_timestamp,
            fee_to_hashes_sizes_timestamps,
        })
    }

    pub fn connect(&self, txn: &mut RwTxn, transactions: &[Transaction]) -> Result<()> {
        self.pending_transactions.clear(txn).into_diagnostic()?;
        for transaction in transactions {
            let transaction_hash = transaction.hash();
            self.remove(txn, &transaction_hash)?;
        }
        Ok(())
    }

    pub fn get_pending_transactions(&self, txn: &RoTxn) -> Result<Vec<Transaction>> {
        let mut transactions = vec![];
        for item in self.pending_transactions.iter(txn).into_diagnostic()? {
            let (transaction_hash, ()) = item.into_diagnostic()?;
            let (transaction, _fee, _timestamp) = self
                .hash_to_transaction_fee_timestamp
                .get(txn, &transaction_hash)
                .into_diagnostic()?
                .ok_or(miette!("transaction not in mempool"))?;
            transactions.push(transaction);
        }
        Ok(transactions)
    }

    pub fn collect_transactions(&self, txn: &mut RwTxn) -> Result<()> {
        let mut spent_utxos = HashSet::new();
        let mut transactions = vec![];
        for item in self
            .fee_to_hashes_sizes_timestamps
            .rev_iter(txn)
            .into_diagnostic()?
        {
            let (_fee, hashes_sizes_timestamps) = item.into_diagnostic()?;
            let transactions_bytes = bincode::serialize(&transactions).into_diagnostic()?;
            let block_size = transactions_bytes.len();
            'outer: for (hash, size, _timestamp) in hashes_sizes_timestamps {
                if block_size + size as usize > BLOCK_SIZE_LIMIT {
                    break;
                }
                let (transaction, fee, timestamp) = self
                    .hash_to_transaction_fee_timestamp
                    .get(txn, &hash)
                    .into_diagnostic()?
                    .ok_or(miette!("transaction doesn't exist"))?;
                for input in &transaction.inputs {
                    if spent_utxos.contains(input) {
                        // If we see a transaction that spends the same utxo as an already included
                        // transaction, we always keep the already included transaction, because it
                        // is more desirable because it has the higher fee, smaller size, or is
                        // older.
                        //
                        // The transactions in the mempool are sorted by fee, size, and timestamp,
                        // and we are iterating in order, that is why the previously included
                        // transaction is always more desirable.
                        continue 'outer;
                    }
                }
                for input in &transaction.inputs {
                    spent_utxos.insert(input.clone());
                }
                transactions.push(transaction);
            }
        }
        let pending_transactions_hashes = transactions.iter().map(|t| t.hash());
        for pending_transaction_hash in pending_transactions_hashes {
            self.pending_transactions
                .put(txn, &pending_transaction_hash, &())
                .into_diagnostic()?;
        }
        Ok(())
    }

    pub fn submit_transaction(
        &self,
        txn: &mut RwTxn,
        transaction: &Transaction,
        fee: u64,
    ) -> Result<()> {
        let transaction_bytes = bincode::serialize(&transaction).into_diagnostic()?;
        let transaction_size = transaction_bytes.len();
        dbg!(&fee);
        dbg!(&transaction_size);
        let transaction_hash = transaction.hash();
        if self
            .hash_to_transaction_fee_timestamp
            .get(txn, &transaction_hash)
            .into_diagnostic()?
            .is_some()
        {
            // If the transaction is already in the mempool, don't do anything.
            return Ok(());
        }
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .into_diagnostic()?
            .as_secs();
        self.hash_to_transaction_fee_timestamp
            .put(
                txn,
                &transaction_hash,
                &(transaction.clone(), fee, timestamp),
            )
            .into_diagnostic()?;
        let mut hashes_sizes_timestamps = self
            .fee_to_hashes_sizes_timestamps
            .get(txn, &fee)
            .into_diagnostic()?
            .unwrap_or(vec![]);
        hashes_sizes_timestamps.push((transaction_hash, transaction_size as u32, timestamp));
        // Sort the vector in ascending order by transaction size in bytes and by timestamp.
        //
        // Smallest and oldest transactions would come first.
        hashes_sizes_timestamps.sort_unstable_by_key(|(_, size, timestamp)| (*size, *timestamp));
        self.fee_to_hashes_sizes_timestamps
            .put(txn, &fee, &hashes_sizes_timestamps)
            .into_diagnostic()?;
        Ok(())
    }

    fn remove(&self, txn: &mut RwTxn, transaction_hash: &[u8; HASH_LENGTH]) -> Result<()> {
        let (_, fee, _) = self
            .hash_to_transaction_fee_timestamp
            .get(txn, transaction_hash)
            .into_diagnostic()?
            .ok_or(miette!("mempool transaction doesn't exist"))?;
        self.hash_to_transaction_fee_timestamp
            .delete(txn, transaction_hash)
            .into_diagnostic()?;
        let hashes_sizes_timestamps = self
            .fee_to_hashes_sizes_timestamps
            .get(txn, &fee)
            .into_diagnostic()?
            .ok_or(miette!("mempool transaction index doesn't exist"))?;
        let hashes_sizes_timestamps: Vec<_> = hashes_sizes_timestamps
            .into_iter()
            .filter(|(hash, _, _)| hash != transaction_hash)
            .collect();
        self.fee_to_hashes_sizes_timestamps
            .put(txn, &fee, &hashes_sizes_timestamps)
            .into_diagnostic()?;
        Ok(())
    }
}
