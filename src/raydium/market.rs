use base64;
use chrono::Local;
use lazy_static::lazy_static;
use solana_account_decoder::UiAccountEncoding;
use solana_client::nonblocking::rpc_client;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig};
use solana_client::rpc_filter::{Memcmp, MemcmpEncodedBytes, MemcmpEncoding, RpcFilterType};
use solana_sdk::account::Account;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use std::error::Error;
use std::str::FromStr;

use super::public_api::Pool;
use super::utils::{
    get_associated_base_vault, get_associated_id, get_associated_open_orders,
    get_associated_quote_vault, get_associated_target_orders,
};
use crate::raydium::utils::get_associated_authority;
use crate::utils::get_token_decimals;

lazy_static! {
    static ref OPENBOOK_PROGRAM_ID: Pubkey =
        Pubkey::from_str("srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX").unwrap();
    static ref RAYDIUM_LIQUIDITY_POOL_V4_PROGRAM: Pubkey =
        Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8").unwrap();
}

const OPENBOOK_DATASIZE: u64 = 388;
const RAYDIUM_POOL_DATASIZE: u64 = 752;

// find openbook markets where base_mint OR quote_mint is the target token address
pub async fn get_openbook_market_for_address(
    target_token_address: &Pubkey,
    client: &RpcClient,
) -> Result<(Pubkey, Account), Box<dyn Error>> {
    const BASE_MINT_OFFSET: usize = 53;
    const QUOTE_MINT_OFFSET: usize = 85;

    let base_mint_filtered_accounts = client
        .get_program_accounts_with_config(
            &OPENBOOK_PROGRAM_ID,
            RpcProgramAccountsConfig {
                filters: Some(vec![
                    RpcFilterType::DataSize(OPENBOOK_DATASIZE),
                    RpcFilterType::Memcmp(Memcmp::new(
                        BASE_MINT_OFFSET,
                        MemcmpEncodedBytes::Base58(target_token_address.to_string()),
                    )),
                ]),
                account_config: RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    commitment: Some(client.commitment()),
                    ..RpcAccountInfoConfig::default()
                },
                ..RpcProgramAccountsConfig::default()
            },
        )
        .await?;

    if base_mint_filtered_accounts.len() > 0 {
        return Ok(base_mint_filtered_accounts[0].clone());
    }

    let quote_mint_filtered_accounts = client
        .get_program_accounts_with_config(
            &OPENBOOK_PROGRAM_ID,
            RpcProgramAccountsConfig {
                filters: Some(vec![
                    RpcFilterType::DataSize(OPENBOOK_DATASIZE),
                    RpcFilterType::Memcmp(Memcmp::new(
                        QUOTE_MINT_OFFSET,
                        MemcmpEncodedBytes::Base58(target_token_address.to_string()),
                    )),
                ]),
                account_config: RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    commitment: Some(client.commitment()),
                    ..RpcAccountInfoConfig::default()
                },
                ..RpcProgramAccountsConfig::default()
            },
        )
        .await
        .unwrap();

    if quote_mint_filtered_accounts.len() > 0 {
        return Ok(quote_mint_filtered_accounts[0].clone());
    }

    Err("No market found for token".into())
}

pub async fn exhaustive_get_openbook_market_for_address(
    target_token_address: &Pubkey,
    client: &RpcClient,
) -> Result<(Pubkey, Account), Box<dyn Error>> {
    loop {
        match get_openbook_market_for_address(target_token_address, client).await {
            Ok(market) => return Ok(market),
            Err(_) => {
                println!(
                    "{} | No market found for token, trying again in 300ms",
                    Local::now().format("%H:%M:%S")
                );
                std::thread::sleep(std::time::Duration::from_micros(300));
            }
        };
    }
}

pub async fn exhaustive_get_raydium_pool_for_address(
    target_token_address: &Pubkey,
    client: &RpcClient,
) -> Result<(Pubkey, Account), Box<dyn Error>> {
    loop {
        match get_raydium_pool_for_address(target_token_address, client).await {
            Ok(market) => return Ok(market),
            Err(_) => {
                println!(
                    "{} | No raydium pool found for token, trying again in 300ms",
                    Local::now().format("%H:%M:%S")
                );
                std::thread::sleep(std::time::Duration::from_micros(300));
            }
        };
    }
}

