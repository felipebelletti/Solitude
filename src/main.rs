mod jito;
mod local_api;
mod openserum_api;
mod raydium_api;

use jito::{
    client_interceptor::ClientInterceptor, cluster_data_impl::ClusterDataImpl, grpc_connect,
    BundleId, SearcherClient, SearcherClientError, SearcherClientResult,
};
use jito_protos::{
    auth::auth_service_client::AuthServiceClient, bundle::Bundle,
    searcher::searcher_service_client::SearcherServiceClient,
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
use solana_program::{instruction::Instruction, system_instruction};
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
    process, result,
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
    // let binding = openserum_api::get_serum_token_data().await?;
    // let serum_token_data = match binding.first() {
    //     Some(data) => data,
    //     None => return Err("No openSerum data found".into()),
    // };

    // println!("{:?}", serum_token_data);

    let main_keypair = Arc::new(Keypair::from_bytes(&bs58::decode(
        "2zS4DvSbA6PdK4aokzG7dSbSMPvD93vb8gvH2J1Rg2RnSxXZddw7nksvfVi2F1BqGJufZjzk13tT3eiL8WM34EMP",
    )
    .into_vec()
    .unwrap()).unwrap());
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
    // let rpc_pubsub_addr = "http://127.0.0.1:8899/"; // CHANGE TO http://127.0.0.1:8899/
    let rpc_pubsub_addr = "https://api.mainnet-beta.solana.com/";

    let (mut searcher_client, _) = jito::get_searcher_client(
        &jito_auth_keypair,
        &graceful_panic(None),
        block_engine_url,
        rpc_pubsub_addr,
    )
    .await
    .expect("get_searcher_client failed");
    let rpc_client = RpcClient::new(rpc_pubsub_addr.to_string());

    let mut rng = thread_rng();
    let tip_program_pubkey: Pubkey = "T1pyyaTNZsKv2WcRAB8oVnk93mLJw2XzjtVYqCsaHqt"
        .parse()
        .unwrap();
    let tip_accounts = generate_tip_accounts(&tip_program_pubkey);
    let tip_account = tip_accounts[rng.gen_range(0..tip_accounts.len())];

    println!("Enter target address: ");
    let target_addr = read_pubkey_from_stdin().unwrap();
    // let paired_addr: Pubkey = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".parse().unwrap(); // USDT
    let paired_addr: Pubkey = "So11111111111111111111111111111111111111112"
        .parse()
        .unwrap(); // SOL
                   // let target_addr: Pubkey = "BdKpATfRZLDVEKiAgq2FggSESTdu3CjHbAqpYcca6rJH"
                   //     .parse()
                   //     .unwrap();
    let watch_mempool_addresses: Vec<Pubkey> = vec![target_addr];

    // let binding = raydium_api::get_pool_by_target(&target_addr.to_string()).await?;
    // let crafted_swap_data = match binding.first() {
    //     Some(data) => data,
    //     None => return Err("No Raydium data found".into()),
    // };
    let crafted_swap_data = local_api::get_raydium_crafted_swap(
        target_addr.to_string(),
        target_addr.to_string(),
        target_addr.to_string(),
        true,
    )
    .await?;

    let user_target_token_account =
        get_associated_token_address(&main_keypair.pubkey(), &target_addr);

    let mut instr_chain: Vec<Instruction> = vec![];
    let buy_amount = 0.001;

    let lamports_rent_exception = rpc_client
        .get_minimum_balance_for_rent_exemption(165)
        .await?;
    let seed = Alphanumeric.sample_string(&mut rand::thread_rng(), 32);
    let created_user_paired_account =
        &Pubkey::create_with_seed(&main_keypair.pubkey(), &seed, &spl_token::id())?;
    // #1
    let create_user_paired_account_instr = system_instruction::create_account_with_seed(
        &main_keypair.pubkey(),                                // source
        created_user_paired_account,                           // newAccount
        &main_keypair.pubkey(),                                // base
        &seed,                                                 // seed
        lamports_rent_exception + sol_to_lamports(buy_amount), // Lamports
        165,                                                   // Space
        &spl_token::id(),                                      // Owner
    );
    instr_chain.push(create_user_paired_account_instr);

    // #2
    let initialize_user_paired_account_instr = spl_token::instruction::initialize_account(
        &spl_token::id(),            // Token Program
        created_user_paired_account, // TokenAddress
        &paired_addr,                // InitAcount
        &main_keypair.pubkey(),      // Owner
    )?;
    instr_chain.push(initialize_user_paired_account_instr);

    let associated_account_exists: bool =
        match rpc_client.get_account(&user_target_token_account).await {
            Ok(account) => /* is_initialized_account(&account.data)*/ !account.data.is_empty(),
            Err(_) => false,
        };

    if !associated_account_exists
    {
        println!("Creating associated account");
        // 3
        let create_associated_account_instr =
            spl_associated_token_account::instruction::create_associated_token_account(
                &main_keypair.pubkey(),
                &main_keypair.pubkey(),
                &target_addr,
                &spl_token::id(),
            );
        instr_chain.push(create_associated_account_instr);
        println!(
            "0: {}, 1: {}, 2: {}, 3: {}",
            &main_keypair.pubkey(),
            &user_target_token_account,
            &target_addr,
            &spl_token::id()
        );
    };

    let swap_instr = amm_swap(
        &ammProgramID,
        &Pubkey::from_str(&crafted_swap_data.id)?,
        &Pubkey::from_str(&crafted_swap_data.authority)?,
        &Pubkey::from_str(&crafted_swap_data.open_orders)?,
        &Pubkey::from_str(&crafted_swap_data.target_orders)?,
        &Pubkey::from_str(&crafted_swap_data.base_vault)?,
        &Pubkey::from_str(&crafted_swap_data.quote_vault)?,
        &Pubkey::from_str(&crafted_swap_data.market_program_id)?,
        &Pubkey::from_str(&crafted_swap_data.market_id)?,
        &Pubkey::from_str(&crafted_swap_data.market_bids)?,
        &Pubkey::from_str(&crafted_swap_data.market_asks)?,
        &Pubkey::from_str(&crafted_swap_data.market_event_queue)?,
        &Pubkey::from_str(&crafted_swap_data.market_base_vault)?,
        &Pubkey::from_str(&crafted_swap_data.market_quote_vault)?,
        &Pubkey::from_str(&crafted_swap_data.market_authority)?,
        &created_user_paired_account,
        &user_target_token_account,
        &main_keypair.pubkey(),
        sol_to_lamports(buy_amount),
        1,
    )
    .expect("amm_swap failed");
    instr_chain.push(swap_instr);

    let close_user_paired_account_instr = spl_token::instruction::close_account(
        &spl_token::id(),            // Token Program
        created_user_paired_account, // Account
        &main_keypair.pubkey(),      // Destination
        &main_keypair.pubkey(),      // Owner
        &[],                         // MultiSigners
    )
    .expect("close_account failed");
    // instr_chain.push(close_user_paired_account_instr);

    instr_chain.push(transfer(
        &main_keypair.pubkey(),
        &tip_account,
        sol_to_lamports(0.04),
    ));

    let blockhash = rpc_client
        .get_latest_blockhash_with_commitment(CommitmentConfig {
            commitment: CommitmentLevel::Finalized,
        })
        .await?
        .0;
    let txn = VersionedTransaction::from(Transaction::new_signed_with_payer(
        &instr_chain,
        Some(&main_keypair.pubkey()),
        &[main_keypair.as_ref()],
        blockhash,
    ));

    // let mut interval = time::interval(Duration::from_millis(300));

    loop {
        let bundle_id = searcher_client
            .send_bundle(vec![txn.clone()], 3)
            .await
            .expect("send_bundle failed");
        println!("Bundle ID: {:?}", bundle_id);
    }

    // loop {
    //     let signature = rpc_client
    //         .send_and_confirm_transaction_with_spinner_and_config(
    //             &txn,
    //             CommitmentConfig::confirmed(),
    //             RpcSendTransactionConfig {
    //                 skip_preflight: true,
    //                 ..RpcSendTransactionConfig::default()
    //             },
    //         )
    //         .await?;
    //     println!("Tx Signature/Hash: {:?}", signature);
    // }

    // let mut mempool_ch = searcher_client
    //     .subscribe_mempool_programs(
    //         &watch_mempool_addresses,
    //         vec![
    //             "amsterdam".to_string(),
    //             "frankfurt".to_string(),
    //             "ny".to_string(),
    //             "tokyo".to_string(),
    //         ],
    //         100,
    //     )
    //     .await?;

    // println!("Listening for pending txs...");

    // let id = searcher_client.send_bundle(vec![backrun_tx], 4).await.unwrap();
    // println!("Bundle ID: {:?}", id);

    // let mut fire = true;
    // while let Some(txs) = mempool_ch.recv().await {
    //     for mempool_tx in txs {
    //         if fire == false {
    //             continue;
    //         }

    //         println!("Received transaction: {:?}", mempool_tx.signatures[0]);

    //         let blockhash = rpc_client
    //             .get_latest_blockhash_with_commitment(CommitmentConfig {
    //                 commitment: CommitmentLevel::Confirmed,
    //             })
    //             .await?
    //             .0;

    //         let backrun_tx = VersionedTransaction::from(Transaction::new_signed_with_payer(
    //             &[
    //                 build_memo(
    //                     format!("kpn: {:?}", mempool_tx.signatures[0].to_string()).as_bytes(),
    //                     &[],
    //                 ),
    //                 transfer(&main_keypair.pubkey(), &tip_account, sol_to_lamports(0.04)),
    //                 swap_instruction.clone(),
    //             ],
    //             Some(&main_keypair.pubkey()),
    //             &[main_keypair.as_ref()],
    //             blockhash,
    //         ));

    //         let txs: Vec<VersionedTransaction> = vec![mempool_tx, backrun_tx];

    //         let bundle_id = match searcher_client
    //             .send_bundle(txs, 3)
    //             .await {
    //                 Ok(bundle_id) => bundle_id,
    //                 Err(e) => {
    //                     println!("SendBundle Err: {:?}", e);
    //                     continue;
    //                 }
    //             };
    //         println!("Bundle ID: {:?}", bundle_id);
    //         // fire = false;
    //     }
    // }
    println!("Channel closed");

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
