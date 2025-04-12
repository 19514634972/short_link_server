use crate::APPLICATION_CONTEXT;
use tokio::fs::read_to_string;
use server_config::config::ApplicationConfig;
use std::path::Path;

//初始化配置信息
pub async fn init_config() -> Result<(), String> {
    let content = read_to_string("config.yml")
        .await
        .map_err(|e| format!("读取配置文件失败: {}", e))?;
        
    let config = ApplicationConfig::new(content.as_str());
    APPLICATION_CONTEXT.set::<ApplicationConfig>(config);
    Ok(())
}

