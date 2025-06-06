pub mod client_interceptor;
pub mod cluster_data_impl;
pub mod convert;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};


use bytes::Bytes;
use futures::{StreamExt};
use jito_protos::{
    auth::auth_service_client::AuthServiceClient,
    bundle::{Bundle, BundleResult},
    searcher::{
        mempool_subscription, searcher_service_client::SearcherServiceClient, MempoolSubscription,
        ProgramSubscriptionV0, SendBundleRequest, SubscribeBundleResultsRequest,
        WriteLockedAccountSubscriptionV0,
    },
};
use log::*;

use solana_sdk::{
    clock::Slot, pubkey::Pubkey, signature::Keypair, transaction::VersionedTransaction,
};
use thiserror::Error;
use tokio::sync::{
    mpsc::{channel, Receiver},
    Mutex,
};
use tonic::{
    codegen::{Body, StdError},
    service::interceptor::InterceptedService,
    transport,
    transport::{Channel, Endpoint},
    Status,
};

use crate::jito::convert::{proto_packet_from_versioned_tx, versioned_tx_from_packet};

use self::{client_interceptor::ClientInterceptor, cluster_data_impl::ClusterDataImpl};

/// BundleId is expected to be a hash of the contained transaction signatures:
/// fn derive_bundle_id(transactions: &[VersionedTransaction]) -> String {
///     let mut hasher = Sha256::new();
///     hasher.update(transactions.iter().map(|tx| tx.signatures[0]).join(","));
///     format!("{:x}", hasher.finalize())
/// }
pub type BundleId = String;

#[derive(Error, Debug)]
pub enum SearcherClientError {
    #[error("block-engine transport error {0}")]
    BlockEngineTransportError(#[from] transport::Error),

    #[error("no upcoming validator is running jito-solana")]
    NoUpcomingJitoValidator,

    #[error("grpc client error {0}")]
    GrpcClientError(#[from] Status),

    #[error("the grpc stream was closed")]
    GrpcStreamClosed,

    #[error("error serializing transaction")]
    TransactionSerializationError,

    #[error("tpu client error")]
    TpuClientError,
}

pub type SearcherClientResult<T> = Result<T, SearcherClientError>;

#[tonic::async_trait]
pub trait ClusterData {
    async fn current_slot(&self) -> Slot;
    async fn next_jito_validator(&self) -> Option<(Pubkey, Slot)>;
}

#[derive(Clone)]
pub struct SearcherClient<C: ClusterData, T> {
    cluster_data: Arc<C>,
    searcher_service_client: Arc<Mutex<SearcherServiceClient<T>>>,
    exit: Arc<AtomicBool>,
}

impl<C: ClusterData + Clone, T> SearcherClient<C, T>
where
    T: tonic::client::GrpcService<tonic::body::BoxBody>,
    T::Error: Into<StdError>,
    T::ResponseBody: Body<Data = Bytes> + Send + 'static,
    <T::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    pub fn new(
        cluster_data: C,
        searcher_service_client: SearcherServiceClient<T>,
        exit: Arc<AtomicBool>,
    ) -> Self {
        Self {
            searcher_service_client: Arc::new(Mutex::new(searcher_service_client)),
            cluster_data: Arc::new(cluster_data),
            exit,
        }
    }

    /// Sends the list of transactions as a bundle iff the leader is a jito-solana.
    /// Returns the bundle's id.
    pub async fn send_bundle(
        &self,
        transactions: Vec<VersionedTransaction>,
    ) -> SearcherClientResult<BundleId> {
        let resp = self
            .searcher_service_client
            .lock()
            .await
            .send_bundle(SendBundleRequest {
                bundle: Some(Bundle {
                    header: None,
                    packets: transactions
                        .iter()
                        .map(proto_packet_from_versioned_tx)
                        .collect(),
                }),
            })
            .await?;

        Ok(resp.into_inner().uuid)
    }

    /// Sends transactions through the normal pipeline, regardless of if the leader is running jito-solana.
    /// Returns a list of results corresponding to the supplied transactions ordering.
    // pub async fn send_transactions(
    //     &self,
    //     tpu_client: &TpuClient,
    //     transactions: Vec<VersionedTransaction>,
    // ) -> Vec<SearcherClientResult<()>> {
    //     let futs = transactions
    //         .into_iter()
    //         .map(|tx| async move {
    //             let serialized_tx = serialize(&tx)
    //                 .map_err(|_e| SearcherClientError::TransactionSerializationError)?;
    //             if !tpu_client.send_wire_transaction(serialized_tx).await {
    //                 Err(SearcherClientError::TpuClientError)
    //             } else {
    //                 Ok(())
    //             }
    //         })
    //         .collect::<Vec<_>>();

    //     join_all(futs).await.into_iter().collect()
    // }

    pub async fn subscribe_mempool_accounts(
        &self,
        accounts: &[Pubkey],
        // Regions to subscribe to
        regions: Vec<String>,
        buffer_size: usize,
    ) -> SearcherClientResult<Receiver<Vec<VersionedTransaction>>> {
        let (sender, receiver) = channel(buffer_size);

        let mut stream = self
            .searcher_service_client
            .lock()
            .await
            .subscribe_mempool(MempoolSubscription {
                msg: Some(mempool_subscription::Msg::WlaV0Sub(
                    WriteLockedAccountSubscriptionV0 {
                        accounts: accounts.iter().map(|account| account.to_string()).collect(),
                    },
                )),
                regions,
            })
            .await?
            .into_inner();

        let exit = self.exit.clone();
        tokio::spawn(async move {
            while !exit.load(Ordering::Relaxed) {
                let msg = match stream.next().await {
                    None => {
                        error!("mempool stream closed");
                        return;
                    }
                    Some(res) => {
                        if let Err(e) = res {
                            error!("mempool stream received error status: {e}");
                            return;
                        }
                        res.unwrap()
                    }
                };

                let transactions = msg
                    .transactions
                    .iter()
                    .filter_map(versioned_tx_from_packet)
                    .collect();

                if let Err(e) = sender.send(transactions).await {
                    error!("error sending transactions: {e}");
                    return;
                }
            }
        });

        Ok(receiver)
    }

    pub async fn subscribe_mempool_programs(
        &self,
        accounts: &[Pubkey],
        // Regions to subscribe to
        regions: Vec<String>,
        buffer_size: usize,
    ) -> SearcherClientResult<Receiver<Vec<VersionedTransaction>>> {
        let (sender, receiver) = channel(buffer_size);

        let mut stream = self
            .searcher_service_client
            .lock()
            .await
            .subscribe_mempool(MempoolSubscription {
                msg: Some(mempool_subscription::Msg::ProgramV0Sub(
                    ProgramSubscriptionV0 {
                        programs: accounts.iter().map(|account| account.to_string()).collect(),
                    },
                )),
                regions,
            })
            .await?
            .into_inner();

        let exit = self.exit.clone();
        tokio::spawn(async move {
            while !exit.load(Ordering::Relaxed) {
                let msg = match stream.next().await {
                    None => {
                        error!("mempool stream closed");
                        return;
                    }
                    Some(res) => {
                        if let Err(e) = res {
                            error!("mempool stream received error status: {e}");
                            return;
                        }
                        res.unwrap()
                    }
                };

                let transactions = msg
                    .transactions
                    .iter()
                    .filter_map(versioned_tx_from_packet)
                    .collect();


                if let Err(e) = sender.send(transactions).await {
                    error!("error sending transactions: {e}");
                    return;
                }
            }
        });

        Ok(receiver)
    }

    pub async fn subscribe_bundle_results(
        &self,
        buffer_size: usize,
    ) -> SearcherClientResult<Receiver<BundleResult>> {
        let (sender, receiver) = channel(buffer_size);

        let mut stream = self
            .searcher_service_client
            .lock()
            .await
            .subscribe_bundle_results(SubscribeBundleResultsRequest {})
            .await?
            .into_inner();

        let exit = self.exit.clone();
        tokio::spawn(async move {
            while !exit.load(Ordering::Relaxed) {
                let msg = match stream.next().await {
                    None => {
                        error!("bundle results stream closed");
                        return;
                    }
                    Some(res) => {
                        if let Err(e) = res {
                            error!("bundle results stream received error status: {e}");
                            return;
                        }
                        res.unwrap()
                    }
                };

                if let Err(e) = sender.send(msg).await {
                    error!("error sending bundle result: {e}");
                    return;
                }
            }
        });

        Ok(receiver)
    }
}

pub async fn grpc_connect(url: &str) -> SearcherClientResult<Channel> {
    let endpoint = if url.contains("https") {
        Endpoint::from_shared(url.to_string())
            .expect("invalid url")
            .tls_config(transport::ClientTlsConfig::new())
    } else {
        Endpoint::from_shared(url.to_string())
    }?;

    Ok(endpoint.connect().await?)
}

pub mod utils {
    use solana_sdk::pubkey::Pubkey;

