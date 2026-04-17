use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[allow(dead_code)]
pub struct Service {
    pub id: String,
    pub name: String,
    pub service_type: String,
    pub config: String,
    pub interval_secs: i64,
    pub enabled: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateService {
    pub name: String,
    pub service_type: String,
    pub config: serde_json::Value,
    pub interval_secs: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateService {
    pub name: Option<String>,
    pub config: Option<serde_json::Value>,
    pub interval_secs: Option<i64>,
    pub enabled: Option<bool>,
}
