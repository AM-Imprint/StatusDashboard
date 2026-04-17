use axum::{
    extract::{Path, Query, State},
    Json,
};
use std::collections::HashMap;
use crate::{error::AppError, models::check_result::CheckResult, state::AppState};

pub async fn list_checks(
    State(state): State<AppState>,
    Path(service_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<CheckResult>>, AppError> {
    let limit: i64 = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(50);

    let results = if let Some(before_id) = params.get("before_id") {
        let before_at = sqlx::query_scalar!(
            r#"SELECT checked_at as "checked_at!" FROM check_results WHERE id = ?"#,
            before_id
        )
        .fetch_optional(&state.db)
        .await?;

        if let Some(at) = before_at {
            sqlx::query_as!(
                CheckResult,
                r#"SELECT id as "id!", service_id as "service_id!", checked_at as "checked_at!",
                   status as "status!", response_ms, detail, error_message
                   FROM check_results WHERE service_id = ? AND checked_at < ?
                   ORDER BY checked_at DESC LIMIT ?"#,
                service_id, at, limit
            )
            .fetch_all(&state.db)
            .await?
        } else {
            vec![]
        }
    } else {
        sqlx::query_as!(
            CheckResult,
            r#"SELECT id as "id!", service_id as "service_id!", checked_at as "checked_at!",
               status as "status!", response_ms, detail, error_message
               FROM check_results WHERE service_id = ?
               ORDER BY checked_at DESC LIMIT ?"#,
            service_id, limit
        )
        .fetch_all(&state.db)
        .await?
    };

    Ok(Json(results))
}
