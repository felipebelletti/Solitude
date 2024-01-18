use std::{error::Error, str::FromStr};

use rand::distributions::{Alphanumeric, DistString};
use raydium_amm::instruction::{AmmInstruction, SwapInstructionBaseIn};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    system_instruction, system_program,
};
use solana_sdk::{
    account::ReadableAccount,
    compute_budget::ComputeBudgetInstruction,
    feature_set::add_set_compute_unit_price_ix,
    native_token::sol_to_lamports,
    pubkey::Pubkey,
    signature::{Keypair, Signer}, signer,
};
use solana_transaction_status::parse_associated_token::spl_associated_token_id;
use spl_associated_token_account::get_associated_token_address;

use crate::config::wallet::Wallet;

use self::market::PoolKey;

pub mod market;
pub mod public_api;
pub mod utils;

pub async fn get_swap_in_instr(
    rpc_client: &RpcClient,
    signer_keypair: &Keypair,
    pool_key: &PoolKey,
    paired_addr: &Pubkey,
    token_addr: &Pubkey,
    sol_amount: f64,
) -> Result<Vec<Instruction>, Box<dyn Error>> {
    let user_target_token_account =
        get_associated_token_address(&signer_keypair.pubkey(), &token_addr);

    let mut instr_chain: Vec<Instruction> = vec![];

    let lamports_rent_exception = rpc_client
        .get_minimum_balance_for_rent_exemption(165)
        .await?;
    let seed = Alphanumeric.sample_string(&mut rand::thread_rng(), 32);
    let created_user_paired_account =
        &Pubkey::create_with_seed(&signer_keypair.pubkey(), &seed, &spl_token::id())?;
    // #1
    let create_user_paired_account_instr = system_instruction::create_account_with_seed(
        &signer_keypair.pubkey(),                              // source
        created_user_paired_account,                           // newAccount
        &signer_keypair.pubkey(),                              // base
        &seed,                                                 // seed
        lamports_rent_exception + sol_to_lamports(sol_amount), // Lamports
        165,                                                   // Space
        &spl_token::id(),                                      // Owner
    );
    instr_chain.push(create_user_paired_account_instr);

    // #2
    let initialize_user_paired_account_instr = spl_token::instruction::initialize_account(
        &spl_token::id(),            // Token Program
        created_user_paired_account, // TokenAddress
        &paired_addr,                // InitAcount
        &signer_keypair.pubkey(),    // Owner
    )?;
    instr_chain.push(initialize_user_paired_account_instr);

    let associated_account_exists: bool =
        match rpc_client.get_account(&user_target_token_account).await {
            Ok(account) =>
            /* is_initialized_account(&account.data)*/
            {
                !account.data.is_empty()
            }
            Err(_) => false,
        };

    if !associated_account_exists {
        println!("Creating associated account");
        // 3
        let create_associated_account_instr =
            spl_associated_token_account::instruction::create_associated_token_account(
                &signer_keypair.pubkey(),
                &signer_keypair.pubkey(),
                &token_addr,
                &spl_token::id(),
            );
        instr_chain.push(create_associated_account_instr);
    };

    let swap_instr = raydium_contract_instructions::amm_instruction::swap_base_in(
        &raydium_contract_instructions::amm_instruction::ID,
        &pool_key.id,
        &pool_key.authority,
        &pool_key.open_orders,
        &pool_key.target_orders,
        &pool_key.base_vault,
        &pool_key.quote_vault,
        &pool_key.market_program_id,
        &pool_key.market_id,
        &pool_key.market_bids,
        &pool_key.market_asks,
        &pool_key.market_event_queue,
        &pool_key.market_base_vault,
        &pool_key.market_quote_vault,
        &pool_key.market_authority,
        &created_user_paired_account,
        &user_target_token_account,
        &signer_keypair.pubkey(),
        sol_to_lamports(sol_amount),
        0,
    )
    .expect("amm_swap failed");
    instr_chain.push(swap_instr);

    let _close_user_paired_account_instr = spl_token::instruction::close_account(
        &spl_token::id(),            // Token Program
        created_user_paired_account, // Account
        &signer_keypair.pubkey(),    // Destination
        &signer_keypair.pubkey(),    // Owner
        &[],                         // MultiSigners
    )
    .expect("close_account failed");
    // instr_chain.push(close_user_paired_account_instr);

    return Ok(instr_chain);
}

