use std::time::Duration;

use reqwest::{Client, ClientBuilder};
use ring::hmac;
use serde::Deserialize;
use serde_json::json;
use tracing::{debug, trace};
use tracing_unwrap::ResultExt;

use crate::USER_AGENT;

#[derive(Deserialize)]
struct ChallengeResponse {
    challenge: String,
}

#[derive(Deserialize)]
struct TokenResponse {
    token: String,
    ttl: u64,
}

pub struct TokenManager {
    cluster_id: String,
    cluster_secret: String,
    token: Option<String>,
    reqwest_client: Client,
}

impl TokenManager {
    pub fn new(cluster_id: String, cluster_secret: String, base_url: String) -> Self {
        let reqwest_client = ClientBuilder::new()
            .base_url(base_url)
            .user_agent(USER_AGENT)
            .build()
            .unwrap_or_log();

        TokenManager {
            cluster_id,
            cluster_secret,
            token: None,
            reqwest_client,
        }
    }

    pub async fn fetch_token(&mut self) -> Result<String, reqwest::Error> {
        let challenge_response: ChallengeResponse = self
            .reqwest_client
            .get("/openbmclapi-agent/challenge")
            .query(&[("clusterId", &self.cluster_id)])
            .send()
            .await?
            .json()
            .await?;

        let key = hmac::Key::new(hmac::HMAC_SHA256, &self.cluster_secret.as_bytes());
        let tag = hmac::sign(&key, challenge_response.challenge.as_bytes());
        let signature = hex::encode(tag.as_ref());
        let token_request_body = json!({
            "clusterId": self.cluster_id,
            "challenge": challenge_response.challenge,
            "signature": signature,
        });
        let token_response: TokenResponse = self
            .reqwest_client
            .post("/openbmclapi-agent/token")
            .json(&token_request_body)
            .send()
            .await?
            .json()
            .await?;

        self.schedule_refresh_token(token_response.ttl).await;

        Ok(token_response.token)
    }

    async fn schedule_refresh_token(&mut self, ttl: u64) {
        let sleep_time = max(
            Duration::from_millis(ttl) - Duration::from_secs(600),
            Duration::from_millis(ttl / 2),
        );
        tokio::time::sleep(sleep_time).await;
        self.get_refreshed_token().await.unwrap_or_log();

        trace!("Scheduled refresh token in {:?}ms", sleep_time.as_millis());
    }

    #[async_recursion::async_recursion]
    async fn get_refreshed_token(&mut self) -> Result<(), reqwest::Error> {
        let token_request_body = json!({
            "clusterId": &self.cluster_id,
            "token": self.token.as_ref()
        });

        let token_response: TokenResponse = self
            .reqwest_client
            .post("/openbmclapi-agent/token")
            .json(&token_request_body)
            .send()
            .await?
            .json()
            .await?;

        self.token = Some(token_response.token);

        debug!("Successfully refreshed token");

        self.schedule_refresh_token(token_response.ttl).await;

        Ok(())
    }
}
use std::cmp::max;
