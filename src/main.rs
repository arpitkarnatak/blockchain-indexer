mod config;
mod indexer;
mod message_queue;

use alloy::sol;
use dotenv::dotenv;
use indexer::Indexer;
use std::error::Error;
use std::fs::File;
use std::sync::Arc;
use tokio::task::JoinHandle;

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
    dotenv().map_err(|err| format!("Environment variable initialization failed. {:?}", err))?;
    // Open the ABI file safely
    let abi_file = File::open("./abi.json")?;

    let indexer = Arc::new(Indexer::new(
        CONTRACT_ADDRESS_USDT, // USDT Address
        22169475,
        22179475,
        "Transfer".to_owned(),
        "Transfer(address,address,uint256)".to_owned(),
        abi_file,
    )?);
    let indexer_clone_1 = indexer.clone();
    let indexer_clone_2 = indexer.clone();

    let handle_event_parsing: JoinHandle<()> = tokio::spawn(async move {
        match indexer_clone_1.event_parser().await {
            Ok(_) => {
                println!("Done")
            }
            Err(e) => {
                eprintln!("Error {:?}", e);
            }
        };
    });

    let backfilling = tokio::spawn(async move {
        match indexer_clone_2.backfill_database().await {
            Ok(_) => {
                println!("Completed Backfill")
            }
            Err(e) => {
                eprintln!("Error {:?}", e);
            }
        };
    });
    handle_event_parsing.await?;
    backfilling.await?;
    Ok(())
}
