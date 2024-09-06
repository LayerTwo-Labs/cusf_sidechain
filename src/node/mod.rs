use bip300301_enforcer_proto::validator::{
    validator_client::ValidatorClient, GetDepositsRequest, GetMainBlockHeightRequest,
    GetMainChainTipRequest, GetMainChainTipResponse,
};
use cusf_sidechain_types::{Hashable, Header, MainBlock, Transaction, HASH_LENGTH};
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

    pub async fn get_next_block(&self) -> Result<(Header, Vec<Transaction>)> {
        let transactions = self.state.get_pending_transactions()?;
        let merkle_root = Header::compute_merkle_root(&transactions);
        let chain_tip = self.state.get_chain_tip()?;
        let prev_side_block_hash = match chain_tip {
            Some((_block_height, (header, (_transaction_range_start, _transaction_range_end)))) => {
                header.hash()
            }
            None => [0; HASH_LENGTH],
        };
        let header = Header {
            prev_side_block_hash,
            merkle_root,
        };
        Ok((header, transactions))
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
