mod bootstrap;
mod cli;
mod config;
mod storage;
mod token;
mod utils;

use anyhow::Result;
use bootstrap::bootstrap;
use cli::parse_cli;
use config::load_config;
use const_format::concatcp;
use salvo::prelude::*;
use tracing::error;

pub const VERSION: &'static str = include_str!(concat!(env!("OUT_DIR"), "/VERSION"));
pub const PKG_VERSION: &'static str = include_str!(concat!(env!("OUT_DIR"), "/PKG_VERSION"));
pub const USER_AGENT: &'static str = concatcp!("openbmclapi-cluster/", PKG_VERSION);

#[handler]
async fn hello() -> &'static str {
    "Hello World"
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = parse_cli();
    let config = match load_config(cli.config) {
        Ok(config) => config,
        Err(err) => {
            error!("Failed to load config: {}", err);
            return Err(err);
        }
    };

    bootstrap(&config).await;

    // let mut alist_storage = get_storage(config.storage[0].clone());

    // let res = alist_storage
    //     .check_missing_files(vec![BMCLAPIFile {
    //         path: "".into(),
    //         hash: "114".into(),
    //         size: 1,
    //         mtime: 0,
    //     }])
    //     .await?;

    // println!("{:?}", res);

    Ok(())
}
