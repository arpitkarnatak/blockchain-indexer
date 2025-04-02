mod config;
mod indexer;
mod message_queue;

use alloy::sol;
use config::Config;
use dotenv::dotenv;
use envconfig::Envconfig;
use indexer::Indexer;
use message_queue::MessageQueue;
use std::error::Error;
use std::fs::File;

// Codegen from ABI file to interact with the contract.

const CONTRACT_ADDRESS_USDT: &str = "0xdAC17F958D2ee523a2206206994597C13D831ec7";
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    #[derive(Debug, serde::Serialize)]
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

    let indexer = Indexer::new(
        CONTRACT_ADDRESS_USDT, // USDT Address
        22169475,
        22169477,
        "Transfer".to_owned(),
        "Transfer(address,address,uint256)".to_owned(),
        abi_file,
    )?;

    // let consumer_handle = tokio::spawn(async {
    //     let eth_queue = MessageQueue::new("eth_events").await.unwrap();
    //     println!("Consumer started");
    //     eth_queue.consume_message().await.unwrap();
    // });

    // Spawn a task for the event parser
    let parser_handle = tokio::spawn(async move {
        indexer.event_parser().await.unwrap();
    });

    // Wait for both tasks to complete (which they won't since they run indefinitely)
    tokio::select! {
        // res = consumer_handle => {
        //     if let Err(e) = res {
        //         eprintln!("Consumer task failed: {}", e);
        //     }
        // }
        res = parser_handle => {
            if let Err(e) = res {
                eprintln!("Parser task failed: {}", e);
            }
        }
    }
    Ok(())
}
