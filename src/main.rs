mod config;
mod eth_adapter;
mod indexer;

use alloy::primitives::Address;
use alloy::providers::Provider;
use alloy::rpc::types::Filter;
use alloy::sol;
use config::Config;
use dotenv::dotenv;
use envconfig::Envconfig;
use eth_adapter::EthAdapter;
use indexer::Indexer;
use std::error::Error;
use std::str::FromStr;
use std::{fs::File};
use futures_util::stream::StreamExt;

// Codegen from ABI file to interact with the contract.

const CONTRACT_ADDRESS_USDT: &str = "0xdAC17F958D2ee523a2206206994597C13D831ec7";
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    USDT_CONTRACT,
    "./abi.json"
);
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    //
    dotenv()?;
    let config = Config::init_from_env()?;

    // Open the ABI file safely
    let abi_file = File::open("./abi.json")?;
    // Initialize the Indexer
    let indexer = Indexer::new(
        CONTRACT_ADDRESS_USDT, // USDT Address
        4634748,
        4634748,
        "Transfer".to_owned(),
        abi_file,
    )?;

    let eth_adapter = EthAdapter::new(config.rpc_url_websocket).await.unwrap();

    let event_signature_string = "Transfer(address,address,uint256)";

    let filter = Filter::new()
        .address(Address::from_str(CONTRACT_ADDRESS_USDT)?)
        .event(event_signature_string);

    let subscription = eth_adapter.provider.subscribe_logs(&filter).await?;
    let mut stream = subscription.into_stream();
    let log = stream.next().await.unwrap();
    let event_object = log.log_decode::<USDT_CONTRACT::Transfer>()?.inner.data;
    println!(
        "{:?} --> {:?} Amt: {:?}",
        &event_object.from, &event_object.to, &event_object.value
    );

    Ok(())
}
