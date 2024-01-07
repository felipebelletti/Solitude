use colored::*;
use solana_program::native_token::lamports_to_sol;
use std::error::Error;
use std::str::FromStr;
use std::sync::Arc;
use std::thread;

use chrono::format::DelayedFormat;
use chrono::format::StrftimeItems;
use jito_protos::auth;
use raydium_amm::instruction::simulate_get_pool_info;
use raydium_amm::instruction::SimulateInstruction;
use raydium_amm::instruction::SwapInstructionBaseIn;
use raydium_amm::instruction::SwapInstructionBaseOut;
use raydium_amm::processor::Processor;
use raydium_amm::state::GetPoolData;
use raydium_amm::state::GetSwapBaseInData;
use raydium_amm::state::SimulateParams;
use solana_account_decoder::UiAccountData;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_request::TokenAccountsFilter};
use solana_program::account_info::AccountInfo;
use solana_program::instruction::Instruction;
use solana_program::native_token::sol_to_lamports;
use solana_program::{program_option::COption, program_pack::Pack, pubkey::Pubkey};
use solana_sdk::account::create_is_signer_account_infos;
use solana_sdk::account::Account;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::commitment_config::CommitmentLevel;
use solana_sdk::transaction::Transaction;
use solana_sdk::transaction::VersionedTransaction;
use solana_sdk::{signature::Keypair, signer::Signer};
use spl_associated_token_account::get_associated_token_address;
use spl_token::state::Account as TokenAccount;
use spl_token::state::Mint;
use tokio::sync::mpsc::UnboundedSender;

use crate::raydium::utils::get_associated_lp_mint;

use crate::raydium;
use crate::raydium::market::PoolKey;
use tokio::time::{self, Duration};

