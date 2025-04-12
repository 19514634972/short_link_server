use redis::Client;
use sqlx::{Pool, MySql};
use server_config::config::ApplicationConfig;
use crate::APPLICATION_CONTEXT;
use db::init_db::conn_pool;
use db::init_redis::init_redis;



#[derive(Clone)]
pub struct AppState {
    pub db: Pool<MySql>,
    pub conf: ApplicationConfig,
    pub redis_client: Client,
}



impl AppState {
    pub fn new(db: Pool<MySql>, conf: ApplicationConfig, redis_client: Client) -> Self {
        AppState { db, conf, redis_client }
    }
}


pub async fn init_state(){
    let cassie_config = APPLICATION_CONTEXT.get::<ApplicationConfig>();
    let pool=conn_pool(cassie_config).await;
    let redis_conn=init_redis(cassie_config).await;
    let app=AppState::new(pool, cassie_config.clone(), redis_conn);

    APPLICATION_CONTEXT.set::<AppState>(app);
}
