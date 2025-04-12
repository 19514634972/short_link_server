pub mod model;
pub mod handler;
pub mod utils;
pub mod router;
pub mod initialize;
pub mod logs;

use crate::logs::log::init_log;
use crate::initialize::config::init_config;
use server_config::config::ApplicationConfig;
use crate::initialize::app_state::init_state;


use log::info;
use state::Container;
pub static APPLICATION_CONTEXT: Container![Send + Sync] = <Container![Send + Sync]>::new();
/*初始化环境上下文*/
pub async fn init_context() {
    init_config().await;
    info!("初始化配置完成");
    init_log();
    info!("初始化完成");
    init_state().await;
}

