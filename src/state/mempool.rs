use heed::{types::*, Env};
use heed::{Database, RwTxn};
use miette::{miette, IntoDiagnostic, Result};
use cusf_sidechain_types::{Hashable, Transaction, HASH_LENGTH};
use std::time::SystemTime;

#[derive(Clone)]
pub struct Mempool {
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
        let hash_to_transaction_fee_timestamp = env
            .create_database(Some("mempool_hash_to_transaction_fee_timestamps"))
            .into_diagnostic()?;
        let fee_to_hashes_sizes_timestamps = env
            .create_database(Some("mempool_fee_to_hashes_sizes_timestamps"))
            .into_diagnostic()?;
        Ok(Self {
            hash_to_transaction_fee_timestamp,
            fee_to_hashes_sizes_timestamps,
        })
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

    pub fn remove(&self, txn: &mut RwTxn, transaction_hash: &[u8; HASH_LENGTH]) -> Result<()> {
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
