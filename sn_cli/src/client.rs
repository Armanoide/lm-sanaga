use crate::error::{Error, Result};
use reqwest::{Client, Response};

pub struct CliClient {
    client: Client,
    base_url: String,
}

impl CliClient {
    pub fn new(base_url: &str) -> Self {
        let client = Client::new();
        CliClient {
            client,
            base_url: base_url.to_string(),
        }
    }

    pub fn get_client(&self) -> &Client {
        &self.client
    }

    async fn handle_response(
        &self,
        res: std::result::Result<Response, reqwest::Error>,
    ) -> Result<String> {
        match res {
            Ok(res) => {
                let text = res.text().await?;
                Ok(text)
            }
            Err(e) => {
                if e.is_connect() {
                    Err(Error::ConnectionRefused(self.base_url.clone()))
                } else {
                    Err(Error::Http(e))
                }
            }
        }
    }

    pub async fn list_model(&self) -> Result<String> {
        let url = format!("{}/api/v1/models", self.base_url);
        let result = self.client.get(&url).send().await;
        Ok(self.handle_response(result).await?)
    }

    pub async fn ps_model(&self) -> Result<String> {
        let url = format!("{}/api/v1/models/ps", self.base_url);
        let result = self.client.get(&url).send().await;
        Ok(self.handle_response(result).await?)
    }

    pub async fn run_model(&self, name: &String) -> Result<String> {
        let url = format!("{}/api/v1/models/run", self.base_url);
        let result = self
            .client
            .get(&url)
            .json(&serde_json::json!({ "name": name }))
            .send()
            .await;
        Ok(self.handle_response(result).await?)
    }
}
