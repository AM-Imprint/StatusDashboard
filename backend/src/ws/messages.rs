use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    CheckCompleted {
        service_id: String,
        check_id: String,
        checked_at: String,
        status: String,
        response_ms: Option<i64>,
        detail: Option<serde_json::Value>,
        error_message: Option<String>,
    },
    IncidentOpened {
        incident_id: String,
        service_id: String,
        started_at: String,
        trigger_status: String,
    },
    IncidentResolved {
        incident_id: String,
        service_id: String,
        resolved_at: String,
    },
    ServiceUpdated {
        service_id: String,
        fields: serde_json::Value,
    },
    Ping {
        ts: String,
    },
}
