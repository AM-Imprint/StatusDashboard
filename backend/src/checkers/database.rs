use async_trait::async_trait;
use serde_json::Value;
use std::time::Instant;
use crate::models::check_result::CheckStatus;
use super::{CheckError, CheckOutput, Checker, ConfigError};

pub struct DatabaseChecker {
    driver: String,
    connection_string: String,
    probe_query: String,
    degraded_ms: Option<u64>,
}

impl DatabaseChecker {
    pub fn from_config(config: &Value) -> Result<Self, ConfigError> {
        let driver = config["driver"]
            .as_str()
            .ok_or_else(|| ConfigError::InvalidConfig("database checker requires 'driver'".into()))?
            .to_string();
        let connection_string = config["connection_string"]
            .as_str()
            .ok_or_else(|| ConfigError::InvalidConfig("database checker requires 'connection_string'".into()))?
            .to_string();
        let probe_query = config["probe_query"]
            .as_str()
            .unwrap_or("SELECT 1")
            .to_string();
        let degraded_ms = config["degraded_ms"].as_u64();
        Ok(Self { driver, connection_string, probe_query, degraded_ms })
    }
}

#[async_trait]
impl Checker for DatabaseChecker {
    async fn check(&self) -> Result<CheckOutput, CheckError> {
        let start = Instant::now();
        let result = self.run_probe().await;
        let elapsed = start.elapsed().as_millis() as u64;

        match result {
            Err(e) => Ok(CheckOutput {
                status: CheckStatus::Down,
                response_ms: Some(elapsed),
                detail: None,
                error_message: Some(e),
            }),
            Ok(()) => {
                let status = if self.degraded_ms.map(|d| elapsed > d).unwrap_or(false) {
                    CheckStatus::Degraded
                } else {
                    CheckStatus::Up
                };
                Ok(CheckOutput {
                    status,
                    response_ms: Some(elapsed),
                    detail: Some(serde_json::json!({ "driver": self.driver })),
                    error_message: None,
                })
            }
        }
    }
}

impl DatabaseChecker {
    async fn run_probe(&self) -> Result<(), String> {
        match self.driver.as_str() {
            "sqlite" => {
                let pool = sqlx::SqlitePool::connect(&self.connection_string)
                    .await
                    .map_err(|e| e.to_string())?;
                sqlx::query(&self.probe_query)
                    .execute(&pool)
                    .await
                    .map_err(|e| e.to_string())?;
                pool.close().await;
                Ok(())
            }
            other => Err(format!("Unsupported database driver: {other}")),
        }
    }
}
