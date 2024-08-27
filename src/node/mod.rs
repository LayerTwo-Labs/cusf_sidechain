use bip300301_enforcer_proto::validator::validator_client::ValidatorClient;
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
        let config_str = std::fs::read_to_string(datadir.join("config.toml")).into_diagnostic()?;
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
