use crate::node::Node;
use cusf_sidechain_proto::sidechain::{
    sidechain_server::Sidechain, CollectTransactionsRequest, CollectTransactionsResponse,
    ConnectMainBlockRequest, ConnectMainBlockResponse, DisconnectMainBlockRequest,
    DisconnectMainBlockResponse, GetChainTipRequest, GetChainTipResponse, GetUtxoSetRequest,
    GetUtxoSetResponse, SubmitBlockRequest, SubmitBlockResponse, SubmitTransactionRequest,
    SubmitTransactionResponse,
};
use cusf_sidechain_types::{
    Hashable, Header, MainBlock, OutPoint, Output, Transaction, WithdrawalBundleEvent,
    WithdrawalBundleEventType, HASH_LENGTH,
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
        let transaction_bytes = request.into_inner().transaction;
        let transaction: Transaction = bincode::deserialize(&transaction_bytes).unwrap();
        dbg!(&transaction);
        self.node.submit_transaction(&transaction).unwrap();
        let response = SubmitTransactionResponse {};
        Ok(Response::new(response))
    }

    async fn submit_block(
        &self,
        request: Request<SubmitBlockRequest>,
    ) -> Result<Response<SubmitBlockResponse>, Status> {
        let block_bytes = request.into_inner().block;
        let (header, coinbase, transactions): (Header, Vec<Output>, Vec<Transaction>) =
            bincode::deserialize(&block_bytes).unwrap();
        let block_hash = hex::encode(&header.hash());
        println!("block {} submitted", block_hash);
        self.node
            .submit_block(header, &coinbase, &transactions)
            .unwrap();
        let response = SubmitBlockResponse {};
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
        let main_block = request.into_inner();
        let block_height = main_block.block_height;
        let block_hash: [u8; HASH_LENGTH] = main_block.block_hash.try_into().unwrap();
        let deposits = main_block.deposits;
        let deposits = deposits
            .into_iter()
            .map(|deposit| {
                (
                    OutPoint::Deposit {
                        sequence_number: deposit.sequence_number,
                    },
                    Output::Regular {
                        address: deposit.address.try_into().unwrap(),
                        value: deposit.value,
                    },
                )
            })
            .collect();
        let withdrawal_bundle_event =
            main_block
                .withdrawal_bundle_event
                .map(|withdrawal_bundle_event| WithdrawalBundleEvent {
                    withdrawal_bundle_event_type: match withdrawal_bundle_event
                        .withdrawal_bundle_event_type
                    {
                        0 => WithdrawalBundleEventType::Submitted,
                        1 => WithdrawalBundleEventType::Failed,
                        2 => WithdrawalBundleEventType::Succeded,
                        _ => todo!(),
                    },
                    m6id: withdrawal_bundle_event.m6id.try_into().unwrap(),
                });
        let bmm_hashes = main_block
            .bmm_hashes
            .into_iter()
            .map(|bmm_hash| bmm_hash.try_into().unwrap())
            .collect();
        let block = MainBlock {
            block_height,
            block_hash,
            deposits,
            withdrawal_bundle_event,
            bmm_hashes,
        };
        self.node.connect_main_block(&block).unwrap();
        let response = ConnectMainBlockResponse {};
        Ok(Response::new(response))
    }

    async fn disconnect_main_block(
        &self,
        request: Request<DisconnectMainBlockRequest>,
    ) -> Result<Response<DisconnectMainBlockResponse>, Status> {
        todo!();
    }

    async fn get_utxo_set(
        &self,
        request: Request<GetUtxoSetRequest>,
    ) -> Result<Response<GetUtxoSetResponse>, Status> {
        let utxos = self.node.get_utxo_set().unwrap();
        let utxos = bincode::serialize(&utxos).unwrap();
        let response = GetUtxoSetResponse { utxos };
        Ok(Response::new(response))
    }
}
