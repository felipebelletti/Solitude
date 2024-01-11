use reqwest;
use serde::{Deserialize};


// #[derive(Deserialize, Debug)]
// pub struct Pool {
//     pub id: String,
//     pub baseMint: String,
//     pub quoteMint: String,
//     pub lpMint: String,
//     pub baseDecimals: u8,
//     pub quoteDecimals: u8,
//     pub lpDecimals: u8,
//     pub version: u8,
//     pub programId: String,
//     pub authority: String,
//     pub openOrders: String,
//     pub targetOrders: String,
//     pub baseVault: String,
//     pub quoteVault: String,
//     pub withdrawQueue: String,
//     pub lpVault: String,
//     pub marketVersion: u8,
//     pub marketProgramId: String,
//     pub marketId: String,
//     pub marketAuthority: String,
//     pub marketBaseVault: String,
//     pub marketQuoteVault: String,
//     pub marketBids: String,
//     pub marketAsks: String,
//     pub marketEventQueue: String,
//     pub lookupTableAccount: Option<String>,
// }

#[derive(Deserialize, Debug)]
pub struct Pool {
    pub id: String,
    #[serde(rename = "baseMint")]
    pub base_mint: String,
    #[serde(rename = "quoteMint")]
    pub quote_mint: String,
    #[serde(rename = "lpMint")]
    pub lp_mint: String,
    #[serde(rename = "baseDecimals")]
    pub base_decimals: u8,
    #[serde(rename = "quoteDecimals")]
    pub quote_decimals: u8,
    #[serde(rename = "lpDecimals")]
    pub lp_decimals: u8,
    pub version: u8,
    #[serde(rename = "marketEventQueue")]
    pub market_event_queue: String,
    #[serde(rename = "marketBaseVault")]
    pub market_base_vault: String,
    #[serde(rename = "marketQuoteVault")]
    pub market_quote_vault: String,
    pub authority: String,
    #[serde(rename = "programId")]
    pub program_id: String,
    #[serde(rename = "openOrders")]
    pub open_orders: String,
    #[serde(rename = "targetOrders")]
    pub target_orders: String,
    #[serde(rename = "baseVault")]
    pub base_vault: String,
    #[serde(rename = "quoteVault")]
    pub quote_vault: String,
    #[serde(rename = "withdrawQueue")]
    pub withdraw_queue: String,
    #[serde(rename = "lpVault")]
    pub lp_vault: String,
    #[serde(rename = "marketVersion")]
    pub market_version: u8,
    #[serde(rename = "marketProgramId")]
    pub market_program_id: String,
    #[serde(rename = "marketId")]
    pub market_id: String,
    #[serde(rename = "marketAuthority")]
    pub market_authority: String,
    #[serde(rename = "marketAsks")]
    pub market_asks: String,
    #[serde(rename = "marketBids")]
    pub market_bids: String,
}

#[derive(Deserialize, Debug)]
pub struct Response {
    official: Vec<Pool>,
    unOfficial: Vec<Pool>,
}

pub async fn get_pools() -> Result<Vec<Pool>, reqwest::Error> {
    let url = "https://api.raydium.io/v2/sdk/liquidity/mainnet.json";
    let resp = reqwest::get(url).await?.json::<Response>().await?;

    let mut all_pools = resp.official;
    all_pools.extend(resp.unOfficial);

    Ok(all_pools)
}

pub async fn get_pool_by_mints(base_mint: &str, quote_mint: Option<&str>) -> Result<Vec<Pool>, reqwest::Error> {
    let all_pools = get_pools().await?;

    let filtered_pools: Vec<Pool> = all_pools
        .into_iter()
        .filter(|pool| {
            pool.base_mint == base_mint && 
            (quote_mint.is_none() || quote_mint.unwrap() == pool.quote_mint)
        })
        .collect();

    Ok(filtered_pools)
}

pub async fn get_pool_by_target(target_mint: &str) -> Result<Vec<Pool>, reqwest::Error> {
    let all_pools = get_pools().await?;

    let filtered_pools: Vec<Pool> = all_pools
        .into_iter()
        .filter(|pool| {
            pool.base_mint == target_mint || pool.quote_mint == target_mint
        })
        .collect();

    Ok(filtered_pools)
}