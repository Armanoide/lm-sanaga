use crate::error::{ErrorCli, Result};
use reqwest::{Client, Response};
use sn_core::server::payload::backend::create_session_request::CreateSessionRequest;
use sn_core::server::payload::backend::generate_text_request::GenerateTextRequest;
use sn_core::server::payload::backend::run_model_request::RunModelRequest;
use sn_core::server::routes::{
    BackendApiMessage, BackendApiModel, BackendApiSession, BackendConversationApi,
};

pub struct CliClient {
    client: Client,
    base_url: String,
    base_url_api: String,
}

impl CliClient {
    pub fn new(base_url: &str) -> Self {
        let client = Client::new();
        CliClient {
            client,
            base_url: base_url.to_string(),
            base_url_api: format!("{}{}", base_url.to_string(), "/api"),
        }
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
                    Err(ErrorCli::ConnectionRefused(self.base_url_api.clone()))
                } else {
                    Err(ErrorCli::Http(e))
                }
            }
        }
    }

    pub async fn list_model(&self) -> Result<String> {
        let url = format!(
            "{}{}",
            self.base_url_api,
            BackendApiModel::List.path().as_str()
        );
        let result = self.client.get(&url).send().await;
        Ok(self.handle_response(result).await?)
    }

    pub async fn list_conversation(&self, session_id: &i32) -> Result<String> {
        let url = format!(
            "{}{}",
            self.base_url_api,
            BackendConversationApi::List.path(Some(session_id)).as_str()
        );
        let result = self.client.get(&url).send().await;
        Ok(self.handle_response(result).await?)
    }

    pub async fn ps_model(&self) -> Result<String> {
        let url = format!(
            "{}{}",
            self.base_url_api,
            BackendApiModel::ListRunning.path().as_str()
        );
        let result = self.client.get(&url).send().await;
        Ok(self.handle_response(result).await?)
    }

    pub async fn run_model(&self, json: &RunModelRequest) -> Result<Response> {
        let url = format!(
            "{}{}",
            self.base_url_api,
            BackendApiModel::Run.path().as_str()
        );
        let result = self
            .client
            .post(&url)
            .json(&serde_json::json!(json))
            .send()
            .await?
            .error_for_status()?;
        Ok(result)
    }
    pub async fn send_prompt(&self, json: &GenerateTextRequest) -> Result<Response> {
        let url = format!(
            "{}{}",
            self.base_url_api,
            BackendApiMessage::Generate.path().as_str()
        );
        let result = self
            .client
            .post(&url)
            .json(&serde_json::json!(json))
            .send()
            .await?
            .error_for_status()?;

        Ok(result)
    }

    pub async fn stop_model(&self, model_id: &String) -> Result<String> {
        let url = format!(
            "{}{}",
            self.base_url_api,
            BackendApiModel::Stop.path().as_str()
        );
        let result = self
            .client
            .post(&url)
            .json(&serde_json::json!({ "id": model_id }))
            .send()
            .await;
        Ok(self.handle_response(result).await?)
    }

    pub async fn create_session(&self, request: &CreateSessionRequest) -> Result<String> {
        let url = format!(
            "{}{}",
            self.base_url_api,
            BackendApiSession::Create.path().as_str()
        );
        let result = self
            .client
            .post(&url)
            .json(&serde_json::json!(request))
            .send()
            .await;
        Ok(self.handle_response(result).await?)
    }
}
