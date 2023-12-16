//! # Ethereum Interactions in Rust
//!
//! This module provides functionality for interacting with the Ethereum blockchain.
//! It includes features such as transforming HTTP responses, retrieving Ethereum gas prices,
//! obtaining Ethereum addresses and balances, converting Ethereum to Wei (the smallest denomination of Ether),
//! and sending Ether in transactions. This module uses the `ic_web3` crate, which is a Rust library for interacting with Ethereum.

use candid::candid_method;
use ic_cdk::api::management_canister::http_request::{HttpResponse, TransformArgs};
use ic_cdk_macros::{self, update, query};
use std::str::FromStr;

use ic_web3::transports::ICHttp;
use ic_web3::Web3;
use ic_web3::ic::{get_eth_addr, KeyInfo};
use ic_web3::{
    ethabi::ethereum_types::U256,
    types::{Address, TransactionParameters},
};
use std::cell::RefCell;

thread_local! {
    static ADDRESS : RefCell<String> = RefCell::new("".to_string());
}

/// The HTTP URL of an Ethereum node.
const URL: &str = "https://eth-sepolia.g.alchemy.com/v2/YPe0Rex7dk40_XbWsn0pdm4UysevzMtq";
/// The unique identifier for the Ethereum network being used.
const CHAIN_ID: u64 = 11155111;
/// A string constant representing the name of a key, used for cryptographic operations.
const KEY_NAME: &str = "dfx_test_key";

/// Transforms an HTTP response by clearing its headers.
#[query(name = "transform")]
#[candid_method(query, rename = "transform")]
fn transform(response: TransformArgs) -> HttpResponse {
    let mut t = response.response;
    t.headers = vec![];
    t 
}

/// Fetches the current gas price from the Ethereum network.
/// Gas is a unit of computational effort on Ethereum, and knowing the gas price is crucial for transaction cost estimation.
#[update(name = "get_eth_gas_price")]
#[candid_method(update, rename = "get_eth_gas_price")]
async fn get_eth_gas_price() -> Result<String, String> {
    let w3 = match ICHttp::new(URL, None) {
        Ok(v) => { Web3::new(v) },
        Err(e) => { return Err(e.to_string()) },
    };
    let gas_price = w3.eth().gas_price().await.map_err(|e| format!("get gas price failed: {}", e))?;
    ic_cdk::println!("gas price: {}", gas_price);
    Ok(format!("{} WEI", gas_price))
}

/// Retrieves the Ethereum address associated with the canister.
/// If the address is already stored in `ADDRESS`, it returns that; otherwise, it fetches a new address and stores it.
#[update]
#[candid_method(update,rename="get_eth_address")]
async fn get_eth_address() -> Result<String,String> {
    if ADDRESS.with(|addr| { addr.borrow().len() > 0 }) {
        return Ok(ADDRESS.with(|addr| { addr.borrow().clone() }));
    }
    let address =  match get_eth_addr(None, None, KEY_NAME.to_string()).await {
        Ok(addr) => { addr },
        Err(e) => { return Err(e) },
    };
    ADDRESS.with(|addr| {
        *addr.borrow_mut() = format!("0x{}",hex::encode(address));
    });
    Ok(format!("0x{}",hex::encode(address)))

}

/// Retrieves the balance of the Ethereum address in Ether.
/// It converts the balance from Wei to Ether for readability.
#[update(name = "get_eth_balance")]
#[candid_method(update, rename = "get_eth_balance")]
async fn get_eth_balance() -> Result<String, String> {
    let addr = get_eth_address().await.map_err(|e| format!("get eth address failed: {}", e))?;
    let w3 = match ICHttp::new(URL, None) {
        Ok(v) => { Web3::new(v) },
        Err(e) => { return Err(e.to_string()) },
    };
    let balance = w3.eth().balance(Address::from_str(&addr).unwrap(), None).await.map_err(|e| format!("get balance failed: {}", e))?;
    let wei_str: String = balance.to_string();
    let eth: f64 = (wei_str.parse::<f64>().unwrap()) / 1e18;
    Ok(format!("{} ETH", eth ))
}

/// Converts an amount in Ether (a floating-point number) to Wei (an integer).
/// Since Ethereum transactions are calculated in Wei, this conversion is essential.
#[query(name="eth_to_wei")]
#[candid_method(query, rename = "eth_to_wei")]
async fn eth_to_wei(eth: f64) -> Result<String, String> {
    let wei = eth * 1e18;
    Ok(format!("{}",wei as u64))
}

/// Sends Ether to another Ethereum address.
/// It constructs and signs a transaction, then sends it to the Ethereum network.
/// This function demonstrates how to create and send transactions on Ethereum.
#[update(name = "send_eth_in_ether")]
#[candid_method(update, rename = "send_eth_in_ether")]
async fn send_eth_in_ether(to: String, eth_value: f64, nonce: Option<u64>) -> Result<String, String> {
    if eth_value <= f64::from(0){
        return Err(format!("value={} can only be a positive number", eth_value))
    }
    let value = (eth_value * 1e18) as u64;
    let to = Address::from_str(&to).map_err(|e| format!("to='{}' is not a valid ethereum address. Error={}", to, e))?;
    let derivation_path = vec![ic_cdk::id().as_slice().to_vec()];
    let key_info = KeyInfo{ derivation_path: derivation_path, key_name: KEY_NAME.to_string(), ecdsa_sign_cycles: None };
    let from_addr = get_eth_addr(None, None, KEY_NAME.to_string())
        .await
        .map_err(|e| format!("get canister eth addr failed: {}", e))?;
    let w3 = match ICHttp::new(URL, None) {
        Ok(v) => { Web3::new(v) },
        Err(e) => { return Err(e.to_string()) },
    };
    let tx_count: U256 = if let Some(count) = nonce {
        count.into() 
    } else {
        let v = w3.eth()
            .transaction_count(from_addr, None)
            .await
            .map_err(|e| format!("get tx count error: {}", e))?;
        v
    };
        
    ic_cdk::println!("canister eth address {} tx count: {}", hex::encode(from_addr), tx_count);
    let tx = TransactionParameters {
        to: Some(to),
        nonce: Some(tx_count),
        value: U256::from(value),
        gas_price: Some(U256::from(100_000_000_000u64)),
        gas: U256::from(21000),
        ..Default::default()
    };
    let signed_tx = w3.accounts()
        .sign_transaction(tx, hex::encode(from_addr), key_info, CHAIN_ID)
        .await
        .map_err(|e| format!("sign tx error: {}", e))?;
    match w3.eth().send_raw_transaction(signed_tx.raw_transaction).await {
        Ok(txhash) => { 
            ic_cdk::println!("txhash: 0x{}", hex::encode(txhash.0));
            Ok(format!("https://sepolia.etherscan.io/tx/0x{}", hex::encode(txhash.0)))
        },
        Err(e) => { Err(format!("Error:{}", e)) },
    }
    
}

// need this to generate candid
ic_cdk::export_candid!();