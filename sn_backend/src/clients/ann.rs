use std::env;

use crate::error::{ErrorBackend, Result};
use reqwest::{Client, Response};
use sn_core::server::defauft_config::{
    DEFAULT_SERVER_ANN_HOST, DEFAULT_SERVER_ANN_PORT, DEFAULT_SERVER_ANN_PROTOCOL,
};
use sn_core::server::payload::ann::partition_status_response::PartitionStatusResponse;
use sn_core::server::payload::ann::search_request::SearchRequest;
use sn_core::server::payload::ann::search_response::SearchResponse;
use sn_core::types::ann_item::AnnItem;
#[derive(Debug, Clone)]
pub struct AnnClient {
    client: Client,
    base_url: String,
}

impl AnnClient {
    pub fn new() -> Self {
        let host = env::var("SERVER_ANN_HOST").unwrap_or(String::from(DEFAULT_SERVER_ANN_HOST));
        let port = env::var("SERVER_ANN_PORT").unwrap_or(String::from(DEFAULT_SERVER_ANN_PORT));
        let protocol =
            env::var("SERVER_ANN_PROTOCOL").unwrap_or(String::from(DEFAULT_SERVER_ANN_PROTOCOL));
        let client = Client::new();
        let base_url = format!("{protocol}://{host}:{port}");

        AnnClient { client, base_url }
    }

    pub fn get_client(&self) -> &Client {
        &self.client
    }

    async fn handle_response(
        &self,
        res: std::result::Result<Response, reqwest::Error>,
    ) -> Result<String> {
        match res?.error_for_status() {
            Ok(res) => {
                let text = res.text().await?;
                Ok(text)
            }
            Err(e) => {
                if e.is_connect() {
                    Err(ErrorBackend::ConnectionRefused(self.base_url.clone()))
                } else {
                    Err(ErrorBackend::Http(e))
                }
            }
        }
    }

    pub async fn ping(&self) -> Result<String> {
        let url = format!("{}/api/ping", self.base_url);
        let result = self.client.get(&url).send().await;
        Ok(self.handle_response(result).await?)
    }

    pub async fn get_partition_status(&self, partition_id: i32) -> Result<PartitionStatusResponse> {
        let url = format!("{}/api/partition/{}/status", self.base_url, partition_id);
        let result = self.client.get(&url).send().await;
        let result = self.handle_response(result).await?;
        let result = serde_json::from_str::<PartitionStatusResponse>(&result)?;
        Ok(result)
    }

    pub async fn embedding_insert_bulk(&self, items: Vec<AnnItem>) -> Result<String> {
        let url = format!("{}/api/embedding/insert_bulk", self.base_url);
        let result = self.client.post(&url).json(&items).send().await;
        Ok(self.handle_response(result).await?)
    }

    pub async fn embedding_insert(&self, item: AnnItem) -> Result<String> {
        let url = format!("{}/api/embedding/insert", self.base_url);
        let result = self.client.post(&url).json(&item).send().await;
        Ok(self.handle_response(result).await?)
    }

    pub async fn search_similarity(&self, payload: SearchRequest) -> Result<SearchResponse> {
        let url = format!("{}/api/embedding/search", self.base_url);
        let result = self.client.post(&url).json(&payload).send().await;
        let result = self.handle_response(result).await?;
        let result = serde_json::from_str::<SearchResponse>(&result)?;
        Ok(result)
    }
}
