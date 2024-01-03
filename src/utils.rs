use std::error::Error;

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_program::{pubkey::Pubkey, program_pack::Pack};

pub async fn get_token_decimals(client: &RpcClient, token_mint_address: &Pubkey) -> Result<u8, Box<dyn Error>> {
    let account = client.get_account(token_mint_address).await?;
    let mint_data = spl_token::state::Mint::unpack(&account.data)?;

    Ok(mint_data.decimals)
}