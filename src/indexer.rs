use serde_json::Value as JSONValue; // Ensure serde_json is used
use std::error::Error;
use std::fs::File;
use std::io::Read;

#[derive(Debug)] // Implement Debug to print Indexer
pub struct Indexer {
    contract_address: String,
    contract_creation_block_number: i64,
    indexer_created_block_number: i64,
    event_name: String,
    abi: JSONValue,
}

impl Indexer {
    pub fn new(
        contract_address: &str,
        contract_creation_block_number: i64,
        indexer_created_block_number: i64,
        event_name: String,
        mut abi_file: File,
    ) -> Result<Self, Box<dyn Error>> {
        let mut abi_content = String::new();

        // Read the ABI file
        abi_file.read_to_string(&mut abi_content)?;

        // Parse JSON safely
        let abi: JSONValue = serde_json::from_str(&abi_content)?;

        // Return Indexer instance
        Ok(Indexer {
            contract_address: contract_address.to_owned(),
            contract_creation_block_number,
            indexer_created_block_number,
            event_name,
            abi,
        })
    }

    pub fn event_parser() -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
