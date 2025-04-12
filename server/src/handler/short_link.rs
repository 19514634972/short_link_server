use std::sync::Arc;
use axum::extract::{State,Path};
use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use log::info;
use tracing_subscriber::fmt::format;
use serde::{Deserialize, Serialize};
use crate::initialize::app_state::AppState;
use crate::utils::six_code;
use crate::utils::verify_code;
use sqlx::MySql;

use crate::model::short_link::{
    SaveLinkReq,
    UpdateLinkReq,
    ShortLinkListReq,
    ShortLink,
};

pub async fn SaveShortLink(
    State(data): State<Arc<AppState>>,
    Json(body): Json<SaveLinkReq>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {

    let mut six_code = String::new();
    loop {
        let generated_code = six_code::generate_random_string(8);
        //查询库中有没有这个code没有
        let query_sql = "select * from short_link where short_code=?";
        let user_exists = sqlx::query_scalar(query_sql)
            .bind(&generated_code.to_string())
            .fetch_optional(&data.db)
            .await
            .map_err(|e| {
                let error_response = serde_json::json!({
                    "status": "fail",
                    "message": format!("Database error: {}", e),
                });
                info!("查询code报错");
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
            })?;


        if let Some(exists) = user_exists {
            if exists {
                continue;
            }
        }

        six_code = generated_code;
        break;
    }

    let short_link=format!("{}{}",data.conf.redirect_url,&six_code);

    let save_sql="insert into short_link (name,short_url,full_url,short_code) values(?,?,?,?)";

    let result=sqlx::query(save_sql)
        .bind(body.name.to_string())
        .bind(short_link)
        .bind(body.full_url.to_string())
        .bind(six_code)
        .execute(&data.db)
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("save Database error: {}", e),
        });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    if result.rows_affected() == 0 {
        let error_response = serde_json::json!({
        "status": "fail",
        "message": "no row data insert",
    });
       return  Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
    }

    let res = serde_json::json!({
            "status": "success",
            "message": "save short link success".to_string(),
        });

    Ok(Json(res))

}



//修改指定短链链接
pub async fn UpdateShortLink(
    State(data): State<Arc<AppState>>,
    Path(id):Path<u64>,
    Json(body): Json<UpdateLinkReq>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
   let res= verify_code::is_five_letters(&body.short_code);
    if !res && !body.short_code.is_empty() {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "short_code is error".to_string(),
        });
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }


    let update_sql="update short_link SET name=?,short_code=? WHERE id=?";

    let result=sqlx::query(update_sql)
        .bind(body.name)
        .bind(body.short_code)
        .bind(id).execute(&data.db)
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("save Database error: {}", e),
        });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    if result.rows_affected() == 0 {
        let error_response = serde_json::json!({
        "status": "fail",
        "message": "no row data insert",
    });
        return  Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
    }

    let res = serde_json::json!({
            "status": "success",
            "message": "update short link success".to_string(),
        });

    Ok(Json(res))

}

//删除
pub async fn DelShortLinkByID(
    State(data): State<Arc<AppState>>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let delete_sql = "DELETE FROM short_link WHERE id=?";
    let result = sqlx::query(delete_sql)
        .bind(id)
        .execute(&data.db)
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Database error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    if result.rows_affected() == 0 {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "short link not found",
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    let response = serde_json::json!({
        "status": "success",
        "message": "delete short link success",
    });

    Ok(Json(response))
}


pub async fn GetShortLinkByID(
    State(data): State<Arc<AppState>>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_sql = "select id,name,short_url,full_url,short_code,created_at,updated_at,deleted_at from short_link WHERE id=? AND deleted_at IS NULL";
    
    let result = sqlx::query_as::<_, ShortLink>(query_sql)
        .bind(id)
        .fetch_optional(&data.db)
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Database error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    match result {
        Some(short_link) => {
            let response = serde_json::json!({
                "status": "success",
                "message": "get short link success".to_string(),
                "data": short_link
            });
            Ok(Json(response))
        }
        None => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": "short link not found",
            });
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
    }
}


//获取短链列表
pub async fn GetShortLinkList(
    State(data): State<Arc<AppState>>,
    Json(body): Json<ShortLinkListReq>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let get_list = "select id,name,short_url,full_url,short_code,created_at,updated_at,deleted_at from short_link WHERE deleted_at IS NULL ORDER BY created_at DESC";
    let page = body.page.unwrap_or(1);
    let page_size = body.pageSize.unwrap_or(10);
    let offset = (page - 1) * page_size;

    let mut query_builder: sqlx::QueryBuilder<'_, MySql> = sqlx::QueryBuilder::new(get_list);
    query_builder.push(" LIMIT ").push_bind(page_size);
    query_builder.push(" OFFSET ").push_bind(offset);

    let res= query_builder.build_query_as::<ShortLink>().fetch_all(&data.db).await.unwrap();
    let response = serde_json::json!({
            "status": "success",
            "message": "get short link list success".to_string(),
            "data": {
                "list": res,
                "page": page,
                "pageSize": page_size
            }
        });

    Ok(Json(response))

}