pub async fn get_swap_out_instr(
    rpc_client: &RpcClient,
    signer_keypair: &Keypair,
    pool_key: &PoolKey,
    paired_addr: &Pubkey,
    token_addr: &Pubkey,
    token_amount: u64,
) -> Result<Vec<Instruction>, Box<dyn Error>> {
    let _user_target_token_account =
        get_associated_token_address(&signer_keypair.pubkey(), &token_addr);

    let mut instr_chain: Vec<Instruction> = vec![];

    let paired_token_token_account =
        get_associated_token_address(&signer_keypair.pubkey(), &paired_addr);

    let target_token_token_account =
        get_associated_token_address(&signer_keypair.pubkey(), &token_addr);

    let is_paired_token_account_initialized: bool =
        match rpc_client.get_account(&paired_token_token_account).await {
            Ok(account) => !account.data.is_empty(),
            Err(_) => false,
        };

    if !is_paired_token_account_initialized {
        let create_associated_account_instr =
            spl_associated_token_account::instruction::create_associated_token_account(
                &signer_keypair.pubkey(),
                &signer_keypair.pubkey(),
                &paired_addr,
                &spl_token::id(),
            );
        instr_chain.push(create_associated_account_instr);
    }

    let swap_instr = raydium_contract_instructions::amm_instruction::swap_base_in(
        &raydium_contract_instructions::amm_instruction::ID,
        &pool_key.id,
        &pool_key.authority,
        &pool_key.open_orders,
        &pool_key.target_orders,
        &pool_key.base_vault,
        &pool_key.quote_vault,
        &pool_key.market_program_id,
        &pool_key.market_id,
        &pool_key.market_bids,
        &pool_key.market_asks,
        &pool_key.market_event_queue,
        &pool_key.market_base_vault,
        &pool_key.market_quote_vault,
        &pool_key.market_authority,
        &target_token_token_account,
        &paired_token_token_account,
        &signer_keypair.pubkey(),
        token_amount,
        0,
    )
    .expect("amm_swap failed");
    instr_chain.push(swap_instr);

    let close_user_paired_account_instr = spl_token::instruction::close_account(
        &spl_token::id(),            // Token Program
        &paired_token_token_account, // Account
        &signer_keypair.pubkey(),    // Destination
        &signer_keypair.pubkey(),    // Owner
        &[],                         // MultiSigners
    )
    .expect("close_account failed");
    instr_chain.push(close_user_paired_account_instr);

    return Ok(instr_chain);
}

struct InstructionArgs {
    target_timestamp: i64,
    amount_in: u64,
    minimum_amount_out: u64,
    bribe_amount: u64,
    ignore_existing_balance: bool,
}

impl InstructionArgs {
    fn pack(&self) -> Result<Vec<u8>, ProgramError> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&self.target_timestamp.to_le_bytes());
        bytes.extend_from_slice(&self.amount_in.to_le_bytes());
        bytes.extend_from_slice(&self.minimum_amount_out.to_le_bytes());
        bytes.extend_from_slice(&self.bribe_amount.to_le_bytes());

        let ignore_existing_balance_byte = if self.ignore_existing_balance { 1 } else { 0 };
        bytes.push(ignore_existing_balance_byte);

        Ok(bytes)
    }
}

#[derive(Debug, Clone)]
pub struct InitializedSwapData {
    pub initialize_swap_chain: Vec<Instruction>,
    pub swap_user_src_token_account: Pubkey,
    pub swap_user_dest_token_account: Pubkey,
}

pub async fn get_modded_initialize_swap_instr(
    rpc_client: &RpcClient,
    signer_keypair: &Keypair,
    paired_addr: &Pubkey,
    wallet: &Wallet,
    token_addr: &Pubkey,
    sol_amount: f64,
) -> Result<InitializedSwapData, Box<dyn Error>> {
    let mut instr_chain: Vec<Instruction> =
        vec![ComputeBudgetInstruction::set_compute_unit_limit(352385)];

    if wallet.spam_microlamports_priority != 0 {
        let increase_gas_priority_instr = ComputeBudgetInstruction::set_compute_unit_price(wallet.spam_microlamports_priority);
        instr_chain.push(
            increase_gas_priority_instr
        );
    }

    let user_target_token_account =
        get_associated_token_address(&signer_keypair.pubkey(), &token_addr);

    let lamports_rent_exception = rpc_client
        .get_minimum_balance_for_rent_exemption(165)
        .await?;
    let seed = Alphanumeric.sample_string(&mut rand::thread_rng(), 32);
    let created_user_paired_account =
        &Pubkey::create_with_seed(&signer_keypair.pubkey(), &seed, &spl_token::id())?;
    // #1
    let create_user_paired_account_instr = system_instruction::create_account_with_seed(
        &signer_keypair.pubkey(),                              // source
        created_user_paired_account,                           // newAccount
        &signer_keypair.pubkey(),                              // base
        &seed,                                                 // seed
        lamports_rent_exception + sol_to_lamports(sol_amount), // Lamports
        165,                                                   // Space
        &spl_token::id(),                                      // Owner
    );
    instr_chain.push(create_user_paired_account_instr);

    // #2
    let initialize_user_paired_account_instr = spl_token::instruction::initialize_account(
        &spl_token::id(),            // Token Program
        created_user_paired_account, // TokenAddress
        &paired_addr,                // InitAcount
        &signer_keypair.pubkey(),    // Owner
    )?;
    instr_chain.push(initialize_user_paired_account_instr);

    // this part is already being done within our custom program
    // let associated_account_exists: bool =
    //     match rpc_client.get_account(&user_target_token_account).await {
    //         Ok(account) => !account.data.is_empty(),
    //         Err(_) => false,
    //     };

    // if !associated_account_exists {
    //     println!("Creating associated account");
    //     let create_associated_account_instr =
    //         spl_associated_token_account::instruction::create_associated_token_account(
    //             &signer_keypair.pubkey(),
    //             &signer_keypair.pubkey(),
    //             &token_addr,
    //             &spl_token::id(),
    //         );
    //     instr_chain.push(create_associated_account_instr);
    // };

    return Ok(InitializedSwapData {
        initialize_swap_chain: instr_chain,
        swap_user_src_token_account: *created_user_paired_account,
        swap_user_dest_token_account: user_target_token_account,
    });
}

