use std::sync::Arc;
use axum::{
    extract::{Path, State},
    http::{StatusCode,HeaderMap},
};
use axum::Json;
use axum::response::IntoResponse;
use crate::initialize::app_state::AppState;
use sqlx::MySql;
use axum::response::Redirect;
use log::info;
use crate::utils::devices::get_web_info;


pub async fn redirect_short_link(
    State(data): State<Arc<AppState>>,
    Path(code): Path<String>,
    headers:    HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {

    //用户代理信息
    let user_agent = headers.get("User-Agent")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("Unknown")
        .to_string();

    let device_type =get_web_info(&user_agent);

    info!("=========用户设备类型{:?}",device_type);

    let query_sql = "SELECT full_url FROM short_link WHERE short_code = ? AND deleted_at IS NULL";

    let full_url: Option<String> = sqlx::query_scalar(query_sql)
        .bind(&code)
        .fetch_optional(&data.db)
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Database error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;
    match full_url {
        Some(url) => Ok(Redirect::permanent(&url)),
        None => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": "Short link not found",
            });
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
    }
}


