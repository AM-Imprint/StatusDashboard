use axum::{
    extract::{Path, State},
    Json,
};
use chrono::Utc;
use crate::{
    error::AppError,
    models::incident::{Incident, ResolveIncident},
    state::AppState,
    ws::messages::WsMessage,
};

pub async fn list_incidents(
    State(state): State<AppState>,
    Path(service_id): Path<String>,
) -> Result<Json<Vec<Incident>>, AppError> {
    let rows = sqlx::query!(
        r#"SELECT id as "id!", service_id as "service_id!", started_at as "started_at!",
           resolved_at, status as "status!", trigger_status as "trigger_status!", notes
           FROM incidents WHERE service_id = ?
           ORDER BY started_at DESC LIMIT 100"#,
        service_id
    )
    .fetch_all(&state.db)
    .await?;

    let incidents = rows
        .into_iter()
        .map(|r| Incident {
            id: r.id,
            service_id: r.service_id,
            started_at: r.started_at,
            resolved_at: r.resolved_at,
            status: r.status,
            trigger_status: r.trigger_status,
            notes: r.notes,
        })
        .collect();

    Ok(Json(incidents))
}

pub async fn resolve_incident(
    State(state): State<AppState>,
    Path((service_id, incident_id)): Path<(String, String)>,
    Json(body): Json<ResolveIncident>,
) -> Result<Json<Incident>, AppError> {
    let now = Utc::now().to_rfc3339();

    let rows = sqlx::query!(
        "UPDATE incidents SET resolved_at = ?, status = 'resolved', notes = ?
         WHERE id = ? AND service_id = ? AND status = 'open'",
        now, body.notes, incident_id, service_id
    )
    .execute(&state.db)
    .await?
    .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound);
    }

    let r = sqlx::query!(
        r#"SELECT id as "id!", service_id as "service_id!", started_at as "started_at!",
           resolved_at, status as "status!", trigger_status as "trigger_status!", notes
           FROM incidents WHERE id = ?"#,
        incident_id
    )
    .fetch_one(&state.db)
    .await?;

    let incident = Incident {
        id: r.id,
        service_id: r.service_id,
        started_at: r.started_at,
        resolved_at: r.resolved_at,
        status: r.status,
        trigger_status: r.trigger_status,
        notes: r.notes,
    };

    let _ = state.tx.send(WsMessage::IncidentResolved {
        incident_id: incident.id.clone(),
        service_id: service_id.clone(),
        resolved_at: now,
    });

    Ok(Json(incident))
}
