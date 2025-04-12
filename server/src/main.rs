/*
 * @Author: wzf 1490216271@qq.com
 * @Date: 2025-04-08 14:41:52
 * @LastEditors: wzf 1490216271@qq.com
 * @LastEditTime: 2025-04-12 20:51:05
 * @FilePath: \short_link_server\server\src\main.rs
 * @Description: 这是默认设置,请设置`customMade`, 打开koroFileHeader查看配置 进行设置: https://github.com/OBKoro1/koro1FileHeader/wiki/%E9%85%8D%E7%BD%AE
 */
use server_config::config::ApplicationConfig;
use db::init_db::conn_pool;
use server::{init_context, APPLICATION_CONTEXT};
use std::sync::Arc;
use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use sqlx::{Pool, MySql};
use tower_http::cors::CorsLayer;
use tokio::time::{sleep, Duration};
use tracing_subscriber::fmt::time::LocalTime;
use time::macros::format_description;

use log::info;
use redis::Client;
use server::initialize::app_state::AppState;
use server::router::router::create_router;
use server::initialize::config::init_config;


#[tokio::main]
async fn main() {
    // 初始化配置
    init_config().await;

    init_context().await;
    info!("初始化环境上下文完成");
    info!("初始化db完成 ");


    //跨域设置
    let cors = CorsLayer::new()
        .allow_origin("http://127.0.0.1:5000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let app_state=APPLICATION_CONTEXT.get::<AppState>();

    let app = create_router(Arc::new(app_state.clone())).layer(cors);

    let binding_address = format!("{}:{}", app_state.conf.system.host, app_state.conf.system.port);

    let listener = tokio::net::TcpListener::bind(binding_address).await.unwrap();

    println!("🚀 服务成功启动",);


    axum::serve(listener, app).await.unwrap()

}
