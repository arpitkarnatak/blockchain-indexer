use std::fs::File;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Indexer {
    contract_address: String,
    contract_creation_block_number: i64,
    indexer_created_block_number: i64,
    event_name: String,
    abi: File
}


impl Indexer {
    pub fn new(contract_address: String, contract_creation_block_number: i64, indexer_created_block_number: i64, event_name: String, abi: File) -> Self {
        Indexer { contract_address, contract_creation_block_number, indexer_created_block_number, event_name, abi }
    }
}
