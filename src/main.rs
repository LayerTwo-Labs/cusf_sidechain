mod authorization;
mod net;
mod node;
mod server;
mod state;

use cusf_sidechain_proto::sidechain::sidechain_server::SidechainServer;
use miette::{miette, IntoDiagnostic, Result};
use node::Node;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<()> {
    let datadir = dirs::data_dir()
        .ok_or(miette!("couldn't get datadir"))?
        .join("cusf_sidechain");
    let mut node = Node::new(&datadir).await?;
    if node.is_clean()? {
        node.initial_sync().await?;
    }
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
