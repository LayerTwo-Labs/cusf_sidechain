mod archive;
mod mempool;
mod utxos;

use archive::Archive;
use bip300301_enforcer_proto::validator::Deposit;
use cusf_sidechain_types::{
    Header, MainBlock, OutPoint, Output, Transaction, WithdrawalBundleEventType, HASH_LENGTH,
};
use heed::{Env, EnvOpenOptions};
use mempool::Mempool;
use miette::{miette, IntoDiagnostic, Result};
use std::{collections::HashMap, path::Path};
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

    pub fn get_utxo_set(&self) -> Result<HashMap<OutPoint, Output>> {
        let txn = self.env.read_txn().into_diagnostic()?;
        let utxos = self.utxos.get_utxo_set(&txn)?;
        Ok(utxos)
    }

    pub fn get_withdrawal_bundle(&self) -> Result<bitcoin::Transaction> {
        let mut txn = self.env.write_txn().into_diagnostic()?;
        self.utxos.collect_withdrawals(&mut txn)?;
        let bundle = self.utxos.get_withdrawal_bundle(&txn)?;
        // txn.commit().into_diagnostic()?;
        Ok(bundle)
    }

    pub fn get_main_chain_tip(&self) -> Result<[u8; HASH_LENGTH]> {
        let txn = self.env.read_txn().into_diagnostic()?;
        let chain_tip = self.utxos.get_main_chain_tip(&txn)?;
        Ok(chain_tip)
    }

    pub fn collect_transactions(&self) -> Result<Vec<Transaction>> {
        let txn = self.env.read_txn().into_diagnostic()?;
        let transactions = self.mempool.collect_transactions(&txn)?;
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

    fn is_valid(
        &self,
        header: &Header,
        coinbase: &[Output],
        transactions: &[Transaction],
    ) -> Result<()> {
        todo!();
    }

    pub fn connect(
        &self,
        header: Header,
        coinbase: &[Output],
        transactions: &[Transaction],
    ) -> Result<()> {
        let mut txn = self.env.write_txn().into_diagnostic()?;
        self.archive.validate_header(&txn, &header)?;
        let block_height = self
            .archive
            .get_chain_tip(&txn)?
            .map(|(block_height, (_header, (_, _)))| block_height + 1)
            .unwrap_or(0);
        self.archive
            .connect(&mut txn, header, coinbase, transactions)?;
        self.utxos
            .connect(&mut txn, block_height, coinbase, transactions)?;
        self.mempool.connect(&mut txn, transactions)?;
        txn.commit().into_diagnostic()?;
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
                    self.utxos
                        .submit_bundle(&mut txn, &withdrawal_bundle_event.m6id)?;
                }
                WithdrawalBundleEventType::Succeded => {
                    self.utxos
                        .succeed_bundle(&mut txn, &withdrawal_bundle_event.m6id)?;
                }
                WithdrawalBundleEventType::Failed => {
                    self.utxos
                        .fail_bundle(&mut txn, &withdrawal_bundle_event.m6id)?;
                }
            }
        }
        let mut main_block_height = self.utxos.get_main_block_height(&txn)?;
        main_block_height += 1;
        if main_block_height != block.block_height {
            return Err(miette!("invalid main block height"));
        }
        self.archive.add_bmm_hashes(&mut txn, &block.bmm_hashes)?;
        self.utxos
            .set_main_block_height(&mut txn, main_block_height)?;
        self.utxos.set_main_chain_tip(&mut txn, &block.block_hash)?;
        txn.commit().into_diagnostic()?;
        Ok(())
    }
}
