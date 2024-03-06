mod archive;
mod mempool;
mod utxos;

use archive::Archive;
use heed::Env;
use mempool::Mempool;
use miette::Result;
use utxos::Utxos;

use crate::types::{Header, Transaction};

pub struct State {
    env: Env,
    utxos: Utxos,
    archive: Archive,
    mempool: Mempool,
}

impl State {
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
