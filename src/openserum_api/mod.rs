use serde::Deserialize;
use solana_sdk::pubkey::Pubkey;

#[derive(Deserialize, Debug)]
pub struct SerumTokenData {
    pub base_lot_size: i64,
    pub base_decimals: u8,
    pub quote_mint: Pubkey, // double check if it's deserializing correctly
    pub base_deposits_float: f64,
    pub quote_deposits_total: i64,
    pub is_zeta: bool,
    pub base_symbol: String,
    pub quote_logo: String,
    pub event_queue: String,
    pub base_mint: Pubkey,
    pub asks: String,
    pub percentage: f64,
    pub bids: String,
    pub quote_lot_size: i64,
    pub quote_decimals: u8,
    pub base_deposits_total: i64,
    pub id: String,
    pub base_logo: String,
    pub base_name: String,
    pub quote_name: String,
    pub quote_symbol: String,
    pub quote_deposits_float: f64,
}

pub async fn get_serum_token_data() -> Result<Vec<SerumTokenData>, reqwest::Error> {
    let url = "https://openserum.io/api/serum/token/HTDzTToLb6B2ueSdEbq2BSzNJVtqX4dX7PqZpkZQ5NCP";
    let resp: Vec<SerumTokenData> = reqwest::get(url).await?.json().await?;

    Ok(resp)
}