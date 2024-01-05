use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path, fs::File, io::{self, BufRead, Read}, error::Error};

#[derive(Debug, Deserialize, Serialize)]
pub struct Wallet {
    pub pk: String,
    pub amounts: HashMap<String, f64>,
    pub bribe_amount: f64,
    pub spam: bool,
    pub filter_liquidity: bool,
}

pub fn read_from_wallet_file() -> Wallet {
    let mut file = File::open("./wallet.json").expect("err read wallet file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("err reading into string wallet contents");
    let wallet: Wallet = serde_json::from_str(&contents).expect("err parsing wallet string content as json");

    return wallet;
    
}