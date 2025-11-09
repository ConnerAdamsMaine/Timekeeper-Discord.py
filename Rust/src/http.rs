use pyo3::prelude::*;
use reqwest::{Client as ReqwestClient, header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT}};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use crate::errors::DiscordError;

const API_BASE: &str = "https://discord.com/api/v10";
const USER_AGENT_STRING: &str = "DiscordBot (discord.py-rust 3.0.0)";

#[derive(Clone)]
pub struct HTTPClient {
    client: ReqwestClient,
    token: Arc<RwLock<Option<String>>>,
    rate_limits: Arc<RwLock<HashMap<String, f64>>>,
}

impl HTTPClient {
    pub fn new() -> Result<Self, DiscordError> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_STRING));

        let client = ReqwestClient::builder()
            .default_headers(headers)
            .build()?;

        Ok(HTTPClient {
            client,
            token: Arc::new(RwLock::new(None)),
            rate_limits: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn set_token(&self, token: String) {
        *self.token.write().await = Some(token);
    }

    pub async fn request(
        &self,
        method: &str,
        path: &str,
        json: Option<Value>,
    ) -> Result<Value, DiscordError> {
        let url = format!("{}{}", API_BASE, path);

        let token = self.token.read().await.clone();
        let mut req = match method {
            "GET" => self.client.get(&url),
            "POST" => self.client.post(&url),
            "PUT" => self.client.put(&url),
            "PATCH" => self.client.patch(&url),
            "DELETE" => self.client.delete(&url),
            _ => return Err(DiscordError::InvalidData(format!("Invalid HTTP method: {}", method))),
        };

        if let Some(token) = token {
            req = req.header(AUTHORIZATION, format!("Bot {}", token));
        }

        if let Some(body) = json {
            req = req.json(&body);
        }

        let response = req.send().await?;
        let status = response.status();

        if status.is_success() {
            Ok(response.json().await?)
        } else {
            match status.as_u16() {
                403 => Err(DiscordError::Forbidden),
                404 => Err(DiscordError::NotFound),
                429 => {
                    let json: Value = response.json().await?;
                    let retry_after = json["retry_after"].as_f64().unwrap_or(1.0);
                    Err(DiscordError::RateLimited { retry_after })
                }
                500..=599 => Err(DiscordError::ServerError),
                _ => Err(DiscordError::InvalidData(format!("HTTP error: {}", status))),
            }
        }
    }

    pub async fn get_gateway(&self) -> Result<String, DiscordError> {
        let response = self.request("GET", "/gateway", None).await?;
        response["url"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| DiscordError::Gateway("No gateway URL in response".to_string()))
    }

    pub async fn get_gateway_bot(&self) -> Result<Value, DiscordError> {
        self.request("GET", "/gateway/bot", None).await
    }
}

pub fn register_module(_m: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}
