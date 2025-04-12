use std::sync::Arc;

use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{
    extract::State,
    http::{header, HeaderMap, Response, StatusCode},
    response::IntoResponse,
    Extension, Json,
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use rand_core::OsRng;
use serde_json::json;
use uuid::Uuid;

use crate::utils::{self,token};
use crate::utils::token::TokenDetails;
use crate::handler::auth::{ErrorResponse, JWTAuthMiddleware};
use crate::model::user::{LoginUserSchema, RegisterUserSchema, User};
use crate::model::reponse::FilteredUser;
use crate::initialize::app_state::AppState;
use redis::AsyncCommands;

pub async fn health_checker_handler() -> impl IntoResponse {
    const MESSAGE: &str = "Rust and Axum Framework: JWT Access and Refresh Tokens";

    let json_response = serde_json::json!({
        "status": "success",
        "message": MESSAGE
    });

    Json(json_response)
}

pub async fn register_user_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<RegisterUserSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
   //查询用存在
    let user_exists=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE email = ?)")
        .bind(body.email.to_owned().to_ascii_lowercase())
        .fetch_one(&data.db)
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Database error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;


    if let Some(exists) = user_exists {
        if exists {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": "User with that email already exists",
            });
            return Err((StatusCode::CONFLICT, Json(error_response)));
        }
    }

    //密码加密
    let salt = SaltString::generate(&mut OsRng);
    let hashed_password = Argon2::default()
        .hash_password(body.password.as_bytes(), &salt)
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Error while hashing password: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })
        .map(|hash| hash.to_string())?;

    let user_uuid = Uuid::new_v4().to_string();
    //存入
    let insert_sql="insert into users (uuid,name,email,password) values (?,?,?,?)";

   let result= sqlx::query(&insert_sql)
       .bind(user_uuid)
       .bind(body.name.to_string())
       .bind(body.email.to_string().to_ascii_lowercase())
       .bind(hashed_password)
        .execute(&data.db)
        .await
       .map_err(|e| {
            let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("save Database error: {}", e),
        });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;


    let get_sql="select * from `users` where id=?";

    let user:Option<User>=sqlx::query_as(get_sql)
        .bind(result.last_insert_id().clone())
        .fetch_optional(&data.db)
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("query Database by id error: {}", e),
        });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;


    if let Some(user)=user{
        let user_response = serde_json::json!({
            "status": "success",
            "data": serde_json::json!({
        "user": filter_user_record(&user)
    })});
        Ok(Json(user_response))
    }else{
        let error_res = serde_json::json!({
            "status": "fail",
            "message": format!("query Database by id error"),
        });

        Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_res)))
    }
}

pub async fn login_user_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<LoginUserSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let get_sql="SELECT * FROM users WHERE email =?";
    let user:User=sqlx::query_as(get_sql)
        .bind(body.email.to_ascii_lowercase())
        .fetch_optional(&data.db)
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
            "status": "error",
            "message": format!("Database error: {}", e),
        });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?
        .ok_or_else(|| {
            let error_response = serde_json::json!({
            "status": "fail",
            "message": "Invalid email or password",
        });
            (StatusCode::BAD_REQUEST, Json(error_response))
        })?;

    let is_valid = match PasswordHash::new(&user.password) {
        Ok(parsed_hash) => Argon2::default()
            .verify_password(body.password.as_bytes(), &parsed_hash)
            .map_or(false, |_| true),
        Err(_) => false,
    };

    if !is_valid {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "Invalid email or password"
        });
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    let access_token_details = generate_token(
        uuid::Uuid::parse_str(&user.uuid).unwrap_or_default(),
        data.conf.access_token.access_token_maxage.parse::<i64>().unwrap(),
        data.conf.access_token.access_token_private_key.to_owned(),
    )?;
    let refresh_token_details = generate_token(
        uuid::Uuid::parse_str(&user.uuid).unwrap_or_default(),
        data.conf.refresh_token.refresh_token_maxage.parse::<i64>().unwrap(),
        data.conf.refresh_token.refresh_token_private_key.to_owned(),
    )?;

    save_token_data_to_redis(&data, &access_token_details, data.conf.access_token.access_token_maxage.parse::<i64>().unwrap()).await?;
    save_token_data_to_redis(
        &data,
        &refresh_token_details,
        data.conf.refresh_token.refresh_token_maxage.parse::<i64>().unwrap(),
    )
        .await?;

    let access_cookie = Cookie::build(
        ("access_token",
         access_token_details.token.clone().unwrap_or_default()),
    )
        .path("/")
        .max_age(time::Duration::minutes(&data.conf.access_token.access_token_maxage.parse::<i64>().unwrap() * 60))
        .same_site(SameSite::Lax)
        .http_only(true);

    let refresh_cookie = Cookie::build(
        ("refresh_token",
         refresh_token_details.token.unwrap_or_default()),
    )
        .path("/")
        .max_age(time::Duration::minutes(&data.conf.refresh_token.refresh_token_maxage.parse::<i64>().unwrap() * 60))
        .same_site(SameSite::Lax)
        .http_only(true);

    let logged_in_cookie = Cookie::build(("logged_in", "true"))
        .path("/")
        .max_age(time::Duration::minutes(&data.conf.access_token.access_token_maxage.parse::<i64>().unwrap() * 60))
        .same_site(SameSite::Lax)
        .http_only(false);

    let mut response = Response::new(
        json!({"status": "success", "access_token": access_token_details.token.unwrap()})
            .to_string(),
    );
    let mut headers = HeaderMap::new();
    headers.append(
        header::SET_COOKIE,
        access_cookie.to_string().parse().unwrap(),
    );
    headers.append(
        header::SET_COOKIE,
        refresh_cookie.to_string().parse().unwrap(),
    );
    headers.append(
        header::SET_COOKIE,
        logged_in_cookie.to_string().parse().unwrap(),
    );

    response.headers_mut().extend(headers);
    Ok(response)
}

