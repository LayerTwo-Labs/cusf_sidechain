mod authorization;
mod net;
mod node;
mod server;
mod state;
mod types;

use miette::{miette, IntoDiagnostic, Result};
use node::Node;
use sidechain_proto::sidechain::sidechain_server::SidechainServer;
use tonic::transport::Server;
use types::{Hashable, OutPoint, Output, ADDRESS_LENGTH};

#[tokio::main]
async fn main() -> Result<()> {
    let datadir = dirs::data_dir()
        .ok_or(miette!("couldn't get datadir"))?
        .join("cusf_sidechain");
    let mut node = Node::new(&datadir).await?;
    node.initial_sync().await?;
    let plain = server::Plain::new(node);
    let addr = "[::1]:50052".parse().into_diagnostic()?;
    println!("Listening for gRPC on {addr}");
    Server::builder()
        .add_service(SidechainServer::new(plain))
        .serve(addr)
        .await
        .into_diagnostic()?;

    Ok(())
}

// 1. Get an array of all deposits from the BIP300301 enforcer
// 2. Load all the deposits into the UTXO set database
