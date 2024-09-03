use crate::node::Node;
use cusf_sidechain_proto::sidechain::{
    sidechain_server::Sidechain, GetNextBlockRequest, GetNextBlockResponse,
    SubmitTransactionRequest, SubmitTransactionResponse,
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

    async fn get_next_block(
        &self,
        request: Request<GetNextBlockRequest>,
    ) -> Result<Response<GetNextBlockResponse>, Status> {
        let block = self.node.get_next_block().await.unwrap();
        let block_bytes = bincode::serialize(&block).unwrap();
        let response = GetNextBlockResponse { block: block_bytes };
        Ok(Response::new(response))
    }
}
