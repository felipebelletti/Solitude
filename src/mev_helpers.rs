use jito::{
    client_interceptor::ClientInterceptor, cluster_data_impl::ClusterDataImpl, SearcherClient,
    SearcherClientError,
};
use jito_protos::bundle::BundleResult;
use solana_program::pubkey::Pubkey;
use solana_sdk::{signature::Keypair, transaction::VersionedTransaction};
use std::{
    error::Error,
    panic::{self, PanicInfo},
    process,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{sync::mpsc, task::JoinHandle};
use tonic::{service::interceptor::InterceptedService, transport::Channel};

use crate::{
    config::wallet,
    jito::{self, BundleId},
};

pub struct MevHelpers {
    pub searcher_clients:
        Vec<Arc<SearcherClient<ClusterDataImpl, InterceptedService<Channel, ClientInterceptor>>>>,
}

impl MevHelpers {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let jito_auth_keypair = Arc::new(
            Keypair::from_bytes(
                &bs58::decode("584Tm5pC9qFrxJ1QLS9mUuAbwXX2U99XYuYrGLUpRnkauCToR8vL3asnTKKvadTQqcxAQjuiMkwsDNpUcoQnp7HM")
                    .into_vec()
                    .unwrap(),
            )
            .unwrap(),
        );

        let rpc_pubsub_addr = "http://127.0.0.1:8900";

        let block_engine_urls = {
            if wallet::read_from_wallet_file().testnet {
                vec![
                    // "https://dallas.testnet.block-engine.jito.wtf",
                    "https://ny.testnet.block-engine.jito.wtf"
                ]
            } else {
                vec![
                    "https://frankfurt.mainnet.block-engine.jito.wtf",
                    "https://amsterdam.mainnet.block-engine.jito.wtf",
                    "https://ny.mainnet.block-engine.jito.wtf",
                    // "https://tokyo.mainnet.block-engine.jito.wtf",
                ]
            }
        };

        let mut searcher_clients = Vec::new();
        for url in block_engine_urls.iter() {
            println!("Initializing Jito MEV on {}", url);

            let (searcher_client, _) = jito::get_searcher_client(
                &jito_auth_keypair,
                &graceful_panic(None),
                url,
                rpc_pubsub_addr,
            )
            .await
            .expect("get_searcher_client failed");

            searcher_clients.push(Arc::new(searcher_client));
        }

        Ok(Self { searcher_clients })
    }

    pub async fn subscribe_mempool_programs(
        &self,
        watch_mempool_addresses: &[Pubkey],
    ) -> mpsc::Receiver<VersionedTransaction> {
        let (tx, rx) = mpsc::channel(1024);

        for searcher_client in self.searcher_clients.iter() {
            let mut mempool_ch = searcher_client
                .subscribe_mempool_programs(
                    watch_mempool_addresses,
                    vec![
                        // "amsterdam".to_string(),
                        // "frankfurt".to_string(),
                        // "ny".to_string(),
                        // "tokyo".to_string(),
                    ],
                    1024,
                )
                .await
                .expect("Failed to subscribe to mempool programs");

            let tx_clone = tx.clone();
            tokio::spawn(async move {
                while let Some(mempool_txs) = mempool_ch.recv().await {
                    for mempool_tx in mempool_txs {
                        if let Err(e) = tx_clone.send(mempool_tx).await {
                            eprintln!("Failed to send transaction: {}", e);
                            break;
                        }
                    }
                }
            });
        }

        rx
    }

    pub async fn susbcribe_mempool_accounts(
        &self,
        watch_mempool_addresses: &[Pubkey],
    ) -> mpsc::Receiver<VersionedTransaction> {
        let (tx, rx) = mpsc::channel(1024);

        for searcher_client in self.searcher_clients.iter() {
            let mut mempool_ch = searcher_client
                .subscribe_mempool_accounts(
                    watch_mempool_addresses,
                    vec![
                        // "amsterdam".to_string(),
                        // "frankfurt".to_string(),
                        // "ny".to_string(),
                        // "tokyo".to_string(),
                    ],
                    1024,
                )
                .await
                .expect("Failed to subscribe to mempool programs");

            let tx_clone = tx.clone();
            tokio::spawn(async move {
                while let Some(mempool_txs) = mempool_ch.recv().await {
                    for mempool_tx in mempool_txs {
                        if let Err(e) = tx_clone.send(mempool_tx).await {
                            eprintln!("Failed to send transaction: {}", e);
                            break;
                        }
                    }
                }
            });
        }

        rx
    }

    pub async fn listen_for_bundle_results(&self) -> mpsc::Receiver<BundleResult> {
        let (tx, rx) = mpsc::channel(1024);

        for searcher_client in self.searcher_clients.iter() {
            let mut bundle_results_receiver = searcher_client
                .subscribe_bundle_results(1024)
                .await
                .expect("Failed to subscribe to bundle results");

            let tx_clone = tx.clone();
            tokio::spawn(async move {
                while let Some(bundle_result) = bundle_results_receiver.recv().await {
                    if let Err(e) = tx_clone.send(bundle_result).await {
                        eprintln!("Failed to send bundle result (into our own channel): {}", e);
                        break;
                    }
                }
            });
        }

        rx
    }

    pub async fn broadcast_bundle_to_all_engines(
        &self,
        bundle_txs: Vec<VersionedTransaction>,
    ) -> Vec<JoinHandle<Result<BundleId, SearcherClientError>>> {
        self.searcher_clients
            .iter()
            .map(|client| {
                let client_clone = Arc::clone(client);
                let bundle_txs_clone = bundle_txs.clone();

                tokio::spawn(async move { client_clone.send_bundle(bundle_txs_clone).await })
            })
            .collect()
    }
}

pub fn graceful_panic(callback: Option<fn(&PanicInfo)>) -> Arc<AtomicBool> {
    let exit = Arc::new(AtomicBool::new(false));
    // Fail fast!
    let panic_hook = panic::take_hook();
    {
        let exit = exit.clone();
        panic::set_hook(Box::new(move |panic_info| {
            if let Some(f) = callback {
                f(panic_info);
            }
            exit.store(true, Ordering::Relaxed);
            println!("Disconnecting from JITO relayer...");
            // let other loops finish up
            std::thread::sleep(Duration::from_secs(5));
            // invoke the default handler and exit the process
            panic_hook(panic_info); // print the panic backtrace. default exit code is 101

            process::exit(1); // bail us out if thread blocks/refuses to join main thread
        }));
    }
    exit
}
