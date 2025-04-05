use envconfig::Envconfig;
use lazy_static::lazy_static;
use std::error::Error;

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "RPC_URL_WS")]
    pub rpc_url_websocket: String,

    #[envconfig(from = "RPC_URL_HTTP")]
    pub rpc_url_http: String,

    #[envconfig(from = "MESSAGE_QUEUE_URL")]
    pub message_queue_url: String,
}

lazy_static! {
    pub static ref CONFIG: Config = Config::init_from_env().unwrap();
}