use crossterm::{
    event::{self, Event as CEvent, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use device_query::{DeviceQuery, DeviceState, Keycode};
use std::io::{self, Write};

pub async fn get_token_decimals(
    client: &RpcClient,
    token_mint_address: &Pubkey,
) -> Result<u8, Box<dyn Error>> {
    let account = client.get_account(token_mint_address).await?;
    let mint_data = spl_token::state::Mint::unpack(&account.data)?;

    Ok(mint_data.decimals)
}

pub async fn get_token_authority(
    client: &RpcClient,
    token_mint_address: &Pubkey,
) -> Result<COption<Pubkey>, Box<dyn Error>> {
    let account_data = client.get_account_data(&token_mint_address).await?;
    let mint = Mint::unpack(&account_data)?;

    Ok(mint.mint_authority)
}

pub async fn sell_stream(
    client: &RpcClient,
    bought_wallet: &Keypair,
    paired_token_addr: &Pubkey,
    target_token_addr: &Pubkey,
    market_account: &Pubkey,
    pool_key: &PoolKey,
    token_bag_cost: f64,
) -> Result<(), Box<dyn Error>> {
    let bought_wallet_address = &bought_wallet.pubkey();

    let binding = client
        .get_token_accounts_by_owner(
            bought_wallet_address,
            TokenAccountsFilter::Mint(*target_token_addr),
        )
        .await?;
    let token_account = binding.first().unwrap();
    let token_account_addr = { Pubkey::from_str(&token_account.pubkey)? };

    let lp_mint_addr = get_associated_lp_mint(
        &raydium_contract_instructions::amm_instruction::ID,
        &pool_key.market_id,
    )?;

    let paired_token_token_account =
        get_associated_token_address(&bought_wallet.pubkey(), &paired_token_addr);

    let target_token_token_account =
        get_associated_token_address(&bought_wallet.pubkey(), &target_token_addr);

    let mut show_profit_tick = time::interval(Duration::from_millis(700));

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(async move {
        keyboard_task(tx).await;
    });

    let mut token_balance: u64 = client
        .get_token_account_balance(&token_account_addr)
        .await?
        .amount
        .parse()?;

    loop {
        tokio::select! {
                    _ = show_profit_tick.tick() => {
                        token_balance = client
                            .get_token_account_balance(&token_account_addr)
                            .await?
                            .amount
                            .parse()?;

                        let simulated_swap_data = simulate_swap(
                            client,
                            pool_key,
                            market_account,
                            paired_token_addr,
                            &target_token_token_account,
                            &paired_token_token_account,
                            bought_wallet_address,
                            token_balance,
                        )
                        .await?;
                        let current_bag_value = lamports_to_sol(simulated_swap_data.minimum_amount_out);

                        let profit_percentage = ((current_bag_value / token_bag_cost) - 1.00) * 100.00;

                        let profit_color = if profit_percentage >= 500.0 {
                            "yellow" // Huge profits
                        } else if profit_percentage > 200.0 {
                            "blue" // Good profits
                        } else if profit_percentage > 0.0 {
                            "green" // Low profits
                        } else {
                            "red" // Loss
                        };

                        let profit_str = format!("{:.2}%", profit_percentage).color(profit_color).bold();

                        let print_data = format!(
                            "\r\n\x1B[2K{}\nTokens: {} | Worth: {} SOL | Price Impact: {}% | Profit: {}",
                            format!("--------- {} ---------", now_ms()).cyan().bold(),
                            token_balance.to_string().purple(),
                            format!("{:.2}", simulated_swap_data.minimum_amount_out).green().bold(),
                            format!("{:.2}", simulated_swap_data.price_impact).blue(),
                            profit_str
                        );
                        
                        disable_raw_mode().expect("Failed to disable raw mode");
                        println!("{}", print_data);
                        enable_raw_mode().expect("Failed to enable raw mode");
                        // io::stdout().flush().unwrap();
                    },
                    Some(key_event) = rx.recv() => {
                        match key_event.code {
                            KeyCode::Char('q') if key_event.modifiers == KeyModifiers::CONTROL => {
                                let swap_instr: Arc<Vec<Instruction>> = Arc::new(
                                    raydium::get_swap_out_instr(
                                        client,
                                        &bought_wallet,
                                        &pool_key,
                                        &paired_token_addr,
                                        &target_token_addr,
                                        token_balance,
                                    )
                                    .await?,
                                );
                            },
                            KeyCode::Char('c') if key_event.modifiers == KeyModifiers::CONTROL => {
                                // let mut stdout = io::stdout();

                                // execute!(stdout, LeaveAlternateScreen).expect("Failed to leave alternate screen");
                                disable_raw_mode().expect("Failed to disable raw mode");
                                break;
                            },
                            _ => {},
                        }
                    },
                }
    }

    // let blockhash = client
    //     .get_latest_blockhash_with_commitment(CommitmentConfig {
    //         commitment: CommitmentLevel::Finalized,
    //     })
    //     .await
    //     .unwrap()
    //     .0;
    // client.send_and_confirm_transaction_with_spinner_and_config(&VersionedTransaction::from(
    //     Transaction::new_signed_with_payer(
    //         &swap_instr,
    //         Some(&bought_wallet.pubkey()),
    //         &[bought_wallet],
    //         blockhash.clone(),
    //     ),
    // ), CommitmentConfig {
    //     ..Default::default()
    // }, RpcSendTransactionConfig {
    //     skip_preflight: true,
    //     ..Default::default()
    // }).await?;

    Ok(())
}

// needs refactor omfg
async fn simulate_swap(
    client: &RpcClient,
    pool_key: &PoolKey,
    market_account: &Pubkey,
    paired_token_addr: &Pubkey,
    target_token_token_account: &Pubkey,
    paired_token_token_account: &Pubkey,
    bought_wallet_address: &Pubkey,
    token_balance: u64,
) -> Result<GetSwapBaseInData, Box<dyn Error>> {
    let mut amm_account = client.get_account(&pool_key.id).await?;
    let mut amm_authority_account = client.get_account(&pool_key.authority).await?;
    let mut open_orders_account = client.get_account(&pool_key.open_orders).await?;
    let mut target_orders_account = client.get_account(&pool_key.target_orders).await?;
    let mut coin_vault_account = client.get_account(&pool_key.base_vault).await?;
    let mut pc_vault_account = client.get_account(&pool_key.quote_vault).await?;
    let mut lp_mint_account = client.get_account(&pool_key.lp_mint).await?;

    let mut market_program_account = client.get_account(&pool_key.market_program_id).await?;
    let mut market_info_account = client.get_account(&market_account).await?;
    let mut market_event_queue_account = client.get_account(&pool_key.market_event_queue).await?;

    let mut user_source_account = client.get_account(&target_token_token_account).await?;
    let mut user_dest_account = Account::new(1, 165, &spl_token::id());

    user_dest_account.data = {
        let forged_spl_dest_account = spl_token::state::Account {
            mint: paired_token_addr.clone(),
            owner: bought_wallet_address.clone(),
            state: spl_token::state::AccountState::Initialized,
            ..Default::default()
        };
        let mut data = vec![0u8; spl_token::state::Account::LEN];
        spl_token::state::Account::pack(forged_spl_dest_account, &mut data)?;
        data
    };

    let mut user_source_owner_account = client.get_account(&bought_wallet_address).await?;

    let mut accounts = vec![
        (&pool_key.id, false, &mut amm_account),
        (&pool_key.authority, false, &mut amm_authority_account),
        (&pool_key.open_orders, false, &mut open_orders_account),
        (&pool_key.target_orders, false, &mut target_orders_account),
        (&pool_key.base_vault, false, &mut coin_vault_account),
        (&pool_key.quote_vault, false, &mut pc_vault_account),
        (&pool_key.lp_mint, false, &mut lp_mint_account),
        (
            &pool_key.market_program_id,
            false,
            &mut market_program_account,
        ),
        (&market_account, false, &mut market_info_account),
        (
            &pool_key.market_event_queue,
            false,
            &mut market_event_queue_account,
        ),
        (&target_token_token_account, false, &mut user_source_account),
        (&paired_token_token_account, false, &mut user_dest_account),
        (&bought_wallet_address, true, &mut user_source_owner_account),
    ];
    let accounts_slice: &mut [(&Pubkey, bool, &mut solana_sdk::account::Account)] =
        accounts.as_mut_slice();

    let account_infos = create_is_signer_account_infos(accounts_slice);

    let simulated_swap_data = Processor::simulate_swap_base_in(
        &raydium_contract_instructions::amm_instruction::ID,
        &account_infos,
        SimulateInstruction {
            param: 1,
            swap_base_in_value: Some(SwapInstructionBaseIn {
                amount_in: token_balance, /* .amount.parse()? */
                minimum_amount_out: 0,
                ..Default::default()
            }),
            ..Default::default()
        },
    )?;

    Ok(simulated_swap_data)
}

async fn keyboard_task(tx: tokio::sync::mpsc::UnboundedSender<KeyEvent>) {
    enable_raw_mode().expect("Failed to enable raw mode");
    // let mut stdout = io::stdout();
    // execute!(stdout, EnterAlternateScreen).expect("Failed to enter alternate screen");

    loop {
        if event::poll(Duration::from_millis(200)).expect("Failed to poll event") {
            if let CEvent::Key(key_event) = event::read().expect("Failed to read event") {
                tx.send(key_event).unwrap();
            }
        }
    }

    // execute!(stdout, LeaveAlternateScreen).expect("Failed to leave alternate screen");
    disable_raw_mode().expect("Failed to disable raw mode");
}

pub fn now_ms() -> DelayedFormat<StrftimeItems<'static>> {
    chrono::Local::now().format("%H:%M:%S%.3f")
}

/*
schema to get pool open time (its within pool_info data)
let mut amm_account = client.get_account(&pool_key.id).await?;
    let mut amm_authority_account = client.get_account(&pool_key.authority).await?;
    let mut open_orders_account = client.get_account(&pool_key.open_orders).await?;
    let mut coin_vault_account = client.get_account(&pool_key.base_vault).await?;
    let mut pc_vault_account = client.get_account(&pool_key.quote_vault).await?;
    let mut lp_mint_account = client.get_account(&lp_mint_addr).await?;
    let mut market_info_account = client.get_account(&market_account_pubkey).await?;
    let mut market_event_queue_account = client.get_account(&pool_key.market_event_queue).await?;

    let mut accounts = vec![
        (&pool_key.id, false, &mut amm_account),
        (&pool_key.authority, false, &mut amm_authority_account),
        (&pool_key.open_orders, false, &mut open_orders_account),
        (&pool_key.base_vault, false, &mut coin_vault_account),
        (&pool_key.quote_vault, false, &mut pc_vault_account),
        (&lp_mint_addr, false, &mut lp_mint_account),
        (&market_account_pubkey, false, &mut market_info_account),
        (&pool_key.market_event_queue, false, &mut market_event_queue_account)
    ];
    let accounts_slice: &mut [(&Pubkey, bool, &mut solana_sdk::account::Account)] = accounts.as_mut_slice();

    let account_infos = create_is_signer_account_infos(accounts_slice);

    let pool_info = Processor::process_simulate_info(
        &raydium_contract_instructions::amm_instruction::ID,
        &account_infos,
        SimulateInstruction {
            param: 0,
            ..Default::default()
        },
    )?;

    println!("{:#?}", pool_info);

    GetPoolData: {"status":6,"coin_decimals":9,"pc_decimals":9,"lp_decimals":9,"pool_pc_amount":1827059158131,"pool_coin_amount":223374915011324050,"pnl_pc_amount":0,"pnl_coin_amount":0,"pool_lp_supply":342976073995411,"pool_open_time":1703797327,"amm_id":"9Rc5LrMNdjxePyd7xjZiSTAJURpzoi6GjiCPqnxQopdD"}
    */
