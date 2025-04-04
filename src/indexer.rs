use alloy::primitives::{Address, BlockNumber};
use alloy::providers::{Provider, WsConnect};
use alloy::rpc::types::Filter;
use alloy::{providers::ProviderBuilder, transports::http::reqwest::Url};
use envconfig::Envconfig;
use futures_util::StreamExt;
use serde_json::{Value as JSONValue, json};
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::str::FromStr;

use crate::config::{self, Config};
use crate::message_queue::MessageQueue;
use crate::{CONTRACT_ADDRESS_USDT, USDT_CONTRACT};

#[derive(Debug)] // Implement Debug to print Indexer
pub struct Indexer {
    contract_address: String,
    contract_creation_block_number: i64,
    indexer_created_block_number: i64,
    event_name: String,
    event_signature: String,
    abi: JSONValue,
}

impl Indexer {
    pub fn new(
        contract_address: &str,
        contract_creation_block_number: i64,
        indexer_created_block_number: i64,
        event_name: String,
        event_signature: String,
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
            event_signature,
            abi,
        })
    }

    // Get the transactions from the contract creation upto the block number from when the indexer was created
    // The block after indexer_created_block_number would be picked up in real time as they get appended to blockchain
    pub async fn backfill_database(&self) -> Result<(), Box<dyn Error>> {
        let mut start_block = self.contract_creation_block_number;
        let mut jump = 1;
        let mut end_block = start_block + jump;

        let http_rpc = Config::init_from_env()?.rpc_url_http;

        let http_provider = ProviderBuilder::new().on_http(Url::parse(&http_rpc)?);

        while start_block <= self.indexer_created_block_number {
            // Double the gap every iteration, once it fails, return to gap of 1
            let filter = Filter::new()
                .address(Address::from_str(&self.contract_address)?)
                .from_block(BlockNumber::from(start_block as u64))
                .to_block(BlockNumber::from(end_block as u64))
                .event(&self.event_signature);
            println!("From blocks {} to {}", start_block, end_block);

            let logs = http_provider.get_logs(&filter).await;
            match logs {
                Ok(logs) => {
                    logs.iter().for_each(|log| {
                        println!("[HTTP] ======= Log incoming ==========");
                        let event_object = log
                            .log_decode::<USDT_CONTRACT::Transfer>()
                            .unwrap()
                            .inner
                            .data;
                        println!("[HTTP] {:?}", &event_object);
                    });
                    jump *= 2;
                    start_block = end_block;
                    end_block = end_block + jump;
                }
                Err(err) => {
                    eprintln!("Error {}", err);
                    jump = 1;
                }
            }
        }
        Ok(())
    }

    // This function listens to events in real time
    pub async fn event_parser(&self) -> Result<(), Box<dyn Error>> {
        let provider = ProviderBuilder::new()
            .on_ws(WsConnect {
                url: Config::init_from_env()?.rpc_url_websocket.clone(),
                auth: None,
                config: None,
            })
            .await
            .unwrap();

        let eth_queue = MessageQueue::new("eth_events").await?;

        let filter = Filter::new()
            .address(Address::from_str(CONTRACT_ADDRESS_USDT)?)
            .event(&self.event_signature);

        let subscription = provider.subscribe_logs(&filter).await?;
        let mut stream = subscription.into_stream();
        while let Some(log) = stream.next().await {
            let event_object = log.log_decode::<USDT_CONTRACT::Transfer>()?.inner.data;
            println!("[WS] {:?} {:?}", &event_object, &log);
        
            let mut event_log = serde_json::json!(log); // Remove & to make it mutable
        
            if let JSONValue::Object(ref mut map) = event_log {
                map.remove("inner"); // Omitting "inner"
                map.remove("data"); // Omitting "data"
                map.remove("topics");
                map.insert("event_data".to_string(), serde_json::json!(event_object));
            }
            eth_queue
                .publish_message(&serde_json::to_string(&event_log).unwrap())
                .await?;
        }

        Ok(())
    }
}

// { inner: Log { address: 0xdac17f958d2ee523a2206206994597c13d831ec7, data: LogData { topics: [0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef, 0x000000000000000000000000ae8a453eb0d22c098cf938429a3abda6d6546411, 0x000000000000000000000000a82559ee217eb35839790275bfb034f5a2885c58], data: 0x000000000000000000000000000000000000000000000000000000012db23120 } }, block_hash: Some(0xfb6de7f4729adca281f508972fb2e5418ec92eeb94e336620cb61c13d93766f9), block_number: Some(22179665), block_timestamp: None, transaction_hash: Some(0x9e9e1bd436776c160ac100363915759f50cafcf132f24792bf3118cc42faed05), transaction_index: Some(125), log_index: Some(754), removed: false
