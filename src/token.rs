use std::borrow::Borrow;
use std::cell::{Cell, RefCell};
use std::cmp::max;
use std::time::Duration;

use async_cell::sync::AsyncCell;
use reqwest::{Client, ClientBuilder};
use ring::hmac;
use serde::Deserialize;
use serde_json::json;
use tracing::{debug, trace};

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

pub struct TokenManager<'a> {
    cluster_id: &'a str,
    cluster_secret: &'a str,
    token: AsyncCell<Option<String>>,
    reqwest_client: Client,
}

impl<'a> TokenManager<'a> {
    pub fn new(cluster_id: &'a str, cluster_secret: &'a str, base_url: &'a str) -> Self {
        let reqwest_client = ClientBuilder::new()
            .base_url(base_url.to_string())
            .user_agent(USER_AGENT)
            .build()
            .unwrap();

        Self {
            cluster_id,
            cluster_secret,
            token: AsyncCell::new(),
            reqwest_client,
        }
    }

    pub async fn fetch_token(&self) -> Result<String, reqwest::Error> {
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

    async fn schedule_refresh_token(&self, ttl: u64) {
        let sleep_time = max(
            Duration::from_millis(ttl) - Duration::from_secs(600),
            Duration::from_millis(ttl / 2),
        );
        tokio::time::sleep(sleep_time).await;
        self.get_refreshed_token().await.unwrap();

        trace!("Scheduled refresh token in {:?}ms", sleep_time.as_millis());
    }

    #[async_recursion::async_recursion]
    async fn get_refreshed_token(&self) -> Result<(), reqwest::Error> {
        let token = self.token.get().await;
        let token_request_body = json!({
            "clusterId": &self.cluster_id,
            "token": token
        });

        let token_response: TokenResponse = self
            .reqwest_client
            .post("/openbmclapi-agent/token")
            .json(&token_request_body)
            .send()
            .await?
            .json()
            .await?;

        self.token.set(Some(token_response.token));

        debug!("Successfully refreshed token");

        self.schedule_refresh_token(token_response.ttl).await;

        Ok(())
    }
}
