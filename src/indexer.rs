use alloy::primitives::{Address, BlockNumber};
use alloy::providers::{Provider, WsConnect};
use alloy::rpc::types::Filter;
use alloy::{providers::ProviderBuilder, transports::http::reqwest::Url};
use futures_util::StreamExt;
use serde_json::Value as JSONValue;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::str::FromStr;

use crate::config::{CONFIG, Config};
use crate::message_queue::MessageQueue;
use crate::{USDT_CONTRACT};

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

        let http_provider = ProviderBuilder::new().on_http(
            Url::parse(&CONFIG.rpc_url_http)
                .map_err(|err| format!("Error Configuring HTTP RPC {:?}", err))?,
        );

        while start_block <= self.indexer_created_block_number {
            // Double the gap every iteration, once it fails, return to gap of 1
            let filter = Filter::new()
                .address(Address::from_str(&self.contract_address)?)
                .from_block(BlockNumber::from(start_block as u64))
                .to_block(BlockNumber::from(end_block as u64))
                .event(&self.event_signature);
            println!("From blocks {} to {}", start_block, end_block);

            let logs = http_provider
                .get_logs(&filter)
                .await
                .map_err(|err| format!("Error while getting logs {:?}", err))?;

            logs.iter().for_each(|log| {
                println!("[HTTP] ======= Log incoming ==========");
                let event_object = log
                    .log_decode::<USDT_CONTRACT::Transfer>()
                    .map_err(|err| format!("Error decoding logs {:?}", err))
                    .unwrap()
                    .inner
                    .data;
                println!("[HTTP] {:?}", &event_object);
            });
            jump *= 2;
            start_block = end_block;
            end_block = end_block + jump;
        }
        Ok(())
    }

    // This function listens to events in real time
    pub async fn event_parser(&self) -> Result<(), Box<dyn Error>> {
        let provider = ProviderBuilder::new()
            .on_ws(WsConnect {
                url: CONFIG.rpc_url_websocket.clone(),
                auth: None,
                config: None,
            })
            .await
            .map_err(|err| format!("Websocket RPC Connection failed to establish: {:?}", err))?;

        let eth_queue = MessageQueue::new("eth_events")
            .await
            .map_err(|err| format!("Failed to instantiate message queue {:?}", err))?;

        let filter = Filter::new()
            .address(Address::from_str(&self.contract_address)?)
            .event(&self.event_signature);

        let subscription = provider
            .subscribe_logs(&filter)
            .await
            .map_err(|err| format!("Error creating subscription {:?}", err))?;

        println!("[WS] ======= Log incoming ==========");

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
                .publish_message(&serde_json::to_string(&event_log)?)
                .await
                .map_err(|err| {
                    format!(
                        "Failed to publish the message: {:?} due to error {:?}",
                        &serde_json::to_string(&event_log),
                        err
                    )
                })?;
        }

        Ok(())
    }
}
