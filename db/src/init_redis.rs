use redis::Client;
use server_config::config::ApplicationConfig;
pub async fn init_redis(conf :&ApplicationConfig)->Client{
    let redis_client = match Client::open(conf.redis_url.to_owned()) {
        Ok(client) => {
            println!("✅成功链redis!");
            client
        }
        Err(e) => {
            println!("🔥 redis链接失败: {}", e);
            std::process::exit(1);
        }
    };

    redis_client
}