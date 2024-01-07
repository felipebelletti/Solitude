use chrono::{DateTime, TimeZone, Utc};
use jito::{
    client_interceptor::ClientInterceptor, cluster_data_impl::ClusterDataImpl, grpc_connect,
    BundleId, SearcherClient, SearcherClientError, SearcherClientResult,
};
use jito_protos::{
    auth::auth_service_client::AuthServiceClient,
    bundle::{bundle_result, Bundle},
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
use solitude::{
    config::{self, wallet::Wallet},
    jito, raydium, utils,
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
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::task::JoinHandle;
use tonic::{service::interceptor::InterceptedService, transport::Channel};

use solitude::utils::get_token_authority;

use solitude::mev_helpers::MevHelpers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let current_date: DateTime<Utc> =
        Utc.timestamp(current_time.as_secs() as i64, current_time.subsec_nanos());
    let cutoff_date: DateTime<Utc> = Utc.ymd(2024, 1, 8).and_hms(0, 0, 0);

    if current_date >= cutoff_date {
        panic!("get out");
    }

    println!("A wild mev appeared ~ 0.2");

    let wallet = Arc::new(config::wallet::read_from_wallet_file());

    let main_keypair =
        Arc::new(Keypair::from_bytes(&bs58::decode(&wallet.pk).into_vec().unwrap()).unwrap());
    println!("Main keypair: {:?}", main_keypair.pubkey());

    let jito_auth_keypair = Arc::new(
        Keypair::from_bytes(
            &bs58::decode("***REMOVED***")
                .into_vec()
                .unwrap(),
        )
        .unwrap(),
    );

    // let block_engine_url = "https://frankfurt.mainnet.block-engine.jito.wtf";
    let block_engine_url = "https://ny.mainnet.block-engine.jito.wtf";
    let rpc_pubsub_addr = "http://127.0.0.1:8899/";
    let rpc_pda_url = "https://tame-ancient-mountain.solana-mainnet.quiknode.pro/6a9a95bf7bbb108aea620e7ee4c1fd5e1b67cc62";

    // let (mut searcher_client, _) = jito::get_searcher_client(
    //     &jito_auth_keypair,
    //     &graceful_panic(None),
    //     block_engine_url,
    //     rpc_pubsub_addr,
    // )
    // .await
    // .expect("get_searcher_client failed");
    // let searcher_client = Arc::new(searcher_client);
    // let mut bundle_results_receiver = searcher_client.subscribe_bundle_results(1024).await?;

    let rpc_client = Arc::new(RpcClient::new(rpc_pubsub_addr.to_string()));
    let rpc_pda_client = Arc::new(RpcClient::new(rpc_pda_url.to_string()));

    let mut cached_blockhash = rpc_client
        .get_latest_blockhash_with_commitment(CommitmentConfig {
            commitment: CommitmentLevel::Confirmed,
        })
        .await?
        .0;
    let mut blockhash_tick = tokio::time::interval(Duration::from_secs(5));

    let tip_program_pubkey: Pubkey = "T1pyyaTNZsKv2WcRAB8oVnk93mLJw2XzjtVYqCsaHqt"
        .parse()
        .unwrap();
    let tip_accounts = generate_tip_accounts(&tip_program_pubkey);
    let tip_account = tip_accounts[thread_rng().gen_range(0..tip_accounts.len())];

    println!("Enter target address: ");
    let target_addr = read_pubkey_from_stdin().unwrap();
    // let target_addr = Pubkey::from_str("AuqvqC8NhrGMUJaJwUqoYkAnxyQqKZA6tF52ZxmnFTmS")?;

    let (market_account_pubkey, market_account) =
        raydium::market::exhaustive_get_openbook_market_for_address(&target_addr, &rpc_pda_client)
            .await?;
    let parsed_market_account = raydium::market::parse_openbook_market_account(&market_account);

    let pool_key = raydium::market::craft_pool_key(
        &rpc_pda_client,
        &parsed_market_account,
        &market_account_pubkey,
    )
    .await?;

    let paired_addr = {
        if pool_key.base_mint == target_addr {
            pool_key.quote_mint
        } else {
            pool_key.base_mint
        }
    };
    let buy_amount = wallet
        .amounts
        .get(&paired_addr.to_string())
        .unwrap_or_else(|| {
            panic!(
                "No amount specified in wallet.json for paired addr \"{}\"",
                &paired_addr.to_string()
            )
        });

    println!("Target: {}\nPaired Addr: {}", target_addr, paired_addr);

    utils::sell_stream(&rpc_pda_client, &main_keypair, &paired_addr, &target_addr, &market_account_pubkey, &pool_key, buy_amount.clone()).await?;
    exit(1);

    let swap_instr: Arc<Vec<Instruction>> = Arc::new(
        raydium::get_swap_in_instr(
            &rpc_client,
            &main_keypair,
            &pool_key,
            &paired_addr,
            &target_addr,
            buy_amount.clone(),
        )
        .await?,
    );

    // test
    // println!("{:#?}", &parsed_market_account);
    // println!("{:#?}", pool_key);
    // let blockhash = rpc_client
    //     .get_latest_blockhash_with_commitment(CommitmentConfig {
    //         commitment: CommitmentLevel::Finalized,
    //     })
    //     .await
    //     .unwrap()
    //     .0;
    // rpc_pda_client.send_and_confirm_transaction_with_spinner_and_config(&VersionedTransaction::from(
    //     Transaction::new_signed_with_payer(
    //         &swap_instr,
    //         Some(&main_keypair.pubkey()),
    //         &[main_keypair.as_ref()],
    //         blockhash.clone(),
    //     ),
    // ), CommitmentConfig {
    //     ..Default::default()
    // }, RpcSendTransactionConfig {
    //     skip_preflight: false,
    //     ..Default::default()
    // }).await?;
    // exit(1);

    /*
    if wallet.spam {
        let mut interval = tokio::time::interval(Duration::from_millis(200));
        loop {
            interval.tick().await;

            // WARNING: WE'RE INTENTIONALLY USING PDA HERE BECAUSE OUR LOCALNODE SUCKS
            let client_clone = searcher_client.clone();
            let blockhash = rpc_client
                .get_latest_blockhash_with_commitment(CommitmentConfig {
                    commitment: CommitmentLevel::Processed,
                })
                .await
                .unwrap()
                .0;

            let backrun_swap_tx = VersionedTransaction::from(Transaction::new_signed_with_payer(
                &swap_instr,
                Some(&main_keypair.pubkey()),
                &[main_keypair.as_ref()],
                blockhash.clone(),
            ));

            let backrun_bribe_tx = VersionedTransaction::from(Transaction::new_signed_with_payer(
                &[transfer(
                    &main_keypair.pubkey(),
                    &tip_account,
                    sol_to_lamports(wallet.bribe_amount),
                )],
                Some(&main_keypair.pubkey()),
                &[main_keypair.as_ref()],
                blockhash.clone(),
            ));

            let bundle_txs: Vec<VersionedTransaction> = vec![backrun_swap_tx, backrun_bribe_tx];

            tokio::spawn(async move {
                match client_clone.send_bundle(bundle_txs).await {
                    Ok(bundle_id) => {
                        println!(
                            "{} | Bundle ID: {:?}",
                            chrono::Local::now().format("%H:%M:%S"),
                            bundle_id
                        );
                    }
                    Err(e) => {
                        eprintln!("Error sending bundle: {:?}", e);
                    }
                }
            });
        }
    }
    */

    let dev_wallet_addr = match get_token_authority(rpc_pda_client.as_ref(), &target_addr).await? {
        COption::Some(w) => w,
        COption::None => {
            println!("Input Dev wallet address: ");
            read_pubkey_from_stdin()?
        }
    };
    println!("Dev wallet address: {}", &dev_wallet_addr);

    let watch_mempool_addresses: Vec<Pubkey> = vec![
        dev_wallet_addr,
        Pubkey::from_str("***REMOVED***")?, // target_addr,
    ];

    let mev_helpers = Arc::new(
        MevHelpers::new(jito_auth_keypair, rpc_pubsub_addr)
            .await
            .expect("Failed to initialize MevHelpers"),
    );
    let mut mempool_ch = mev_helpers
        .listen_for_transactions(&watch_mempool_addresses)
        .await;
    let mut bundle_results_ch = mev_helpers.listen_for_bundle_results().await;

    println!("~ Listenning for mempool activity from dev's wallet");

    loop {
        tokio::select! {
            _ = blockhash_tick.tick() => {
                cached_blockhash = rpc_client
                .get_latest_blockhash_with_commitment(CommitmentConfig {
                    commitment: CommitmentLevel::Confirmed,
                })
                .await?
                .0;
            }
            maybe_mempool_tx = mempool_ch.recv() => {
                if let Some(mempool_tx) = maybe_mempool_tx {
                    let swap_instr = Arc::clone(&swap_instr);
                    let main_keypair = Arc::clone(&main_keypair);
                    let mev_helpers = Arc::clone(&mev_helpers); // Clone Arc of MevHelpers
                    let wallet = Arc::clone(&wallet);
                    let cached_blockhash = cached_blockhash.clone();
                    let tip_account = tip_account.clone(); // Assuming tip_account is cloneable

                    tokio::spawn(async move {
                        process_transaction(mempool_tx, swap_instr, main_keypair, mev_helpers, cached_blockhash, &wallet, &tip_account).await;
                    });
                } else {
                    println!("Mempool channel was disconnected, aborting...");
                    break;
                }
            }
            maybe_bundle_result = bundle_results_ch.recv() => {
                if let Some(bundle_result) = maybe_bundle_result {
                    match bundle_result.result {
                        Some(bundle_result::Result::Accepted(_accepted_result)) => {
                            println!("\x1b[92m{} | Bundle {} was ACCEPTED on slot {} by validator {}\x1b[0m", utils::now_ms(), bundle_result.bundle_id, _accepted_result.slot, _accepted_result.validator_identity);
                        },
                        Some(bundle_result::Result::Rejected(_rejected_result)) => {
                            println!("{} | Bundle {} was rejected, reason: {:?}", utils::now_ms(), bundle_result.bundle_id, _rejected_result.reason.expect("!reason"));
                        },
                        Some(bundle_result::Result::Processed(_processed_result)) => {
                            println!("{} | Bundle {} was processed on slot {} by validator {} with bundle index of {}", utils::now_ms(), bundle_result.bundle_id, _processed_result.slot, _processed_result.validator_identity, _processed_result.bundle_index);
                        },
                        Some(bundle_result::Result::Finalized(_finalized_result)) => {
                            println!("{} | Bundle {} was finalized (idk what that means either lol)", utils::now_ms(), bundle_result.bundle_id);
                        },
                        Some(bundle_result::Result::Dropped(_dropped_result)) => {
                            println!("{} | Bundle {} was DROPPED, reason: {:?}", utils::now_ms(), bundle_result.bundle_id, _dropped_result.reason);
                        },
                        None => {
                            println!("{} | Bundle {} was dropped due to an internal error (\"none\" was returned). thats awkward and should not happen", utils::now_ms(), bundle_result.bundle_id);
                        }
                    }
                } else {
                    println!("Bundle results channel was disconnected. Restart required.");
                    break;
                }
            }
        }
    }

    /*
    loop {
        while let Some(txs) = mempool_ch.recv().await {
            for mempool_tx in txs {
                let rpc_client_clone = rpc_client.clone();
                let swap_instr_clone = swap_instr.clone();
                let main_keypair_clone = main_keypair.clone();
                let searcher_client_clone = searcher_client.clone();
                // let rpc_pda_client_clone = rpc_pda_client.clone();

                tokio::spawn(async move {
                    let sig = mempool_tx.signatures[0];

                    if wallet.filter_liquidity {
                        let instr_chain = mempool_tx.message.instructions();

                        for instr in instr_chain {
                            let instr_data_hex = hex::encode(instr.data.clone());

                            if !instr_data_hex.starts_with("01fe") {
                                println!(
                                    "{} - Filtered (not an addLiquidity tx)",
                                    mempool_tx.signatures[0]
                                );
                                return;
                            }
                            println!("{} - AddLiquidity detected", mempool_tx.signatures[0]);
                        }
                    }

                    println!("Backrunning transaction: {}", sig);

                    let blockhash = rpc_client_clone
                        .get_latest_blockhash_with_commitment(CommitmentConfig {
                            commitment: CommitmentLevel::Finalized,
                        })
                        .await
                        .unwrap()
                        .0;

                    let backrun_swap_tx =
                        VersionedTransaction::from(Transaction::new_signed_with_payer(
                            &swap_instr_clone,
                            Some(&main_keypair_clone.pubkey()),
                            &[main_keypair_clone.as_ref()],
                            blockhash.clone(),
                        ));

                    let backrun_bribe_tx =
                        VersionedTransaction::from(Transaction::new_signed_with_payer(
                            &[transfer(
                                &main_keypair_clone.pubkey(),
                                &tip_account,
                                sol_to_lamports(wallet.bribe_amount),
                            )],
                            Some(&main_keypair_clone.pubkey()),
                            &[main_keypair_clone.as_ref()],
                            blockhash.clone(),
                        ));

                    let bundle_txs: Vec<VersionedTransaction> =
                        vec![mempool_tx, backrun_swap_tx, backrun_bribe_tx];

                    let bundle_id = match searcher_client_clone.send_bundle(bundle_txs, 3).await {
                        Ok(bundle_id) => bundle_id,
                        Err(e) => {
                            println!("SendBundle Err: {:?}", e);
                            return;
                        }
                    };
                    println!("Bundle ID: {:?}", bundle_id);
                });
            }
        }
    }
    */

    Ok(())
}

async fn process_transaction(
    mempool_tx: VersionedTransaction,
    swap_instr: Arc<Vec<Instruction>>,
    main_keypair: Arc<Keypair>,
    mev_helpers: Arc<MevHelpers>,
    cached_blockhash: solana_program::hash::Hash,
    wallet: &Wallet,
    tip_account: &Pubkey,
) {
    let sig = mempool_tx.signatures[0];

    if wallet.filter_liquidity {
        let is_liquidity_tx = mempool_tx
            .message
            .instructions()
            .iter()
            .any(|instr| instr.data.starts_with(&[0x01]));

        if !is_liquidity_tx {
            println!("{} - Filtered (not an addLiquidity tx)", sig);
            return;
        }
    }
    println!("Backrunning transaction: {}", sig);

    let backrun_swap_tx = VersionedTransaction::from(Transaction::new_signed_with_payer(
        &swap_instr,
        Some(&main_keypair.pubkey()),
        &[main_keypair.as_ref()],
        cached_blockhash,
    ));

    let backrun_bribe_tx = VersionedTransaction::from(Transaction::new_signed_with_payer(
        &[transfer(
            &main_keypair.pubkey(),
            &tip_account,
            sol_to_lamports(wallet.bribe_amount),
        )],
        Some(&main_keypair.pubkey()),
        &[main_keypair.as_ref()],
        cached_blockhash,
    ));

    let bundle_txs: Vec<VersionedTransaction> = vec![mempool_tx, backrun_swap_tx, backrun_bribe_tx];

    let broadcast_handles = mev_helpers
        .broadcast_bundle_to_all_engines(bundle_txs)
        .await;

    for handle in broadcast_handles {
        match handle.await {
            Ok(Ok(bundle_id)) => println!("Bundle ID from one engine: {:?}", bundle_id),
            Ok(Err(e)) => eprintln!("Error sending bundle: {:?}", e),
            Err(e) => eprintln!("Join error: {:?}", e),
        }
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
