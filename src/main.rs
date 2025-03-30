mod config;
mod eth_adapter;
mod indexer;

use config::Config;
use dotenv::dotenv;
use envconfig::Envconfig;
use eth_adapter::EthAdapter;
use indexer::Indexer;
use std::env;
use std::error::Error;
use std::str::FromStr;
use std::{fs::File, future};
use web3::ethabi::{LogParam, ParseLog, RawLog};
use web3::signing::keccak256;
use web3::types::{Transaction, TransactionReceipt, H160, H256};
use web3::{futures::StreamExt, types::FilterBuilder};

const CONTRACT_ADDRESS_USDT: &str = "0xdAC17F958D2ee523a2206206994597C13D831ec7";
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

    let eth_adapter = EthAdapter::new(config.rpc_url_websocket).await?;
    //println!("Indexer: {:?} {:?}", indexer, eth_adapter);

    let event_signature = "Transfer(address,address,uint256)";
    let filter = FilterBuilder::default()
        .address(vec![CONTRACT_ADDRESS_USDT.parse().unwrap()])
        .topics(
            Some(vec![H256::from_slice(&keccak256(
                event_signature.as_bytes(),
            ))]),
            None,
            None,
            None,
        )
        .build();

    let mut sub = eth_adapter
        .web3
        .eth_subscribe()
        .subscribe_logs(filter.to_owned())
        .await?;

    println!("Got subscription id: {:?}", sub.id());

    (&mut sub)
    .take(1) // Remove to keep running
    .for_each(|log| {
        let eth_adapter: EthAdapter = eth_adapter.clone();
        async move {
            if let Ok(event_data) = log {
                println!("Got: {:?}", event_data.transaction_hash);
                if let Some(tx_hash) = event_data.transaction_hash {
                    match eth_adapter.web3.eth().transaction_receipt(tx_hash).await {
                        Ok(Some(tx_receipt)) => println!("Transaction Receipt {:?}", tx_receipt.logs.iter().for_each(|log| {
                            if log.address == H160::from_str(CONTRACT_ADDRESS_USDT).unwrap() {
                                println!("This is the relevant log {:?}", log);
                            }
                        })),
                        Ok(None) => println!("Transaction receipt not found"),
                        Err(err) => eprintln!("Error fetching transaction receipt: {:?}", err),
                    }
                }
            }
        }
    })
    .await;

    sub.unsubscribe().await?;
    println!("Filter was {:?}", filter);
    Ok(())
}
