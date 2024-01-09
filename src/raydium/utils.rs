use solana_sdk::pubkey::Pubkey;
use std::error::Error;

pub fn get_associated_id(program_id: &Pubkey, market_id: &Pubkey) -> Result<Pubkey, Box<dyn Error>> {
    let seeds = &[program_id.as_ref(), market_id.as_ref(), b"amm_associated_seed"];
    Ok(Pubkey::find_program_address(seeds, program_id).0)
}

pub fn get_associated_authority(program_id: &Pubkey) -> Result<Pubkey, Box<dyn Error>> {
    let seed: &[u8] = b"amm authority";
    let seeds: &[&[u8]] = &[&seed];
    Ok(Pubkey::find_program_address(seeds, program_id).0)
}

pub fn get_associated_base_vault(program_id: &Pubkey, market_id: &Pubkey) -> Result<Pubkey, Box<dyn Error>> {
    // println!("Using programId: {} | Using marketId: {}", program_id, market_id);

    let seeds = &[program_id.as_ref(), market_id.as_ref(), b"coin_vault_associated_seed"];
    Ok(Pubkey::find_program_address(seeds, program_id).0)
}

pub fn get_associated_quote_vault(program_id: &Pubkey, market_id: &Pubkey) -> Result<Pubkey, Box<dyn Error>> {
    let seeds = &[program_id.as_ref(), market_id.as_ref(), b"pc_vault_associated_seed"];
    Ok(Pubkey::find_program_address(seeds, program_id).0)
}

pub fn get_associated_lp_mint(program_id: &Pubkey, market_id: &Pubkey) -> Result<Pubkey, Box<dyn Error>> {
    let seeds = &[program_id.as_ref(), market_id.as_ref(), b"lp_mint_associated_seed"];
    Ok(Pubkey::find_program_address(seeds, program_id).0)
}

pub fn get_associated_lp_vault(program_id: &Pubkey, market_id: &Pubkey) -> Result<Pubkey, Box<dyn Error>> {
    let seeds = &[program_id.as_ref(), market_id.as_ref(), b"temp_lp_token_associated_seed"];
    Ok(Pubkey::find_program_address(seeds, program_id).0)
}

pub fn get_associated_target_orders(program_id: &Pubkey, market_id: &Pubkey) -> Result<Pubkey, Box<dyn Error>> {
    let seeds = &[program_id.as_ref(), market_id.as_ref(), b"target_associated_seed"];
    Ok(Pubkey::find_program_address(seeds, program_id).0)
}

pub fn get_associated_withdraw_queue(program_id: &Pubkey, market_id: &Pubkey) -> Result<Pubkey, Box<dyn Error>> {
    let seeds = &[program_id.as_ref(), market_id.as_ref(), b"withdraw_associated_seed"];
    Ok(Pubkey::find_program_address(seeds, program_id).0)
}

pub fn get_associated_open_orders(program_id: &Pubkey, market_id: &Pubkey) -> Result<Pubkey, Box<dyn Error>> {
    let seeds = &[program_id.as_ref(), market_id.as_ref(), b"open_order_associated_seed"];
    Ok(Pubkey::find_program_address(seeds, program_id).0)
}