pub async fn refresh_access_token_handler(
    cookie_jar: CookieJar,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let message = "could not refresh access token";

    let refresh_token = cookie_jar
        .get("refresh_token")
        .map(|cookie| cookie.value().to_string())
        .ok_or_else(|| {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": message
            });
            (StatusCode::FORBIDDEN, Json(error_response))
        })?;

    let refresh_token_details =
        match token::verify_jwt_token(data.conf.refresh_token.refresh_token_public_key.to_owned(), &refresh_token)
        {
            Ok(token_details) => token_details,
            Err(e) => {
                let error_response = serde_json::json!({
                    "status": "fail",
                    "message": format_args!("{:?}", e)
                });
                return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
            }
        };

    let mut redis_client = data
        .redis_client
        .get_async_connection()
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "error",
                "message": format!("Redis error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    let redis_token_user_id = redis_client
        .get::<_, String>(refresh_token_details.token_uuid.to_string())
        .await
        .map_err(|_| {
            let error_response = serde_json::json!({
                "status": "error",
                "message": "Token is invalid or session has expired",
            });
            (StatusCode::UNAUTHORIZED, Json(error_response))
        })?;

    let user_id_uuid = uuid::Uuid::parse_str(&redis_token_user_id).map_err(|_| {
        let error_response = serde_json::json!({
            "status": "error",
            "message": "Token is invalid or session has expired",
        });
        (StatusCode::UNAUTHORIZED, Json(error_response))
    })?;


    let get_sql="select * from `users` where uuid=?";
    let user:User=sqlx::query_as(get_sql)
        .bind(user_id_uuid)
        .fetch_optional(&data.db)
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Error fetching user from database: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?
        .ok_or_else(|| {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": "The user belonging to this token no longer exists".to_string(),
            });
            (StatusCode::UNAUTHORIZED, Json(error_response))
        })?;




    let access_token_details = generate_token(
        uuid::Uuid::parse_str(&user.uuid).unwrap_or_default(),
        data.conf.access_token.access_token_maxage.parse::<i64>().unwrap(),
        data.conf.access_token.access_token_private_key.to_owned(),
    )?;

    save_token_data_to_redis(&data, &access_token_details, data.conf.access_token.access_token_maxage.parse::<i64>().unwrap()).await?;

    let access_cookie = Cookie::build(
        ("access_token",
         access_token_details.token.clone().unwrap_or_default()),
    )
        .path("/")
        .max_age(time::Duration::minutes(data.conf.access_token.access_token_maxage.parse::<i64>().unwrap() * 60))
        .same_site(SameSite::Lax)
        .http_only(true);

    let logged_in_cookie = Cookie::build(("logged_in", "true"))
        .path("/")
        .max_age(time::Duration::minutes(data.conf.access_token.access_token_maxage.parse::<i64>().unwrap() * 60))
        .same_site(SameSite::Lax)
        .http_only(false);

    let mut response = Response::new(
        json!({"status": "success", "access_token": access_token_details.token.unwrap()})
            .to_string(),
    );
    let mut headers = HeaderMap::new();
    headers.append(
        header::SET_COOKIE,
        access_cookie.to_string().parse().unwrap(),
    );
    headers.append(
        header::SET_COOKIE,
        logged_in_cookie.to_string().parse().unwrap(),
    );

    response.headers_mut().extend(headers);
    Ok(response)
}

