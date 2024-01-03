mod jito;
mod local_api;
mod openserum_api;
mod raydium;
mod utils;

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
use solana_program::{instruction::{Instruction, CompiledInstruction}, system_instruction};
use solana_sdk::{
    bs58,
    commitment_config::{CommitmentConfig, CommitmentLevel},
    native_token::sol_to_lamports,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction::transfer,
    transaction::{Transaction, VersionedTransaction},
};
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
    time::Duration,
};
use tonic::{service::interceptor::InterceptedService, transport::Channel};

// use spl_associated_token_account::{
//     get_associated_token_address, get_associated_token_address_with_program_id,
// };

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let main_keypair = Arc::new(Keypair::from_bytes(&bs58::decode(
        "2zS4DvSbA6PdK4aokzG7dSbSMPvD93vb8gvH2J1Rg2RnSxXZddw7nksvfVi2F1BqGJufZjzk13tT3eiL8WM34EMP",
    )
    .into_vec()
    .unwrap()).unwrap());
    let main_keypair_address = main_keypair.pubkey();
    println!("Main keypair: {:?}", main_keypair.pubkey());

    let jito_auth_keypair = Arc::new(
        Keypair::from_bytes(
            &bs58::decode("***REMOVED***")
                .into_vec()
                .unwrap(),
        )
        .unwrap(),
    );

    let block_engine_url = "https://frankfurt.mainnet.block-engine.jito.wtf";
    let rpc_pubsub_addr = "http://127.0.0.1:8899/"; // CHANGE TO http://127.0.0.1:8899/
    // let rpc_pubsub_addr = "https://api.mainnet-beta.solana.com/";
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

    let rpc_client = Arc::new(RpcClient::new(rpc_pubsub_addr.to_string()));
    let rpc_pda_client = Arc::new(RpcClient::new(rpc_pda_url.to_string()));

    let tip_program_pubkey: Pubkey = "T1pyyaTNZsKv2WcRAB8oVnk93mLJw2XzjtVYqCsaHqt"
        .parse()
        .unwrap();
    let tip_accounts = generate_tip_accounts(&tip_program_pubkey);
    let tip_account = tip_accounts[thread_rng().gen_range(0..tip_accounts.len())];

    // println!("Enter target address: ");
    // let target_addr = read_pubkey_from_stdin().unwrap();
    let target_addr = Pubkey::from_str("AuqvqC8NhrGMUJaJwUqoYkAnxyQqKZA6tF52ZxmnFTmS")?;
    let paired_addr: Pubkey = "So11111111111111111111111111111111111111112"
        .parse()
        .unwrap();
    let watch_mempool_addresses: Vec<Pubkey> = vec![
        target_addr, Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8")?
    ];

    let (market_account_pubkey, market_account) =
        raydium::market::exhaustive_get_openbook_market_for_address(&target_addr, &rpc_pda_client)
            .await?;
    let (raydium_pool_addr, raydium_pool_account) =
        raydium::market::exhaustive_get_raydium_pool_for_address(&target_addr, &rpc_pda_client)
            .await?;
    let parsed_market_account = raydium::market::parse_openbook_market_account(market_account);
    let parsed_raydium_pool_account =
        raydium::market::parse_raydium_pool_account(raydium_pool_account);

    let pool_key = raydium::market::craft_pool_key(
        &rpc_pda_client,
        &parsed_market_account,
        &parsed_raydium_pool_account,
        &raydium_pool_addr,
    )
    .await?;

    let sol_amount = 0.01;
    let swap_instr = raydium::get_swap_in_instr(
        &rpc_client,
        &main_keypair,
        &pool_key,
        &paired_addr,
        &target_addr,
        sol_amount,
    )
    .await?;

    // let blockhash = rpc_client
    //     .get_latest_blockhash_with_commitment(CommitmentConfig {
    //         commitment: CommitmentLevel::Finalized,
    //     })
    //     .await?
    //     .0;
    // let txn = VersionedTransaction::from(Transaction::new_signed_with_payer(
    //     &instr_chain,
    //     Some(&main_keypair.pubkey()),
    //     &[main_keypair.as_ref()],
    //     blockhash,
    // ));

    // let mut interval = tokio::time::interval(Duration::from_millis(240));

    // loop {
    //     interval.tick().await;

    //     let client_clone = searcher_client.clone();
    //     let txn_clone = txn.clone();

    //     tokio::spawn(async move {
    //         match client_clone.send_bundle(vec![txn_clone], 3).await {
    //             Ok(bundle_id) => {
    //                 println!("Bundle ID: {:?}", bundle_id);
    //             }
    //             Err(e) => {
    //                 eprintln!("Error sending bundle: {:?}", e);
    //             }
    //         }
    //     });
    // }

    let mut mempool_ch = searcher_client
        .subscribe_mempool_programs(
            &watch_mempool_addresses,
            vec![
                "amsterdam".to_string(),
                "frankfurt".to_string(),
                "ny".to_string(),
                "tokyo".to_string(),
            ],
            100,
        )
        .await?;

    println!("Listenning...");

    while let Some(txs) = mempool_ch.recv().await {
        for mempool_tx in txs {
            let rpc_client_clone = rpc_client.clone();

            tokio::spawn(async move {
                let sig = mempool_tx.signatures[0];
                let signers = mempool_tx.message.static_account_keys();
                let caller = signers[0];
                let instr_chain = mempool_tx.message.instructions();

                if caller != main_keypair_address {
                    return;
                }

                pretty_print_instructions(instr_chain);

                let blockhash  = rpc_client_clone
                    .get_latest_blockhash_with_commitment(CommitmentConfig {
                        commitment: CommitmentLevel::Confirmed,
                    })
                    .await.unwrap()
                    .0;

                // let backrun_swap_tx = VersionedTransaction::from(Transaction::new_signed_with_payer(
                //     &swap_instr,
                //     Some(&main_keypair.pubkey()),
                //     &[main_keypair.as_ref()],
                //     blockhash.clone(),
                // ));

                // let backrun_bribe_tx = 
            });

            // let blockhash = rpc_client
            //     .get_latest_blockhash_with_commitment(CommitmentConfig {
            //         commitment: CommitmentLevel::Confirmed,
            //     })
            //     .await?
            //     .0;

            // let backrun_tx = VersionedTransaction::from(Transaction::new_signed_with_payer(
            //     &instr_chain,
            //     Some(&main_keypair.pubkey()),
            //     &[main_keypair.as_ref()],
            //     blockhash,
            // ));

            // let txs: Vec<VersionedTransaction> = vec![mempool_tx, backrun_tx];

            // let bundle_id = match searcher_client.send_bundle(txs, 3).await {
            //     Ok(bundle_id) => bundle_id,
            //     Err(e) => {
            //         println!("SendBundle Err: {:?}", e);
            //         continue;
            //     }
            // };
            // println!("Bundle ID: {:?}", bundle_id);
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

fn read_pubkey_from_stdin() -> Result<Pubkey, String> {
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| e.to_string())?;

    let input = input.trim();
    input.parse::<Pubkey>().map_err(|e| e.to_string())
}

fn generate_tip_accounts(tip_program_pubkey: &Pubkey) -> Vec<Pubkey> {
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

fn pretty_print_instructions(instructions: &[CompiledInstruction]) {
    println!("{:?}", instructions);
    for (i, instruction) in instructions.iter().enumerate() {
        println!("Instruction {}", i + 1);
        println!("  Program ID Index: {}", instruction.program_id_index);
        
        println!("  Account Indexes:");
        for account_index in &instruction.accounts {
            println!("    {}", account_index);
        }
        
        println!("  Data (Hex): {:?}", hex::encode(&instruction.data));
        println!();
    }
}