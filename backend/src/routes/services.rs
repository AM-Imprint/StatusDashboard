use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use uuid::Uuid;
use crate::{
    error::AppError,
    models::service::{CreateService, UpdateService},
    scheduler,
    state::AppState,
    ws::messages::WsMessage,
};

pub async fn list_services(
    State(state): State<AppState>,
) -> Result<Json<Vec<serde_json::Value>>, AppError> {
    let rows = sqlx::query!(
        r#"SELECT id as "id!", name as "name!", service_type as "service_type!",
           config as "config!", interval_secs as "interval_secs!", enabled as "enabled!",
           system_id, created_at as "created_at!", updated_at as "updated_at!"
           FROM services ORDER BY created_at ASC"#
    )
    .fetch_all(&state.db)
    .await?;

    let mut result = Vec::with_capacity(rows.len());
    for r in rows {
        let latest = sqlx::query!(
            r#"SELECT id as "id!", checked_at as "checked_at!", status as "status!",
               response_ms, error_message
               FROM check_results WHERE service_id = ? ORDER BY checked_at DESC LIMIT 1"#,
            r.id
        )
        .fetch_optional(&state.db)
        .await?;

        let latest_check = latest.map(|c| serde_json::json!({
            "id": c.id,
            "checked_at": c.checked_at,
            "status": c.status,
            "response_ms": c.response_ms,
            "error_message": c.error_message
        }));

        result.push(serde_json::json!({
            "id": r.id,
            "name": r.name,
            "service_type": r.service_type,
            "config": serde_json::from_str::<serde_json::Value>(&r.config).unwrap_or(serde_json::Value::Null),
            "interval_secs": r.interval_secs,
            "enabled": r.enabled == 1,
            "system_id": r.system_id,
            "created_at": r.created_at,
            "updated_at": r.updated_at,
            "latest_check": latest_check
        }));
    }
    Ok(Json(result))
}

pub async fn get_service(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query!(
        r#"SELECT id as "id!", name as "name!", service_type as "service_type!",
           config as "config!", interval_secs as "interval_secs!", enabled as "enabled!",
           system_id, created_at as "created_at!", updated_at as "updated_at!"
           FROM services WHERE id = ?"#,
        id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(serde_json::json!({
        "id": r.id,
        "name": r.name,
        "service_type": r.service_type,
        "config": serde_json::from_str::<serde_json::Value>(&r.config).unwrap_or(serde_json::Value::Null),
        "interval_secs": r.interval_secs,
        "enabled": r.enabled == 1,
        "system_id": r.system_id,
        "created_at": r.created_at,
        "updated_at": r.updated_at
    })))
}

pub async fn create_service(
    State(state): State<AppState>,
    Json(body): Json<CreateService>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let interval = body.interval_secs.unwrap_or(60);
    let config_str = body.config.to_string();

    crate::checkers::build_checker(&body.service_type, &body.config)
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    sqlx::query!(
        "INSERT INTO services (id, name, service_type, config, interval_secs, enabled, system_id, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, 1, ?, ?, ?)",
        id, body.name, body.service_type, config_str, interval, body.system_id, now, now
    )
    .execute(&state.db)
    .await?;

    scheduler::spawn_service(id.clone(), &state.db, state.tx.clone(), &state.scheduler_handles).await;

    let resp = serde_json::json!({ "id": id, "name": body.name, "system_id": body.system_id, "created_at": now });
    Ok((StatusCode::CREATED, Json(resp)))
}

pub async fn update_service(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateService>,
) -> Result<Json<serde_json::Value>, AppError> {
    let r = sqlx::query!(
        r#"SELECT id as "id!", name as "name!", config as "config!",
           interval_secs as "interval_secs!", enabled as "enabled!", system_id
           FROM services WHERE id = ?"#,
        id
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let name = body.name.unwrap_or(r.name);
    let config_str = body.config.as_ref().map(|c| c.to_string()).unwrap_or(r.config);
    let interval = body.interval_secs.unwrap_or(r.interval_secs);
    let enabled: i64 = body.enabled.map(|e| e as i64).unwrap_or(r.enabled);
    // double-Option: Some(Some(id)) = assign, Some(None) = unassign, None = no change
    let new_system_id: Option<Option<String>> = body.system_id;
    let now = Utc::now().to_rfc3339();

    match &new_system_id {
        Some(sid) => {
            sqlx::query!(
                "UPDATE services SET name = ?, config = ?, interval_secs = ?, enabled = ?, system_id = ?, updated_at = ?
                 WHERE id = ?",
                name, config_str, interval, enabled, sid, now, id
            )
            .execute(&state.db)
            .await?;
        }
        None => {
            sqlx::query!(
                "UPDATE services SET name = ?, config = ?, interval_secs = ?, enabled = ?, updated_at = ?
                 WHERE id = ?",
                name, config_str, interval, enabled, now, id
            )
            .execute(&state.db)
            .await?;
        }
    }

    if enabled == 1 {
        scheduler::spawn_service(id.clone(), &state.db, state.tx.clone(), &state.scheduler_handles).await;
    } else {
        scheduler::abort_service(&id, &state.scheduler_handles).await;
    }

    let effective_system_id = new_system_id
        .clone()
        .unwrap_or(r.system_id.map(Some).unwrap_or(None));

    let _ = state.tx.send(WsMessage::ServiceUpdated {
        service_id: id.clone(),
        fields: serde_json::json!({
            "name": name,
            "interval_secs": interval,
            "enabled": enabled == 1,
            "system_id": effective_system_id
        }),
    });

    Ok(Json(serde_json::json!({ "id": id, "updated_at": now })))
}

pub async fn delete_service(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let rows = sqlx::query!("DELETE FROM services WHERE id = ?", id)
        .execute(&state.db)
        .await?
        .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound);
    }

    scheduler::abort_service(&id, &state.scheduler_handles).await;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_uptime(
    State(state): State<AppState>,
    Path(id): Path<String>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let window = params.get("window").map(String::as_str).unwrap_or("7d");
    let days: i64 = match window {
        "24h" => 1,
        "30d" => 30,
        "90d" => 90,
        _ => 7,
    };
    let since = (Utc::now() - chrono::Duration::days(days)).to_rfc3339();

    let total: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM check_results WHERE service_id = ? AND checked_at >= ?",
        id, since
    )
    .fetch_one(&state.db)
    .await?;

    let up: i64 = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM check_results WHERE service_id = ? AND checked_at >= ? AND status = 'up'",
        id, since
    )
    .fetch_one(&state.db)
    .await?;

    let pct = if total == 0 { None } else { Some(up as f64 / total as f64 * 100.0) };
    Ok(Json(serde_json::json!({ "window": window, "uptime_pct": pct, "total_checks": total, "up_checks": up })))
}
