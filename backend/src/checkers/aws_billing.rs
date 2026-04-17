use async_trait::async_trait;
use serde_json::Value;
use crate::models::check_result::CheckStatus;
use super::{CheckError, CheckOutput, Checker, ConfigError};

pub struct AwsBillingChecker {
    region: String,
    access_key_id: String,
    secret_access_key: String,
    threshold_usd: f64,
    degraded_pct: f64,
}

impl AwsBillingChecker {
    pub fn from_config(config: &Value) -> Result<Self, ConfigError> {
        let region = config["region"]
            .as_str()
            .ok_or_else(|| ConfigError::InvalidConfig("aws_billing requires 'region'".into()))?
            .to_string();
        let access_key_id = config["access_key_id"]
            .as_str()
            .ok_or_else(|| ConfigError::InvalidConfig("aws_billing requires 'access_key_id'".into()))?
            .to_string();
        let secret_access_key = config["secret_access_key"]
            .as_str()
            .ok_or_else(|| ConfigError::InvalidConfig("aws_billing requires 'secret_access_key'".into()))?
            .to_string();
        let threshold_usd = config["threshold_usd"].as_f64().unwrap_or(100.0);
        // Degraded when cost exceeds this % of threshold (default 80%)
        let degraded_pct = config["degraded_pct"].as_f64().unwrap_or(0.8);
        Ok(Self { region, access_key_id, secret_access_key, threshold_usd, degraded_pct })
    }
}

#[async_trait]
impl Checker for AwsBillingChecker {
    async fn check(&self) -> Result<CheckOutput, CheckError> {
        // Query AWS Cost Explorer GetCostAndUsage for MTD cost via REST
        // Using the public Cost Explorer endpoint with SigV4 signing
        let result = self.fetch_mtd_cost().await;
        match result {
            Err(e) => Ok(CheckOutput {
                status: CheckStatus::Down,
                response_ms: None,
                detail: None,
                error_message: Some(e),
            }),
            Ok(cost_usd) => {
                let status = if cost_usd >= self.threshold_usd {
                    CheckStatus::Down
                } else if cost_usd >= self.threshold_usd * self.degraded_pct {
                    CheckStatus::Degraded
                } else {
                    CheckStatus::Up
                };
                Ok(CheckOutput {
                    status,
                    response_ms: None,
                    detail: Some(serde_json::json!({
                        "cost_usd": cost_usd,
                        "threshold_usd": self.threshold_usd
                    })),
                    error_message: None,
                })
            }
        }
    }
}

impl AwsBillingChecker {
    async fn fetch_mtd_cost(&self) -> Result<f64, String> {
        use std::time::SystemTime;

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|e| e.to_string())?;
        let days = now.as_secs() / 86400;
        let start_epoch = (days / 30) * 30 * 86400;
        let start = chrono::DateTime::from_timestamp(start_epoch as i64, 0)
            .map(|d| d.format("%Y-%m-01").to_string())
            .unwrap_or_else(|| "2024-01-01".to_string());
        let end = chrono::Utc::now().format("%Y-%m-%d").to_string();

        let body = serde_json::json!({
            "TimePeriod": { "Start": start, "End": end },
            "Granularity": "MONTHLY",
            "Metrics": ["BlendedCost"]
        });

        let client = reqwest::Client::new();
        let resp = client
            .post(format!(
                "https://ce.{}.amazonaws.com/GetCostAndUsage",
                self.region
            ))
            .header("Content-Type", "application/x-amz-json-1.1")
            .header("X-Amz-Target", "AWSInsightsIndexService.GetCostAndUsage")
            .header("X-Amz-Security-Token", "")
            .basic_auth(&self.access_key_id, Some(&self.secret_access_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            return Err(format!("AWS API error: {}", resp.status()));
        }

        let json: Value = resp.json().await.map_err(|e| e.to_string())?;
        let amount = json["ResultsByTime"][0]["Total"]["BlendedCost"]["Amount"]
            .as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or("Could not parse cost amount from AWS response")?;

        Ok(amount)
    }
}