pub async fn logout_handler(
    cookie_jar: CookieJar,
    Extension(auth_guard): Extension<JWTAuthMiddleware>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let message = "Token is invalid or session has expired";

    let refresh_token = cookie_jar
        .get("refresh_token")
        .map(|cookie| cookie.value().to_string())
        .ok_or_else(|| {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": message
            });
            (StatusCode::FORBIDDEN, Json(error_response))
        })?;

    let refresh_token_details =
        match token::verify_jwt_token(data.conf.refresh_token.refresh_token_public_key.to_owned(), &refresh_token)
        {
            Ok(token_details) => token_details,
            Err(e) => {
                let error_response = serde_json::json!({
                    "status": "fail",
                    "message": format_args!("{:?}", e)
                });
                return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
            }
        };

    let mut redis_client = data
        .redis_client
        .get_async_connection()
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "error",
                "message": format!("Redis error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    redis_client
        .del(&[
            refresh_token_details.token_uuid.to_string(),
            auth_guard.access_token_uuid.to_string(),
        ])
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "error",
                "message": format_args!("{:?}", e)
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    let access_cookie = Cookie::build(("access_token", ""))
        .path("/")
        .max_age(time::Duration::minutes(-1))
        .same_site(SameSite::Lax)
        .http_only(true);
    let refresh_cookie = Cookie::build(("refresh_token", ""))
        .path("/")
        .max_age(time::Duration::minutes(-1))
        .same_site(SameSite::Lax)
        .http_only(true);

    let logged_in_cookie = Cookie::build(("logged_in", "true"))
        .path("/")
        .max_age(time::Duration::minutes(-1))
        .same_site(SameSite::Lax)
        .http_only(false);

    let mut headers = HeaderMap::new();
    headers.append(
        header::SET_COOKIE,
        access_cookie.to_string().parse().unwrap(),
    );
    headers.append(
        header::SET_COOKIE,
        refresh_cookie.to_string().parse().unwrap(),
    );
    headers.append(
        header::SET_COOKIE,
        logged_in_cookie.to_string().parse().unwrap(),
    );

    let mut response = Response::new(json!({"status": "success"}).to_string());
    response.headers_mut().extend(headers);
    Ok(response)
}

pub async fn  get_me_handler(
    Extension(jwtauth): Extension<JWTAuthMiddleware>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let json_response = serde_json::json!({
        "status":  "success",
        "data": serde_json::json!({
            "user": filter_user_record(&jwtauth.user)
        })
    });

    Ok(Json(json_response))
}

fn filter_user_record(user: &User) -> FilteredUser {
    FilteredUser {
        id: user.id.to_string(),
        uuid: user.uuid.clone(),
        email: user.email.to_owned(),
        name: user.name.to_owned(),
        photo: user.photo.to_owned(),
        role: user.role.to_owned(),
        verified: user.verified,
        createdAt: user.created_at.unwrap(),
        updatedAt: user.updated_at.unwrap(),
    }
}


fn generate_token(
    user_id: uuid::Uuid,
    max_age: i64,
    private_key: String,
) -> Result<TokenDetails, (StatusCode, Json<serde_json::Value>)> {
    token::generate_jwt_token(user_id, max_age, private_key).map_err(|e| {
        let error_response = serde_json::json!({
            "status": "error",
            "message": format!("error generating token: {}", e),
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })
}

async fn save_token_data_to_redis(
    data: &Arc<AppState>,
    token_details: &TokenDetails,
    max_age: i64,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    let mut redis_client = data
        .redis_client
        .get_async_connection()
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "error",
                "message": format!("Redis error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;
    redis_client
        .set_ex(
            token_details.token_uuid.to_string(),
            token_details.user_id.to_string(),
            (max_age * 60) as u64,
        )
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "error",
                "message": format_args!("{}", e),
            });
            (StatusCode::UNPROCESSABLE_ENTITY, Json(error_response))
        })?;
    Ok(())
}
