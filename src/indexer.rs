use alloy::primitives::{Address, BlockNumber};
use alloy::providers::{Provider, WsConnect};
use alloy::rpc::types::Filter;
use alloy::{providers::ProviderBuilder, transports::http::reqwest::Url};
use envconfig::Envconfig;
use futures_util::StreamExt;
use serde_json::Value as JSONValue; // Ensure serde_json is used
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::str::FromStr;

use crate::config::{self, Config};
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
                        println!(
                            "[HTTP] {:?} ---> {:?} [{:?}]",
                            event_object.from, event_object.to, event_object.value
                        );
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
        let filter = Filter::new()
            .address(Address::from_str(CONTRACT_ADDRESS_USDT)?)
            .event(&self.event_signature);

        let subscription = provider.subscribe_logs(&filter).await?;
        let mut stream = subscription.into_stream();
        let log = stream.next().await.unwrap();
        let event_object = log.log_decode::<USDT_CONTRACT::Transfer>()?.inner.data;
        println!(
            "[WS] {:?} --> {:?} Amt: {:?}",
            &event_object.from, &event_object.to, &event_object.value
        );
        Ok(())
    }
}
