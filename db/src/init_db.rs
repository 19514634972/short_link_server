use sqlx::{MySql, Pool};
use sqlx::mysql::MySqlPoolOptions;
use std::time::Duration;
use server_config::config::ApplicationConfig;
// 初始化 MySQL 连接池
pub async fn conn_pool(cassie_config: &ApplicationConfig) -> Pool<MySql> {
    MySqlPoolOptions::new()
        .max_connections(5) // 设置最大连接数
        .acquire_timeout(Duration::from_secs(30)) // 设置连接超时时间
        .connect(cassie_config.database_url())
        .await
        .expect("链接mysql失败")
}