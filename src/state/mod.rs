mod archive;
mod mempool;
mod utxos;

use std::path::Path;

use archive::Archive;
use bip300301_enforcer_proto::validator::Deposit;
use heed::{Env, EnvOpenOptions};
use mempool::Mempool;
use miette::{IntoDiagnostic, Result};
use utxos::Utxos;

use cusf_sidechain_types::{Header, OutPoint, Output, Transaction};

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

    pub fn load_deposits(&self, deposits: &[Deposit], main_block_height: u32) -> Result<()> {
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
        txn.commit().into_diagnostic()?;
        Ok(())
    }

    fn is_valid(&self, header: &Header, transactions: &[Transaction]) -> Result<()> {
        todo!();
    }

    fn connect(&self, header: &Header, transactions: &[Transaction]) -> Result<()> {
        todo!();
    }

    fn disconnect(&self, number: u32) -> Result<()> {
        todo!();
    }
}
