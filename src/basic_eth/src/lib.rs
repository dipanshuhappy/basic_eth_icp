use candid::{candid_method, Principal};
use ic_cdk::api::management_canister::http_request::{HttpResponse, TransformArgs};
use ic_cdk_macros::{self, update, query};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

use ic_web3::transports::ICHttp;
use ic_web3::Web3;
use ic_web3::ic::{get_eth_addr, KeyInfo};
use ic_web3::{
    contract::{Contract, Options},
    ethabi::ethereum_types::{U64, U256},
    types::{Address, TransactionParameters, BlockId},
};
use std::cell::RefCell;

/// Shared memory for caching the Ethereum address, protected by a mutex.
lazy_static::lazy_static! {
    static ref ADDRESS: Arc<Mutex<String>> = Arc::new(Mutex::new("".to_string()));
}

/// Configuration parameters
const URL: &str = env!("ETH_NODE_URL", "Missing Ethereum node URL in environment variables");
const CHAIN_ID: u64 = env!("ETH_CHAIN_ID", "Missing Ethereum chain ID in environment variables")
    .parse()
    .expect("Invalid Ethereum chain ID");

/// Transforms an HTTP response by clearing its headers.
#[query(name = "transform")]
#[candid_method(query, rename = "transform")]
fn transform(response: TransformArgs) -> HttpResponse {
    let mut t = response.response;
    t.headers = vec![];
    t
}

/// Fetches the current gas price from the Ethereum network with error handling and retries.
#[update(name = "get_eth_gas_price")]
#[candid_method(update, rename = "get_eth_gas_price")]
async fn get_eth_gas_price() -> Result<String, String> {
    let w3 = match ICHttp::new(URL, None) {
        Ok(v) => Web3::new(v),
        Err(e) => return Err(e.to_string()),
    };

    for _ in 0..3 {
        match w3.eth().gas_price().await {
            Ok(gas_price) => {
                ic_cdk::println!("Gas price: {}", gas_price);
                return Ok(format!("{} WEI", gas_price));
            }
            Err(e) => {
                ic_cdk::println!("Error fetching gas price: {}", e);
                sleep(Duration::from_secs(1)).await;
            }
        }
    }

    Err("Failed to fetch gas price after multiple attempts".to_string())
}

/// Retrieves the Ethereum address associated with the canister with caching and periodic refresh.
#[update]
#[candid_method(update, rename = "get_eth_address")]
async fn get_eth_address() -> Result<String, String> {
    let cached_address = ADDRESS.lock().unwrap().clone();

    if !cached_address.is_empty() {
        return Ok(cached_address);
    }

    match get_eth_addr(None, None, KEY_NAME.to_string()).await {
        Ok(address) => {
            let formatted_address = format!("0x{}", hex::encode(&address));
            ADDRESS.lock().unwrap().replace(formatted_address.clone());
            Ok(formatted_address)
        }
        Err(e) => Err(format!("Failed to get Ethereum address: {}", e)),
    }
}

/// Retrieves the balance of the Ethereum address in Ether with proper error handling.
#[update(name = "get_eth_balance")]
#[candid_method(update, rename = "get_eth_balance")]
async fn get_eth_balance() -> Result<String, String> {
    let addr = get_eth_address().await?;
    let w3 = match ICHttp::new(URL, None) {
        Ok(v) => Web3::new(v),
        Err(e) => return Err(e.to_string()),
    };

    match w3.eth().balance(Address::from_str(&addr).unwrap(), None).await {
        Ok(balance) => {
            let wei_str: String = balance.to_string();
            let wei: U256 = wei_str.parse().unwrap();
            let eth: f64 = wei.as_u128() as f64 / 1e18;
            Ok(format!("{} ETH", eth))
        }
        Err(e) => Err(format!("Failed to get balance: {}", e)),
    }
}

/// Converts an amount in Ether to Wei with input validation.
#[query(name = "eth_to_wei")]
#[candid_method(query, rename = "eth_to_wei")]
async fn eth_to_wei(eth: f64) -> Result<String, String> {
    if eth < 0.0 {
        return Err("Invalid input: Ethereum amount cannot be negative".to_string());
    }

    let wei = (eth * 1e18) as u64;
    Ok(format!("{}", wei))
}

/// Sends Ether to another Ethereum address with enhanced error handling and retries.
#[update(name = "send_eth_in_ether")]
#[candid_method(update, rename = "send_eth_in_ether")]
async fn send_eth_in_ether(to: String, eth_value: f64, nonce: Option<u64>) -> Result<String, String> {
    let value = (eth_value * 1e18) as u64;
    let derivation_path = vec![ic_cdk::id().as_slice().to_vec()];
    let key_info = KeyInfo {
        derivation_path,
        key_name: KEY_NAME.to_string(),
        ecdsa_sign_cycles: None,
    };

    let from_addr = match get_eth_addr(None, None, KEY_NAME.to_string()).await {
        Ok(addr) => addr,
        Err(e) => return Err(format!("Failed to get canister Ethereum address: {}", e)),
    };

    let w3 = match ICHttp::new(URL, None) {
        Ok(v) => Web3::new(v),
        Err(e) => return Err(e.to_string()),
    };

    let tx_count: U256 = if let Some(count) = nonce {
        count.into()
    } else {
        match w3.eth().transaction_count(from_addr, None).await {
            Ok(v) => v,
            Err(e) => return Err(format!("Failed to get transaction count: {}", e)),
        }
    };

    ic_cdk::println!("Canister Ethereum address {} tx count: {}", hex::encode(&from_addr), tx_count);

    let to = Address::from_str(&to).map_err(|e| format!("Invalid recipient address: {}", e))?;
    let tx = TransactionParameters {
        to: Some(to),
        nonce: Some(tx_count),
        value: U256::from(value),
        gas_price: Some(U256::from(100_000_000_000u64)),
        gas: U256::from(21000),
        ..Default::default()
    };

    let signed_tx = match w3.accounts().sign_transaction(tx, hex::encode(&from_addr), key_info, CHAIN_ID).await {
        Ok(signed_tx) => signed_tx,
        Err(e) => return Err(format!("Failed to sign transaction: {}", e)),
    };

    match w3.eth().send_raw_transaction(signed_tx.raw_transaction).await {
        Ok(txhash) => {
            ic_cdk::println!("Transaction hash: {}", hex::encode(&txhash.0));
            Ok(format!("https://etherscan.io/tx/{}", hex::encode(&txhash.0)))
        }
        Err(e) => Err(format!("Failed to send raw transaction: {}", e)),
    }
}
