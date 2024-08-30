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
    let outpoint = OutPoint::new(true, 1002, 99);

    let output = Output::Regular {
        address: [0; ADDRESS_LENGTH],
        value: 0,
    };

    let outpoint_bytes = bincode::serialize(&outpoint).into_diagnostic()?;
    let output_bytes = bincode::serialize(&output).into_diagnostic()?;

    println!("{outpoint}");
    dbg!(outpoint_bytes.len());
    dbg!(output_bytes.len());
    dbg!(hex::encode(outpoint.hash()));
    dbg!(hex::encode(output.hash()));
    dbg!(hex::encode(output.address()));
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
