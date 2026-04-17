use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Instant;
use crate::models::check_result::CheckStatus;
use super::{CheckError, CheckOutput, Checker, ConfigError};

pub struct HttpChecker {
    url: String,
    method: String,
    expected_status: u16,
    timeout_ms: u64,
    degraded_ms: Option<u64>,
    headers: HashMap<String, String>,
}

impl HttpChecker {
    pub fn from_config(config: &Value) -> Result<Self, ConfigError> {
        let url = config["url"]
            .as_str()
            .ok_or_else(|| ConfigError::InvalidConfig("http checker requires 'url'".into()))?
            .to_string();
        let method = config["method"].as_str().unwrap_or("GET").to_uppercase();
        let expected_status = config["expected_status"].as_u64().unwrap_or(200) as u16;
        let timeout_ms = config["timeout_ms"].as_u64().unwrap_or(10_000);
        let degraded_ms = config["degraded_ms"].as_u64();
        let headers = config["headers"]
            .as_object()
            .map(|m| {
                m.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default();
        Ok(Self { url, method, expected_status, timeout_ms, degraded_ms, headers })
    }
}

#[async_trait]
impl Checker for HttpChecker {
    async fn check(&self) -> Result<CheckOutput, CheckError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(self.timeout_ms))
            .build()
            .map_err(|e| CheckError::Error(e.to_string()))?;

        let mut req = client.request(
            self.method.parse().unwrap_or(reqwest::Method::GET),
            &self.url,
        );
        for (k, v) in &self.headers {
            req = req.header(k, v);
        }

        let start = Instant::now();
        let result = req.send().await;
        let elapsed = start.elapsed().as_millis() as u64;

        match result {
            Err(e) => Ok(CheckOutput {
                status: CheckStatus::Down,
                response_ms: Some(elapsed),
                detail: None,
                error_message: Some(e.to_string()),
            }),
            Ok(resp) => {
                let actual_status = resp.status().as_u16();
                let status = if actual_status != self.expected_status {
                    CheckStatus::Down
                } else if self.degraded_ms.map(|d| elapsed > d).unwrap_or(false) {
                    CheckStatus::Degraded
                } else {
                    CheckStatus::Up
                };
                Ok(CheckOutput {
                    status,
                    response_ms: Some(elapsed),
                    detail: Some(serde_json::json!({ "http_status": actual_status })),
                    error_message: None,
                })
            }
        }
    }
}
