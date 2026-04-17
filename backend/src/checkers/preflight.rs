use async_trait::async_trait;
use serde_json::Value;
use std::time::Instant;
use tokio::process::Command;
use crate::models::check_result::CheckStatus;
use super::{CheckError, CheckOutput, Checker, ConfigError};

pub struct PreflightChecker {
    command: String,
    args: Vec<String>,
    expected_exit_code: i32,
    timeout_ms: u64,
    degraded_ms: Option<u64>,
}

impl PreflightChecker {
    pub fn from_config(config: &Value) -> Result<Self, ConfigError> {
        let command = config["command"]
            .as_str()
            .ok_or_else(|| ConfigError::InvalidConfig("preflight checker requires 'command'".into()))?
            .to_string();
        let args = config["args"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        let expected_exit_code = config["expected_exit_code"].as_i64().unwrap_or(0) as i32;
        let timeout_ms = config["timeout_ms"].as_u64().unwrap_or(30_000);
        let degraded_ms = config["degraded_ms"].as_u64();
        Ok(Self { command, args, expected_exit_code, timeout_ms, degraded_ms })
    }
}

#[async_trait]
impl Checker for PreflightChecker {
    async fn check(&self) -> Result<CheckOutput, CheckError> {
        let start = Instant::now();
        let result = tokio::time::timeout(
            std::time::Duration::from_millis(self.timeout_ms),
            Command::new(&self.command).args(&self.args).output(),
        )
        .await;
        let elapsed = start.elapsed().as_millis() as u64;

        match result {
            Err(_) => Ok(CheckOutput {
                status: CheckStatus::Down,
                response_ms: Some(elapsed),
                detail: None,
                error_message: Some("Command timed out".to_string()),
            }),
            Ok(Err(e)) => Ok(CheckOutput {
                status: CheckStatus::Down,
                response_ms: Some(elapsed),
                detail: None,
                error_message: Some(e.to_string()),
            }),
            Ok(Ok(output)) => {
                let exit_code = output.status.code().unwrap_or(-1);
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                let status = if exit_code != self.expected_exit_code {
                    CheckStatus::Down
                } else if self.degraded_ms.map(|d| elapsed > d).unwrap_or(false) {
                    CheckStatus::Degraded
                } else {
                    CheckStatus::Up
                };

                Ok(CheckOutput {
                    status,
                    response_ms: Some(elapsed),
                    detail: Some(serde_json::json!({
                        "exit_code": exit_code,
                        "stdout": stdout.trim(),
                        "stderr": stderr.trim()
                    })),
                    error_message: if exit_code != self.expected_exit_code {
                        Some(format!("Exit code {exit_code}, expected {}", self.expected_exit_code))
                    } else {
                        None
                    },
                })
            }
        }
    }
}