pub fn get_modded_swap_chain(
    pool_key: &PoolKey,
    InitializedSwapData {
        mut initialize_swap_chain,
        swap_user_src_token_account,
        swap_user_dest_token_account,
    }: InitializedSwapData,
    signer_keypair: &Keypair,
    target_timestamp: i64,
    sol_amount: f64,
    min_amount_out_raw: u64,

    jito_bribe_wallet: &Pubkey,
    bribe_amount: f64,
    target_token_address: &Pubkey,
) -> Result<Vec<Instruction>, Box<dyn Error>> {
    let program_id = Pubkey::from_str("4dGJwas2qktCfQZ1oUc9hsJTFeTeE8HzLZiP3m5D12bw")?;

    let swap_accounts = vec![
        // raydium program id (modded addition)
        AccountMeta::new_readonly(*market::RAYDIUM_LIQUIDITY_POOL_V4_PROGRAM, false),
        // spl token (not related to raydium) (TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA)
        AccountMeta::new_readonly(spl_token::id(), false),
        // amm
        AccountMeta::new(pool_key.id, false),
        AccountMeta::new_readonly(pool_key.authority, false),
        AccountMeta::new(pool_key.open_orders, false),
        AccountMeta::new(pool_key.target_orders, false),
        AccountMeta::new(pool_key.base_vault, false),
        AccountMeta::new(pool_key.quote_vault, false),
        // serum
        AccountMeta::new_readonly(pool_key.market_program_id, false),
        AccountMeta::new(pool_key.market_id, false),
        AccountMeta::new(pool_key.market_bids, false),
        AccountMeta::new(pool_key.market_asks, false),
        AccountMeta::new(pool_key.market_event_queue, false),
        AccountMeta::new(pool_key.market_base_vault, false),
        AccountMeta::new(pool_key.market_quote_vault, false),
        AccountMeta::new(pool_key.market_authority, false),
        // user
        AccountMeta::new(swap_user_src_token_account, false),
        AccountMeta::new(swap_user_dest_token_account, false),
        AccountMeta::new(signer_keypair.pubkey(), true),

        AccountMeta::new(*jito_bribe_wallet, false),
        AccountMeta::new(system_program::id(), false),
        AccountMeta::new(*target_token_address, false),
        AccountMeta::new(spl_associated_token_id(), false),

        AccountMeta::new(Pubkey::from_str("SysvarRent111111111111111111111111111111111")?, false),
    ];
    let swap_instr = Instruction {
        accounts: swap_accounts.clone(),
        data: InstructionArgs {
            target_timestamp: target_timestamp,
            amount_in: sol_to_lamports(sol_amount),
            minimum_amount_out: min_amount_out_raw,
            bribe_amount: sol_to_lamports(bribe_amount),
            ignore_existing_balance: false,
        }
        .pack()?,
        program_id,
    };

    let close_user_src_token_account = spl_token::instruction::close_account(
        &spl_token::id(),            // Token Program
        &swap_user_src_token_account, // Account
        &signer_keypair.pubkey(),    // Destination
        &signer_keypair.pubkey(),    // Owner
        &[],                         // MultiSigners
    )
    .expect("close_account failed");

    initialize_swap_chain.extend_from_slice(&[swap_instr, close_user_src_token_account]);

    Ok(initialize_swap_chain)
}
