use chrono::{DateTime, TimeZone, Utc};
use jito::{
    client_interceptor::ClientInterceptor, cluster_data_impl::ClusterDataImpl, grpc_connect,
    BundleId, SearcherClient, SearcherClientError, SearcherClientResult,
};
use jito_protos::{
    auth::auth_service_client::AuthServiceClient,
    bundle::Bundle,
    searcher::{self, searcher_service_client::SearcherServiceClient},
};
use rand::{
    distributions::{Alphanumeric, DistString},
    thread_rng, Rng,
};
use raydium_amm::instruction::{self, simulate_swap_base_in};
use raydium_contract_instructions::{
    amm_instruction::{swap_base_in as amm_swap, ID as ammProgramID},
    stable_instruction::{swap_base_in as stable_swap, ID as stableProgramID},
};
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_config::RpcSendTransactionConfig};
use solana_program::{
    instruction::{CompiledInstruction, Instruction},
    program_option::COption,
    system_instruction,
};
use solana_sdk::{
    bs58,
    commitment_config::{CommitmentConfig, CommitmentLevel},
    native_token::sol_to_lamports,
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    system_instruction::transfer,
    transaction::{Transaction, VersionedTransaction},
};
use solana_transaction_status::UiTransactionEncoding;
use solitude::{config, jito, raydium};
use spl_associated_token_account::get_associated_token_address;
use spl_memo::build_memo;
use spl_token::state::is_initialized_account;
use std::{
    error::Error,
    io::{self, Write},
    panic::{self, PanicInfo},
    process::{self, exit},
    result,
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tonic::{service::interceptor::InterceptedService, transport::Channel};

use solitude::utils::get_token_authority;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let jito_auth_keypair = Arc::new(
        Keypair::from_bytes(
            &bs58::decode("***REMOVED***")
                .into_vec()
                .unwrap(),
        )
        .unwrap(),
    );

    let block_engine_url = "https://frankfurt.mainnet.block-engine.jito.wtf";
    let rpc_pubsub_addr = "http://127.0.0.1:8899/";
    let rpc_pda_url = "https://tame-ancient-mountain.solana-mainnet.quiknode.pro/6a9a95bf7bbb108aea620e7ee4c1fd5e1b67cc62";

    let (mut searcher_client, _) = jito::get_searcher_client(
        &jito_auth_keypair,
        &graceful_panic(None),
        block_engine_url,
        rpc_pubsub_addr,
    )
    .await
    .expect("get_searcher_client failed");
    let searcher_client = Arc::new(searcher_client);

    // TODO: read from tracker-wallets.jsonl
    let watch_mempool_addresses: Vec<Pubkey> = vec![Pubkey::from_str(
        "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8",
    )?];

    let mut mempool_ch = searcher_client
        .subscribe_mempool_programs(
            &watch_mempool_addresses,
            vec![
                "amsterdam".to_string(),
                "frankfurt".to_string(),
                "ny".to_string(),
                "tokyo".to_string(),
            ],
            1024,
        )
        .await?;

    let methods: Arc<Vec<String>> = Arc::new(vec![
        "09506ede560cba03010000000000000000".to_string(), // ? swap
        "0b00bca0650100000000000c3d5d53aa01".to_string(), // ? swap
        "09acf3f4509e1c00000000000000000000".to_string(), // dragonfly
    ]);
    loop {
        while let Some(txs) = mempool_ch.recv().await {
            for mempool_tx in txs {
                let methods_clone = methods.clone();

                tokio::spawn(async move {
                    let sig = mempool_tx.signatures[0];
                    let signer = mempool_tx.message.static_account_keys()[0];
                    let instr_chain = mempool_tx.message.instructions();

                    // println!("{} - {}", sig, hex::encode(instr_chain[0].data.clone()));

                    for instr in instr_chain {
                        let instr_data_hex = hex::encode(instr.data.clone());

                        if methods_clone.contains(&instr_data_hex) {
                            println!("SNIPING");
                            println!("Signature: {}", sig);
                            println!("Signer: {}", signer);
                            println!("Instruction data (id. hex): {:?}", instr.data);
                            println!("Input Accounts: {:?}", instr.accounts);
                        }

                        if instr_data_hex.starts_with("01fe") {
                            println!("ADD LIQUITY");
                            println!("Signature: {}", sig);
                            println!("Signer: {}", signer);
                            println!("Instruction data (id. hex): {:?}", instr.data);
                            println!("Input Accounts: {:?}", instr.accounts);
                        }
                    }
                });
            }
        }
    }

    Ok(())
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
            println!("exiting process");
            // let other loops finish up
            std::thread::sleep(Duration::from_secs(5));
            // invoke the default handler and exit the process
            panic_hook(panic_info); // print the panic backtrace. default exit code is 101

            process::exit(1); // bail us out if thread blocks/refuses to join main thread
        }));
    }
    exit
}
