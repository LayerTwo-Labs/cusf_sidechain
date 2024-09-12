use bip300301_enforcer_proto::validator::{
    validator_client::ValidatorClient, GetDepositsRequest, GetMainBlockHeightRequest,
    GetMainChainTipRequest, GetMainChainTipResponse,
};
use cusf_sidechain_types::{
    Hashable, Header, MainBlock, OutPoint, Output, Transaction, HASH_LENGTH,
};
use miette::{IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};
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

    pub async fn collect_transactions(&self) -> Result<Vec<Transaction>> {
        let transactions = self.state.collect_transactions()?;
        Ok(transactions)
    }

    pub async fn get_chain_tip(&self) -> Result<(u32, [u8; HASH_LENGTH])> {
        let chain_tip = self.state.get_chain_tip()?;
        let (block_height, prev_side_block_hash) = match chain_tip {
            Some((block_height, (header, (_transaction_range_start, _transaction_range_end)))) => {
                (block_height, header.hash())
            }
            None => (0, [0; HASH_LENGTH]),
        };
        Ok((block_height, prev_side_block_hash))
    }

    pub fn get_utxo_set(&self) -> Result<HashMap<OutPoint, Output>> {
        self.state.get_utxo_set()
    }

    pub fn get_withdrawal_bundle(&self) -> Result<bitcoin::Transaction> {
        self.state.get_withdrawal_bundle()
    }

    pub fn submit_transaction(&self, transaction: &Transaction) -> Result<()> {
        self.state.submit_transaction(transaction)?;
        Ok(())
    }

    pub fn submit_block(
        &self,
        header: Header,
        coinbase: &[Output],
        transactions: &[Transaction],
    ) -> Result<()> {
        self.state.connect(header, coinbase, transactions)?;
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
        let main_chain_tip = self
            .client
            .get_main_chain_tip(GetMainChainTipRequest {})
            .await
            .into_diagnostic()?
            .into_inner()
            .block_hash;
        let main_chain_tip: [u8; HASH_LENGTH] = main_chain_tip.try_into().unwrap();
        self.state
            .load_deposits(&deposits, main_block_height, &main_chain_tip)?;
        Ok(())
    }

    pub fn connect_main_block(&self, block: &MainBlock) -> Result<()> {
        self.state.connect_main_block(block)?;
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
