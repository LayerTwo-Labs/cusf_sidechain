mod archive;
mod mempool;
mod utxos;

use archive::Archive;
use bip300301_enforcer_proto::validator::Deposit;
use cusf_sidechain_types::{
    Hashable, Header, MainBlock, OutPoint, Output, Transaction, WithdrawalBundleEventType,
    HASH_LENGTH,
};
use heed::{Env, EnvOpenOptions, RwTxn};
use mempool::Mempool;
use miette::{miette, IntoDiagnostic, Result};
use std::path::Path;
use utxos::Utxos;

#[derive(Clone)]
pub struct State {
    env: Env,
    utxos: Utxos,
    archive: Archive,
    mempool: Mempool,
}

impl State {
    pub fn new(datadir: &Path) -> Result<Self> {
        let env = EnvOpenOptions::new()
            .max_dbs(Mempool::NUM_DBS + Archive::NUM_DBS + Utxos::NUM_DBS)
            .open(datadir.join("data.mdb"))
            .into_diagnostic()?;
        let mempool = Mempool::new(&env)?;
        let archive = Archive::new(&env)?;
        let utxos = Utxos::new(&env)?;
        Ok(Self {
            env,
            mempool,
            archive,
            utxos,
        })
    }

    pub fn is_clean(&self) -> Result<bool> {
        let txn = self.env.read_txn().into_diagnostic()?;
        self.utxos.is_empty(&txn)
    }

    pub fn get_chain_tip(&self) -> Result<Option<(u32, (Header, (u64, u64)))>> {
        let txn = self.env.read_txn().into_diagnostic()?;
        let chain_tip = self.archive.get_chain_tip(&txn)?;
        Ok(chain_tip)
    }

    pub fn get_main_chain_tip(&self) -> Result<[u8; HASH_LENGTH]> {
        let txn = self.env.read_txn().into_diagnostic()?;
        let chain_tip = self.utxos.get_main_chain_tip(&txn)?;
        Ok(chain_tip)
    }

    pub fn get_pending_transactions(&self) -> Result<Vec<Transaction>> {
        let mut txn = self.env.read_txn().into_diagnostic()?;
        let transactions = self.mempool.get_pending_transactions(&mut txn)?;
        Ok(transactions)
    }

    pub fn submit_transaction(&self, transaction: &Transaction) -> Result<()> {
        let mut txn = self.env.write_txn().into_diagnostic()?;
        let fee = self.utxos.get_transaction_fee(&txn, transaction)?;
        self.mempool
            .submit_transaction(&mut txn, transaction, fee)?;
        txn.commit().into_diagnostic()?;
        Ok(())
    }

    pub fn load_deposits(
        &self,
        deposits: &[Deposit],
        main_block_height: u32,
        main_chain_tip: &[u8; HASH_LENGTH],
    ) -> Result<()> {
        let mut txn = self.env.write_txn().into_diagnostic()?;
        for deposit in deposits {
            let outpoint = OutPoint::Deposit {
                sequence_number: deposit.sequence_number,
            };
            let output = Output::Regular {
                address: deposit.address.clone().try_into().unwrap(),
                value: deposit.value,
            };
            self.utxos.add_utxo(&mut txn, &outpoint, &output)?;
            println!("{outpoint} -> {output}");
        }
        self.utxos
            .set_main_block_height(&mut txn, main_block_height)?;
        self.utxos.set_main_chain_tip(&mut txn, main_chain_tip)?;
        txn.commit().into_diagnostic()?;
        Ok(())
    }

    fn is_valid(&self, header: &Header, transactions: &[Transaction]) -> Result<()> {
        todo!();
    }

    fn connect(&self, txn: &mut RwTxn, header: Header, transactions: &[Transaction]) -> Result<()> {
        self.archive.connect(txn, header, transactions)?;
        self.utxos.connect(txn, transactions)?;
        self.mempool.connect(txn, transactions)?;
        Ok(())
    }

    fn disconnect(&self, number: u32) -> Result<()> {
        todo!();
    }

    pub fn connect_main_block(&self, block: &MainBlock) -> Result<()> {
        let mut txn = self.env.write_txn().into_diagnostic()?;
        for deposit in &block.deposits {
            let (outpoint, output) = deposit;
            self.utxos.add_utxo(&mut txn, outpoint, output)?;
        }
        if let Some(withdrawal_bundle_event) = &block.withdrawal_bundle_event {
            match withdrawal_bundle_event.withdrawal_bundle_event_type {
                WithdrawalBundleEventType::Submitted => {
                    todo!()
                }
                WithdrawalBundleEventType::Succeded => {
                    todo!()
                }
                WithdrawalBundleEventType::Failed => {
                    todo!()
                }
            }
        }
        let (header, transactions) = {
            let prev_side_block_hash = {
                if let Some((
                    _block_number,
                    (prev_header, (_transaction_range_start, _transaction_range_end)),
                )) = self.archive.get_chain_tip(&txn)?
                {
                    prev_header.hash()
                } else {
                    [0; HASH_LENGTH]
                }
            };
            let transactions = self.mempool.get_pending_transactions(&txn)?;
            let merkle_root = Header::compute_merkle_root(&transactions);
            let header = Header {
                prev_side_block_hash,
                merkle_root,
            };
            (header, transactions)
        };
        let block_hash = header.hash();
        for bmm_hash in &block.bmm_hashes {
            if block_hash == *bmm_hash {
                self.connect(&mut txn, header, &transactions)?;
                break;
            }
        }
        let mut main_block_height = self.utxos.get_main_block_height(&txn)?;
        main_block_height += 1;
        if main_block_height != block.block_height {
            return Err(miette!("invalid main block height"));
        }
        self.utxos
            .set_main_block_height(&mut txn, main_block_height)?;
        self.utxos.set_main_chain_tip(&mut txn, &block.block_hash)?;
        txn.commit().into_diagnostic()?;
        Ok(())
    }
}
