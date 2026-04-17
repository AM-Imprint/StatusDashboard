use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use uuid::Uuid;
use crate::{
    error::AppError,
    models::system::{CreateSystem, UpdateSystem},
    state::AppState,
    ws::messages::WsMessage,
};

pub async fn list_systems(
    State(state): State<AppState>,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let rows = sqlx::query!(
        r#"SELECT id as "id!", name as "name!", description, created_at as "created_at!", updated_at as "updated_at!"
           FROM systems ORDER BY created_at ASC"#
    )
    .fetch_all(&state.db)
    .await?;

    let mut result = Vec::with_capacity(rows.len());
    for r in &rows {
        let health = derive_health(&r.id, &state).await;
        let count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM service_systems WHERE system_id = ?", r.id
        )
        .fetch_one(&state.db)
        .await?;

        result.push(serde_json::json!({
            "id": r.id,
            "name": r.name,
            "description": r.description,
            "health": health,
            "service_count": count,
            "created_at": r.created_at,
            "updated_at": r.updated_at
        }));
    }

    Ok(Json(result))
}

pub async fn create_system(
    State(state): State<AppState>,
    Json(body): Json<CreateSystem>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    sqlx::query!(
        "INSERT INTO systems (id, name, description, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
        id, body.name, body.description, now, now
    )
    .execute(&state.db)
    .await?;

    let resp = serde_json::json!({
        "id": id,
        "name": body.name,
        "description": body.description,
        "health": "unknown",
        "service_count": 0,
        "created_at": now,
        "updated_at": now
    });
    Ok((StatusCode::CREATED, Json(resp)))
}

pub async fn update_system(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateSystem>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query!(
        r#"SELECT id as "id!", name as "name!", description, created_at as "created_at!", updated_at as "updated_at!"
           FROM systems WHERE id = ?"#,
        id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let name = body.name.unwrap_or(r.name);
    let description = body.description.or(r.description);
    let now = Utc::now().to_rfc3339();

    sqlx::query!(
        "UPDATE systems SET name = ?, description = ?, updated_at = ? WHERE id = ?",
        name, description, now, id
    )
    .execute(&state.db)
    .await?;

    let _ = state.tx.send(WsMessage::SystemUpdated {
        system_id: id.clone(),
        fields: serde_json::json!({ "name": name, "description": description, "updated_at": now }),
    });

    Ok(Json(serde_json::json!({ "id": id, "name": name, "updated_at": now })))
}

pub async fn delete_system(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    // ON DELETE SET NULL handles reassigning services to ungrouped
    let rows = sqlx::query!("DELETE FROM systems WHERE id = ?", id)
        .execute(&state.db)
        .await?
        .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn derive_health(system_id: &str, state: &AppState) -> &'static str {
    let statuses = sqlx::query!(
        r#"SELECT status as "status!" FROM check_results
           WHERE service_id IN (SELECT service_id FROM service_systems WHERE system_id = ?)
           AND id IN (
               SELECT id FROM check_results cr2
               WHERE cr2.service_id = check_results.service_id
               ORDER BY checked_at DESC LIMIT 1
           )"#,
        system_id
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    if statuses.is_empty() {
        return "unknown";
    }

    let mut worst = "up";
    for row in &statuses {
        match row.status.as_str() {
            "down"     => return "down",
            "degraded" => worst = "degraded",
            _          => {}
        }
    }
    worst
}
