use envconfig::Envconfig;

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "RPC_URL_WS")]
    pub rpc_url_websocket: String,

    #[envconfig(from = "RPC_URL_HTTP")]
    pub rpc_url_http: String,
}
