mod indexer; 

use std::fs::File;
use std::error::Error;
use indexer::Indexer;

fn main() -> Result<(), Box<dyn Error>> {
    // Open the ABI file safely
    let abi_file = File::open("./abi.json")?;
    
    // Initialize the Indexer
    let indexer = Indexer::new(
        "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_owned(),  // USDT Address
        4634748, 
        4634748, 
        "Transfer".to_owned(), 
        abi_file
    )?;

    println!("Indexer: {:?}", indexer);
    Ok(())
}