    pub fn derive_tip_accounts(tip_program_pubkey: &Pubkey) -> Vec<Pubkey> {
        let tip_pda_0 = Pubkey::find_program_address(&[b"TIP_ACCOUNT_0"], tip_program_pubkey).0;
        let tip_pda_1 = Pubkey::find_program_address(&[b"TIP_ACCOUNT_1"], tip_program_pubkey).0;
        let tip_pda_2 = Pubkey::find_program_address(&[b"TIP_ACCOUNT_2"], tip_program_pubkey).0;
        let tip_pda_3 = Pubkey::find_program_address(&[b"TIP_ACCOUNT_3"], tip_program_pubkey).0;
        let tip_pda_4 = Pubkey::find_program_address(&[b"TIP_ACCOUNT_4"], tip_program_pubkey).0;
        let tip_pda_5 = Pubkey::find_program_address(&[b"TIP_ACCOUNT_5"], tip_program_pubkey).0;
        let tip_pda_6 = Pubkey::find_program_address(&[b"TIP_ACCOUNT_6"], tip_program_pubkey).0;
        let tip_pda_7 = Pubkey::find_program_address(&[b"TIP_ACCOUNT_7"], tip_program_pubkey).0;

        vec![
            tip_pda_0, tip_pda_1, tip_pda_2, tip_pda_3, tip_pda_4, tip_pda_5, tip_pda_6, tip_pda_7,
        ]
    }
}

pub async fn get_searcher_client(
    auth_keypair: &Arc<Keypair>,
    exit: &Arc<AtomicBool>,
    block_engine_url: &str,
    rpc_pubsub_addr: &str,
) -> SearcherClientResult<(
    SearcherClient<ClusterDataImpl, InterceptedService<Channel, ClientInterceptor>>,
    ClusterDataImpl,
)> {
    let auth_channel = grpc_connect(block_engine_url).await?;
    let client_interceptor =
        ClientInterceptor::new(AuthServiceClient::new(auth_channel), auth_keypair).await?;

    let searcher_channel = grpc_connect(block_engine_url).await?;
    let searcher_service_client =
        SearcherServiceClient::with_interceptor(searcher_channel, client_interceptor);

    let cluster_data_impl = ClusterDataImpl::new(
        rpc_pubsub_addr.to_string(),
        searcher_service_client.clone(),
        exit.clone(),
    )
    .await;

    Ok((
        SearcherClient::new(
            cluster_data_impl.clone(),
            searcher_service_client,
            exit.clone(),
        ),
        cluster_data_impl,
    ))
}