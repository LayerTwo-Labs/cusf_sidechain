use bip300301_enforcer_proto::validator::{
    validator_client::ValidatorClient, GetDepositsRequest, GetMainBlockHeightRequest,
};
use cusf_sidechain_types::Transaction;
use miette::{IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tonic::transport::Channel;

use crate::state::State;

#[derive(Clone)]
pub struct Node {
    config: Config,
    state: State,
    client: ValidatorClient<Channel>,
}

impl Node {
    pub async fn new(datadir: &Path) -> Result<Self> {
        let config = confy::load("cusf_sidechain", None).into_diagnostic()?;
        let state = State::new(datadir)?;
        let client = ValidatorClient::connect("http://[::1]:50051")
            .await
            .into_diagnostic()?;
        Ok(Self {
            config,
            state,
            client,
        })
    }

    pub fn is_clean(&self) -> Result<bool> {
        self.state.is_clean()
    }

    pub fn submit_transaction(&self, transaction: &Transaction) -> Result<()> {
        self.state.submit_transaction(transaction)?;
        Ok(())
    }

    pub async fn initial_sync(&mut self) -> Result<()> {
        let deposits = self
            .client
            .get_deposits(GetDepositsRequest {
                sidechain_number: 0,
            })
            .await
            .into_diagnostic()?
            .into_inner()
            .deposits;
        let main_block_height = self
            .client
            .get_main_block_height(GetMainBlockHeightRequest {})
            .await
            .into_diagnostic()?
            .into_inner()
            .height;
        dbg!(main_block_height);
        self.state.load_deposits(&deposits, main_block_height)?;
        Ok(())
    }

    fn run(&self) -> Result<()> {
        todo!();
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct Config {}

impl Default for Config {
    fn default() -> Self {
        Self {}
    }
}
