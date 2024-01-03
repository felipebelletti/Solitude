use std::error::Error;

use jito_protos::auth;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_program::{pubkey::Pubkey, program_pack::Pack, program_option::COption};
use spl_token::state::Mint;

pub async fn get_token_decimals(client: &RpcClient, token_mint_address: &Pubkey) -> Result<u8, Box<dyn Error>> {
    let account = client.get_account(token_mint_address).await?;
    let mint_data = spl_token::state::Mint::unpack(&account.data)?;

    Ok(mint_data.decimals)
}

pub async fn get_token_authority(client: &RpcClient, token_mint_address: &Pubkey) -> Result<COption<Pubkey>, Box<dyn Error>> {
    let account_data = client.get_account_data(&token_mint_address).await?;
    let mint = Mint::unpack(&account_data)?;

    Ok(mint.mint_authority)
}