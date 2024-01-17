use solana_program::instruction::CompiledInstruction;
use solana_sdk::{bs58, pubkey::Pubkey, signature::Keypair};

use solitude::{jito, utils};

use std::{
    collections::HashMap,
    error::Error,
    panic::{self, PanicInfo},
    process::{self},
    str::FromStr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let jito_auth_keypair = Arc::new(
        Keypair::from_bytes(
            &bs58::decode("***REMOVED***")
                .into_vec()
                .unwrap(),
        )
        .unwrap(),
    );

    let block_engine_url = "https://frankfurt.mainnet.block-engine.jito.wtf";
    let rpc_pubsub_addr = "http://127.0.0.1:8899/";
    let _rpc_pda_url = "https://tame-ancient-mountain.solana-mainnet.quiknode.pro/6a9a95bf7bbb108aea620e7ee4c1fd5e1b67cc62";

    let (searcher_client, _) = jito::get_searcher_client(
        &jito_auth_keypair,
        &graceful_panic(None),
        block_engine_url,
        rpc_pubsub_addr,
    )
    .await
    .expect("get_searcher_client failed");
    let searcher_client = Arc::new(searcher_client);

    // TODO: read from tracker-wallets.jsonl
    let watch_mempool_addresses: Vec<Pubkey> = vec![
        Pubkey::from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8")?,
        Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")?,
    ];

    let mut mempool_ch = searcher_client
        .subscribe_mempool_programs(
            &watch_mempool_addresses,
            vec![
                "amsterdam".to_string(),
                "frankfurt".to_string(),
                "ny".to_string(),
                "tokyo".to_string(),
            ],
            1024,
        )
        .await?;

    let methods: Arc<Vec<String>> = Arc::new(vec![
        "0950".to_string(), // ? swap
        "0b00".to_string(), // ? swap
        "0900".to_string(), // ? swap
        "09ac".to_string(), // dragonfly
    ]);
    let wallet_to_person: Arc<Mutex<HashMap<Pubkey, &str>>> = Arc::new(Mutex::new(HashMap::new()));
    wallet_to_person.lock().unwrap().insert(
        Pubkey::from_str("9mor3nwvFkd1xiAy56jw4hnasZFFkjbKLzdZ1kxSyz77")?,
        "wep1",
    );
    wallet_to_person.lock().unwrap().insert(
        Pubkey::from_str("BSyAtkb4S36XXs9VDz7xc1KiN4RbuwRdTYJTSAo96mop")?,
        "wep2",
    );
    wallet_to_person.lock().unwrap().insert(
        Pubkey::from_str("CLT6JrLf2AMx7ju2hQpz7yUhEBziVgAnDVNx8kmJbT67")?,
        "pipi",
    );
    wallet_to_person.lock().unwrap().insert(
        Pubkey::from_str("MAFYMHwNxpzW3w8cTQnmqfEssgxq3cRsVNrjs6kNkAG")?,
        "anemone",
    );
    wallet_to_person.lock().unwrap().insert(
        Pubkey::from_str("8bBmx5L4XKfKcc27ry4nQ4uPx4xr31AAei5EXWwatnZ5")?,
        "balao",
    );
    wallet_to_person.lock().unwrap().insert(
        Pubkey::from_str("CxVb5zuyLsUiKHwtjtKW96YcdF24pJTWoitNXXyVMHcX")?,
        "snk",
    );
    wallet_to_person.lock().unwrap().insert(
        Pubkey::from_str("4FTvLGKLfrhMechFrRwngC8bKENCGEjutg5xXdzyKkzc")?,
        "maskot",
    );
    wallet_to_person.lock().unwrap().insert(
        Pubkey::from_str("bjEf4LPBqwGfn8xGfSToy7VUanQeT1EifEwZDopHk9Y")?,
        "detetive",
    );
    let ignore_unknown_callers = true;

    loop {
        while let Some(txs) = mempool_ch.recv().await {
            for mempool_tx in txs {
                let methods_clone: Arc<Vec<String>> = methods.clone();
                let wallet_to_person_clone = wallet_to_person.clone();

                tokio::spawn(async move {
                    let hash = mempool_tx.signatures[0];
                    let accounts: &[Pubkey] = mempool_tx.message.static_account_keys();
                    let signer = accounts[0];
                    let instr_chain = mempool_tx.message.instructions();

                    if instr_chain.len() == 2 && hex::encode(instr_chain[1].data.clone()).starts_with("0006") {
                        println!("Alpha launch detected ({})", hash);
                        utils::tell(format!("Alpha launch detected ({})", hash));
                        return;
                    }

                    for instr in instr_chain {
                        let instr_data_hex = hex::encode(instr.data.clone());

                        if methods_clone
                            .iter()
                            .any(|method| instr_data_hex.starts_with(method))
                        {
                            let account_indexes_used = &instr.accounts;

                            let _swap_grouped_accounts = account_indexes_used
                                .iter()
                                .filter_map(|&index| accounts.get(index as usize))
                                .collect::<Vec<&Pubkey>>();

                            let associated_account_instr: &CompiledInstruction = match instr_chain
                                .iter()
                                .find(|&instr| hex::encode(instr.data.clone()) == "00")
                            {
                                Some(instr) => instr,
                                None => {
                                    // println!("Could not get associated_account_instr ({})", hash);
                                    return;
                                }
                            };

                            let target_token_address_index = match associated_account_instr
                                .accounts
                                .get(3)
                            {
                                Some(value) => value,
                                None => {
                                    println!("Could not get target_token_address_index from associated_account_instr");
                                    return;
                                }
                            };
                            let target_token_address = {
                                if target_token_address_index > &accounts.len() as &u8 {
                                    println!("target_token_address_index > accounts.len()");
                                    Pubkey::default()
                                } else {
                                    accounts[*target_token_address_index as usize]
                                }
                            };

                            let person = match wallet_to_person_clone.lock().unwrap().get(&signer) {
                                Some(value) => value,
                                None => {
                                    if ignore_unknown_callers {
                                        return;
                                    }
                                    "unknown"
                                }
                            };
                            println!(
                                "{} is getting sniped by {} ({}) | {}",
                                target_token_address, signer, person, hash
                            );
                        }
                    }
                });
            }
        }
    }

    Ok(())
}

pub fn graceful_panic(callback: Option<fn(&PanicInfo)>) -> Arc<AtomicBool> {
    let exit = Arc::new(AtomicBool::new(false));
    // Fail fast!
    let panic_hook = panic::take_hook();
    {
        let exit = exit.clone();
        panic::set_hook(Box::new(move |panic_info| {
            if let Some(f) = callback {
                f(panic_info);
            }
            exit.store(true, Ordering::Relaxed);
            println!("Disconnecting from JITO relayer...");
            // let other loops finish up
            std::thread::sleep(Duration::from_secs(5));
            // invoke the default handler and exit the process
            panic_hook(panic_info); // print the panic backtrace. default exit code is 101

            process::exit(1); // bail us out if thread blocks/refuses to join main thread
        }));
    }
    exit
}
