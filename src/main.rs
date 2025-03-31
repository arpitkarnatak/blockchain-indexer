mod config;
mod indexer;

use alloy::sol;
use config::Config;
use dotenv::dotenv;
use envconfig::Envconfig;
use indexer::Indexer;
use std::error::Error;
use std::fs::File;

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

    let indexer = Indexer::new(
        CONTRACT_ADDRESS_USDT, // USDT Address
        4634748,
        4634748,
        "Transfer".to_owned(),
        "Transfer(address,address,uint256)".to_owned(),
        abi_file,
    )?;
    indexer.backfill_database().await?;
    indexer.event_parser().await?;
    Ok(())
}
