use std::error::Error;
use std::str::FromStr;

use chrono::format::DelayedFormat;
use chrono::format::StrftimeItems;
use jito_protos::auth;
use solana_account_decoder::UiAccountData;
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_request::TokenAccountsFilter};
use solana_program::{program_option::COption, program_pack::Pack, pubkey::Pubkey};
use solana_sdk::{signature::Keypair, signer::Signer};
use spl_token::state::Account as TokenAccount;
use spl_token::state::Mint;

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
    bought_wallet: Keypair,
    paired_token_addr: &Pubkey,
    target_token_addr: &Pubkey,
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

    let token_balance = client.get_token_account_balance(&token_account_addr).await?;

    

    Ok(())
}

pub fn now_ms() -> DelayedFormat<StrftimeItems<'static>>{
    chrono::Local::now().format("%H:%M:%S%.3f")
}