mod cli;
mod config;
mod storage;
mod token;

use cli::parse_cli;
use config::load_config;
use salvo::prelude::*;
use tracing_unwrap::OptionExt;

pub const VERSION: &'static str = include_str!(concat!(env!("OUT_DIR"), "/VERSION"));
pub const USER_AGENT: &'static str = concat!(
    "openbmclapi-cluster/",
    include_str!(concat!(env!("OUT_DIR"), "/PKG_VERSION"))
);

#[handler]
async fn hello() -> &'static str {
    "Hello World"
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let cli = parse_cli();
    let config = load_config(cli.config).expect_or_log("failed to load config file");

    let router = Router::new().get(hello);
    let acceptor = TcpListener::new("127.0.0.1:5800").bind().await;
    Server::new(acceptor).serve(router).await;
}
