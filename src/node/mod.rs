use miette::Result;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, path::PathBuf};

use crate::state::State;

pub struct Node {
    config: Config,
    state: State,
    mainchain_client: (),
    // Propagating bytes through the network.
    network_manager: (),
}

impl Node {
    fn run(&self) -> Result<()> {
        todo!();
    }
}

#[derive(Serialize, Deserialize)]
struct Config {
    mainchain_addr: SocketAddr,
    sidechain_addr: SocketAddr,
    network_addr: SocketAddr,
    datadir: PathBuf,
}
