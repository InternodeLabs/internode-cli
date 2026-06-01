use reqwest::Client;
use serde_json::Value;

use crate::config::{load_config, require_api_key, Config};
use crate::error::CliError;

pub struct ApiClient {
    http: Client,
    pub api_url: String,
    api_key: String,
}

impl ApiClient {
    pub fn new() -> Result<Self, CliError> {
        let config = load_config();
        require_api_key(&config)?;
        Self::from_config(config)
    }

    pub fn from_config(config: Config) -> Result<Self, CliError> {
        // 15-minute hard cap so `embeddings sync --timeout 900` can still
        // round-trip; the server-side timeout (default 120s, max 900s) is the
        // real bound — this just makes sure the client doesn't drop the
        // connection before the server has finished.
        let http = Client::builder()
            .timeout(std::time::Duration::from_secs(900))
            .build()
            .map_err(|e| CliError::Internal(format!("HTTP client init: {e}")))?;
        Ok(Self {
            http,
            api_url: config.api_url,
            api_key: config.api_key,
        })
    }

    pub async fn get(&self, path: &str) -> Result<Value, CliError> {
        let url = format!("{}{path}", self.api_url);
        let resp = self.http.get(&url).header("X-CLI-API-Key", &self.api_key).send().await?;
        Self::parse_response(resp).await
    }

    pub async fn post(&self, path: &str, body: &Value) -> Result<Value, CliError> {
        let url = format!("{}{path}", self.api_url);
        let resp = self
            .http
            .post(&url)
            .header("X-CLI-API-Key", &self.api_key)
            .json(body)
            .send()
            .await?;
        Self::parse_response(resp).await
    }

    pub async fn patch(&self, path: &str, body: &Value) -> Result<Value, CliError> {
        let url = format!("{}{path}", self.api_url);
        let resp = self
            .http
            .patch(&url)
            .header("X-CLI-API-Key", &self.api_key)
            .json(body)
            .send()
            .await?;
        Self::parse_response(resp).await
    }

    async fn parse_response(resp: reqwest::Response) -> Result<Value, CliError> {
        let status = resp.status();
        let body: Value = resp
            .json()
            .await
            .unwrap_or_else(|_| serde_json::json!({"error": "Non-JSON response"}));

        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            let msg = body["detail"]
                .as_str()
                .or_else(|| body["error"].as_str())
                .unwrap_or("Unauthorized")
                .to_string();
            return Err(CliError::Auth(msg));
        }
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(CliError::Server("Rate limited. Try again shortly.".into()));
        }
        if status == reqwest::StatusCode::UNPROCESSABLE_ENTITY {
            let msg = body["detail"]
                .as_str()
                .unwrap_or("Validation error")
                .to_string();
            return Err(CliError::BadInput(msg));
        }
        if status.is_client_error() {
            let msg = body["detail"]
                .as_str()
                .or_else(|| body["error"].as_str())
                .unwrap_or("Bad request")
                .to_string();
            return Err(CliError::BadInput(msg));
        }
        if status.is_server_error() {
            let msg = body["detail"]
                .as_str()
                .unwrap_or("Server error")
                .to_string();
            return Err(CliError::Server(msg));
        }

        Ok(body)
    }
}
