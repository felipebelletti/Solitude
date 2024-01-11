    // TODO (https://github.dev/openbook-dex/openbook-v2 search for "let market_base_vault = " etc, I just don't want to mess with it rn)
    // https://github.dev/raydium-io/raydium-sdk (search for getAssociatedBaseVault, etc)
    // https://solana.stackexchange.com/questions/8393/find-marketauthority-serumvaultsigner-owner-of-non-ata-vault-accounts

    /*
    let crafted_swap_data = local_api::get_raydium_crafted_swap(
        target_addr.to_string(),
        "".to_string(),
        target_addr.to_string(),
    )
    .await?;

    println!("{:?}\n", crafted_swap_data);

    let swap_instruction = instruction::swap_base_in(
        &Pubkey::from_str(&crafted_swap_data.program_id).unwrap(),
        &Pubkey::from_str(&crafted_swap_data.id).unwrap(),
        &Pubkey::from_str(&crafted_swap_data.authority).unwrap(),
        &Pubkey::from_str(&crafted_swap_data.open_orders).unwrap(),
        &Pubkey::from_str(&crafted_swap_data.target_orders).unwrap(),
        &Pubkey::from_str(&crafted_swap_data.base_vault).unwrap(),
        &Pubkey::from_str(&crafted_swap_data.quote_vault).unwrap(),
        &Pubkey::from_str(&crafted_swap_data.market_program_id).unwrap(),
        &Pubkey::from_str(&crafted_swap_data.market_id).unwrap(),
        &Pubkey::from_str(&crafted_swap_data.market_bids).unwrap(),
        &Pubkey::from_str(&crafted_swap_data.market_asks).unwrap(),
        &Pubkey::from_str(&crafted_swap_data.market_event_queue).unwrap(),
        // &Pubkey::from_str(&crafted_swap_data.market_base_vault).unwrap(),  // SerumCoinVaultAccount (wrong)
        &Pubkey::from_str("FGoMpWDmxRXh4SiMFkUichRow6Z58215AwhVNFZ4uLLh").unwrap(),
        // &Pubkey::from_str(&crafted_swap_data.market_quote_vault).unwrap(), // SerumPcVaultAccount (wrong)
        &Pubkey::from_str("HaSmnKmukZeBKyD6dhwfUDL5oTJAEza1aSH1BjYSrSkt").unwrap(),
        &Pubkey::from_str(&crafted_swap_data.market_authority).unwrap(),
        &main_keypair.pubkey(),
        &main_keypair.pubkey(),
        &main_keypair.pubkey(),
        sol_to_lamports(0.001),
        0,
    )
    .expect("swap_base_in failed");
    */

https://solscan.io/tx/5YfrpvaW2eY1ryE5YsJ9BdXDEjRKAYcStcZWqNteXQNYh3nCH75rvBpqKbjFLPcuhXsengCyTKc99PQf5w1ik3qM
this is an addliquidity tx