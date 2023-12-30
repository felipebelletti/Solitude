use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct PoolRequest {
    pub base_mint: String,
    pub quote_mint: String,
    pub target_token: String,
}

#[derive(Deserialize, Debug)]
pub struct PoolResponse {
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

pub async fn get_raydium_crafted_swap(base_mint: String, quote_mint: String, target_token: String, exhaustive: bool) -> Result<PoolResponse, reqwest::Error> {
    loop {
        let client = reqwest::Client::new();
        let url = "http://127.0.0.1:3000/api/get_pool_info";
        let req_body = PoolRequest {
            base_mint: base_mint.clone(),
            quote_mint: quote_mint.clone(),
            target_token: target_token.clone(),
        };
    
        let resp = match client.post(url)
            .json(&req_body)
            .send()
            .await?
            .json::<PoolResponse>()
            .await {
                Ok(resp) => resp,
                Err(e) => {
                    if exhaustive {
                        println!("Openserum err: {:?}", e);
                        continue;
                    }
                    return Err(e)
                }
            };
    
        return Ok(resp)
    }
}