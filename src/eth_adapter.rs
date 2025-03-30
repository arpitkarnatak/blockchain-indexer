use std::error::Error;
use web3::{
    Web3,
    transports::{self, WebSocket},
};

#[derive(Debug, Clone)]
pub struct EthAdapter {
    pub web3: Web3<WebSocket>,
}

impl EthAdapter {
    pub async fn new(rpc_url: String) -> Result<Self, Box<dyn Error>> {
        let transport = transports::WebSocket::new(&rpc_url).await?;
        let web3 = web3::Web3::new(transport);
        Ok(EthAdapter { web3 })
    }
}
