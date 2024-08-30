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

use crate::types::{Header, OutPoint, Output, Transaction};

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

    pub fn load_deposits(&self, deposits: &[Deposit]) -> Result<()> {
        let mut txn = self.env.write_txn().into_diagnostic()?;
        for deposit in deposits {
            let outpoint = OutPoint::new(true, deposit.sequence_number, 0);
            let output = Output::Regular {
                address: deposit.address.clone().try_into().unwrap(),
                value: deposit.value,
            };
            self.utxos.add_utxo(&mut txn, &outpoint, &output)?;
            println!("{outpoint} -> {output}");
        }
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
