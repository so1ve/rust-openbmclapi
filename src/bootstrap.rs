use tracing::{error, info};

use crate::config::Config;
use crate::token::TokenManager;
use crate::PKG_VERSION;

pub async fn bootstrap(config: &Config) {
    info!("Booting {PKG_VERSION}");
    let token_manager =
        TokenManager::new(&config.cluster_id, &config.cluster_secret, &config.bmclapi);
    if let Err(err) = token_manager.fetch_token().await {
        error!("Failed to fetch token: {}", err);
    };
}
