use alloy::providers::{
    Identity, ProviderBuilder, RootProvider, WsConnect,
    fillers::{BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller},
};
use std::error::Error;

#[derive(Debug, Clone)]
pub struct EthAdapter {
    pub provider: FillProvider<
        JoinFill<
            Identity,
            JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>,
        >,
        RootProvider,
    >,
}

impl EthAdapter {
    pub async fn new(rpc_url: String) -> Result<Self, Box<dyn Error>> {
        let websocket_provider = ProviderBuilder::new()
            .on_ws(WsConnect {
                url: rpc_url.clone(),
                auth: None,
                config: None,
            })
            .await
            .unwrap();

        Ok(EthAdapter {
            provider: websocket_provider,
        })
    }
}
