use crate::node::Node;
use sidechain_proto::sidechain::{
    sidechain_server::Sidechain, SubmitTransactionRequest, SubmitTransactionResponse,
};
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
        let response = SubmitTransactionResponse {};
        Ok(Response::new(response))
    }
}
