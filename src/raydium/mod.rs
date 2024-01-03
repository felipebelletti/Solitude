use std::error::Error;

use rand::distributions::{Alphanumeric, DistString};
use solana_program::{instruction::Instruction, system_instruction};
use spl_associated_token_account::get_associated_token_address;
use solana_sdk::{
    bs58,
    commitment_config::{CommitmentConfig, CommitmentLevel},
    native_token::sol_to_lamports,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction::transfer,
    transaction::{Transaction, VersionedTransaction},
};
use solana_client::nonblocking::rpc_client::RpcClient;

use self::market::PoolKey;

pub mod public_api;
pub mod market;

pub async fn get_swap_in_instr(rpc_client: &RpcClient, signer_keypair: &Keypair, pool_key: &PoolKey, paired_addr: &Pubkey, token_addr: &Pubkey, sol_amount: f64) -> Result<Vec<Instruction>, Box<dyn Error>> {
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
        &signer_keypair.pubkey(),                                // source
        created_user_paired_account,                           // newAccount
        &signer_keypair.pubkey(),                                // base
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
        &signer_keypair.pubkey(),      // Owner
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
        1,
    )
    .expect("amm_swap failed");
    instr_chain.push(swap_instr);

    let close_user_paired_account_instr = spl_token::instruction::close_account(
        &spl_token::id(),            // Token Program
        created_user_paired_account, // Account
        &signer_keypair.pubkey(),      // Destination
        &signer_keypair.pubkey(),      // Owner
        &[],                         // MultiSigners
    )
    .expect("close_account failed");
    // instr_chain.push(close_user_paired_account_instr);

    // instr_chain.push(transfer(
    //     &signer_keypair.pubkey(),
    //     &tip_account,
    //     sol_to_lamports(0.01),
    // ));

    return Ok(instr_chain);
}