// find openbook markets where base_mint OR quote_mint is the target token address
pub async fn get_raydium_pool_for_address(
    target_token_address: &Pubkey,
    client: &RpcClient,
) -> Result<(Pubkey, Account), Box<dyn Error>> {
    const BASE_MINT_OFFSET: usize = 400;
    const QUOTE_MINT_OFFSET: usize = 432;

    let base_mint_filtered_accounts = client
        .get_program_accounts_with_config(
            &RAYDIUM_LIQUIDITY_POOL_V4_PROGRAM,
            RpcProgramAccountsConfig {
                filters: Some(vec![
                    RpcFilterType::DataSize(RAYDIUM_POOL_DATASIZE),
                    RpcFilterType::Memcmp(Memcmp::new(
                        BASE_MINT_OFFSET,
                        MemcmpEncodedBytes::Base58(target_token_address.to_string()),
                    )),
                ]),
                account_config: RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    commitment: Some(client.commitment()),
                    ..RpcAccountInfoConfig::default()
                },
                ..RpcProgramAccountsConfig::default()
            },
        )
        .await?;

    if base_mint_filtered_accounts.len() > 0 {
        return Ok(base_mint_filtered_accounts[0].clone());
    }

    let quote_mint_filtered_accounts = client
        .get_program_accounts_with_config(
            &RAYDIUM_LIQUIDITY_POOL_V4_PROGRAM,
            RpcProgramAccountsConfig {
                filters: Some(vec![
                    RpcFilterType::DataSize(RAYDIUM_POOL_DATASIZE),
                    RpcFilterType::Memcmp(Memcmp::new(
                        QUOTE_MINT_OFFSET,
                        MemcmpEncodedBytes::Base58(target_token_address.to_string()),
                    )),
                ]),
                account_config: RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    commitment: Some(client.commitment()),
                    ..RpcAccountInfoConfig::default()
                },
                ..RpcProgramAccountsConfig::default()
            },
        )
        .await
        .unwrap();

    if quote_mint_filtered_accounts.len() > 0 {
        return Ok(quote_mint_filtered_accounts[0].clone());
    }

    Err("No raydium pool found for token".into())
}

#[derive(Debug)]
pub struct OpenbookMarket {
    pub market_id: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub market_base_vault: Pubkey,
    pub market_quote_vault: Pubkey,
    pub market_event_queue: Pubkey,
    pub market_bids: Pubkey,
    pub market_asks: Pubkey,
}

pub fn parse_openbook_market_account(account: &Account) -> OpenbookMarket {
    return OpenbookMarket {
        market_id: Pubkey::new(&account.data[13..45]),
        base_mint: Pubkey::new(&account.data[53..85]),
        quote_mint: Pubkey::new(&account.data[85..117]),
        market_base_vault: Pubkey::new(&account.data[117..149]),
        market_quote_vault: Pubkey::new(&account.data[165..197]),
        market_event_queue: Pubkey::new(&account.data[253..285]),
        market_bids: Pubkey::new(&account.data[285..317]),
        market_asks: Pubkey::new(&account.data[317..349]),
    };
}

#[derive(Debug)]
pub struct RaydiumPool {
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub lp_mint: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub open_orders: Pubkey,
    pub market_id: Pubkey,
    pub market_program_id: Pubkey,
    pub target_orders: Pubkey,
}

pub fn parse_raydium_pool_account(account: Account) -> RaydiumPool {
    return RaydiumPool {
        base_mint: Pubkey::new(&account.data[400..432]),
        quote_mint: Pubkey::new(&account.data[432..464]),
        lp_mint: Pubkey::new(&account.data[464..496]),
        base_vault: Pubkey::new(&account.data[336..368]),
        quote_vault: Pubkey::new(&account.data[368..400]),
        open_orders: Pubkey::new(&account.data[496..528]),
        market_id: Pubkey::new(&account.data[528..560]),
        market_program_id: Pubkey::new(&account.data[560..592]),
        target_orders: Pubkey::new(&account.data[592..624]),
    };
}

#[derive(Debug)]
pub struct PoolKey {
    pub id: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    // pub lp_mint: Pubkey, // not necessary for building a swap instruction
    pub base_decimals: u8,
    pub quote_decimals: u8,
    // pub lp_decimals: u8, // not necessary for building a swap instruction
    pub version: i32,
    pub program_id: Pubkey,
    pub authority: Pubkey,
    pub open_orders: Pubkey,
    pub target_orders: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub withdraw_queue: String, // cte. "11111111111111111111111111111111"
    pub lp_vault: String,       // cte. "11111111111111111111111111111111"
    pub market_version: i32,
    pub market_program_id: Pubkey,
    pub market_id: Pubkey,
    pub market_authority: Pubkey,
    pub market_base_vault: Pubkey,
    pub market_quote_vault: Pubkey,
    pub market_bids: Pubkey,
    pub market_asks: Pubkey,
    pub market_event_queue: Pubkey,
}

