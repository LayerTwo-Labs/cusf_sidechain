use crate::node::Node;
use cusf_sidechain_proto::sidechain::{
    sidechain_server::Sidechain, CollectTransactionsRequest, CollectTransactionsResponse,
    ConnectMainBlockRequest, ConnectMainBlockResponse, DisconnectMainBlockRequest,
    DisconnectMainBlockResponse, GetChainTipRequest, GetChainTipResponse, SubmitTransactionRequest,
    SubmitTransactionResponse,
};
use cusf_sidechain_types::Transaction;
use tonic::{Request, Response, Status};

#[derive(Clone)]
pub struct Plain {
    node: Node,
}

impl Plain {
    pub fn new(node: Node) -> Self {
        Self { node }
    }
}

#[tonic::async_trait]
impl Sidechain for Plain {
    async fn submit_transaction(
        &self,
        request: Request<SubmitTransactionRequest>,
    ) -> Result<Response<SubmitTransactionResponse>, Status> {
        let transaction_bytes = request.into_inner().transaction;
        let transaction: Transaction = bincode::deserialize(&transaction_bytes).unwrap();
        dbg!(&transaction);
        self.node.submit_transaction(&transaction).unwrap();
        let response = SubmitTransactionResponse {};
        Ok(Response::new(response))
    }

    async fn collect_transactions(
        &self,
        request: Request<CollectTransactionsRequest>,
    ) -> Result<Response<CollectTransactionsResponse>, Status> {
        let transactions = self.node.collect_transactions().await.unwrap();
        let transactions = bincode::serialize(&transactions).unwrap();
        let response = CollectTransactionsResponse { transactions };
        Ok(Response::new(response))
    }

    async fn get_chain_tip(
        &self,
        request: Request<GetChainTipRequest>,
    ) -> Result<Response<GetChainTipResponse>, Status> {
        let (block_height, block_hash) = self.node.get_chain_tip().await.unwrap();
        let response = GetChainTipResponse {
            block_height,
            block_hash: block_hash.to_vec(),
        };
        Ok(Response::new(response))
    }

    async fn connect_main_block(
        &self,
        request: Request<ConnectMainBlockRequest>,
    ) -> Result<Response<ConnectMainBlockResponse>, Status> {
        todo!();
    }

    async fn disconnect_main_block(
        &self,
        request: Request<DisconnectMainBlockRequest>,
    ) -> Result<Response<DisconnectMainBlockResponse>, Status> {
        todo!();
    }
}