pub async fn craft_pool_key(
    rpc_client: &RpcClient,
    openbook_market: &OpenbookMarket,
    openbook_market_addr: &Pubkey,
) -> Result<PoolKey, Box<dyn Error>> {
    let base_decimals = get_token_decimals(rpc_client, &openbook_market.base_mint)
        .await
        .unwrap();
    let quote_decimals = get_token_decimals(rpc_client, &openbook_market.quote_mint)
        .await
        .unwrap();

    let market_base_vault_account = rpc_client
        .get_account(&openbook_market.market_base_vault)
        .await?;
    let market_authority = Pubkey::new(&market_base_vault_account.data[32..64]);

    let raydium_pool_addr = get_associated_id(
        &RAYDIUM_LIQUIDITY_POOL_V4_PROGRAM,
        &openbook_market.market_id,
    )?;

    println!("raydium_pool_addr: {}", raydium_pool_addr);

    // derivated keys from raydium_pool_id
    let derivated_raydium_open_orders = get_associated_open_orders(
        &RAYDIUM_LIQUIDITY_POOL_V4_PROGRAM,
        &openbook_market.market_id,
    )
    .expect("derivated_raydium_open_orders");

    println!(
        "derivated_raydium_open_orders: {}",
        derivated_raydium_open_orders
    );

    let derivated_raydium_target_orders = get_associated_target_orders(
        &RAYDIUM_LIQUIDITY_POOL_V4_PROGRAM,
        &openbook_market.market_id,
    )
    .expect("derivated_raydium_target_orders");

    println!(
        "derivated_raydium_target_orders: {}",
        derivated_raydium_target_orders
    );

    let derivated_raydium_pool_base_vault = get_associated_base_vault(
        &RAYDIUM_LIQUIDITY_POOL_V4_PROGRAM,
        &openbook_market.market_id,
    )
    .expect("derivated_raydium_pool_base_vault");

    println!(
        "derivated_raydium_pool_base_vault: {}",
        derivated_raydium_pool_base_vault
    );

    let derivated_raydium_pool_quote_vault = get_associated_quote_vault(
        &RAYDIUM_LIQUIDITY_POOL_V4_PROGRAM,
        &openbook_market.market_id,
    )
    .expect("derivated_raydium_pool_quote_vault");

    println!(
        "derivated_raydium_pool_quote_vault: {}",
        derivated_raydium_pool_quote_vault
    );

    // let derivated_raydium_pool_base_vault_account = rpc_client
    //     .get_account(&derivated_raydium_pool_base_vault)
    //     .await.expect("derivated_raydium_pool_base_vault_account");
    // let authority = Pubkey::new(&derivated_raydium_pool_base_vault_account.data[32..64]);
    let authority =
        get_associated_authority(&RAYDIUM_LIQUIDITY_POOL_V4_PROGRAM).expect("authority");
    println!("Authority: {}", authority);

    Ok(PoolKey {
        id: raydium_pool_addr,
        base_mint: openbook_market.base_mint,
        quote_mint: openbook_market.quote_mint,
        base_decimals: base_decimals,
        quote_decimals: quote_decimals,
        version: 4,
        program_id: *RAYDIUM_LIQUIDITY_POOL_V4_PROGRAM,
        authority: authority,
        open_orders: derivated_raydium_open_orders,
        target_orders: derivated_raydium_target_orders,
        base_vault: derivated_raydium_pool_base_vault,
        quote_vault: derivated_raydium_pool_quote_vault,
        withdraw_queue: "11111111111111111111111111111111".to_string(),
        lp_vault: "11111111111111111111111111111111".to_string(),
        market_version: 4,
        market_program_id: *OPENBOOK_PROGRAM_ID,
        market_id: openbook_market_addr.clone(),
        market_authority: market_authority,
        market_base_vault: openbook_market.market_base_vault,
        market_quote_vault: openbook_market.market_quote_vault,
        market_bids: openbook_market.market_bids,
        market_asks: openbook_market.market_asks,
        market_event_queue: openbook_market.market_event_queue,
    })
}
