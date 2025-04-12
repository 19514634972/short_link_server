use axum::extract::ConnectInfo;
use std::net::SocketAddr;
use log::info;
use serde::{Deserialize, Serialize};
use reqwest::Client;

#[derive(Debug, Serialize, Deserialize)]
pub struct LocationInfo {
    pub ip: String,
    pub country: String,
    pub region: String,
    pub city: String,
    pub isp: String,
}

pub async fn get_location_info(ip: &str) -> Result<LocationInfo, Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = format!("http://ip-api.com/json/{}", ip);
    let response = client.get(&url)
        .send()
        .await?
        .json::<LocationInfo>()
        .await?;

    Ok(response)
}

pub async fn get_client_info(ConnectInfo(addr): ConnectInfo<SocketAddr>) -> Result<LocationInfo, Box<dyn std::error::Error>> {
    let ip = addr.ip().to_string();
    info!("Client ++++++++++++IP: {}", ip);
    get_location_info(&ip).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_location_info() {
        let result = get_location_info("8.8.8.8").await;
        assert!(result.is_ok());
        if let Ok(info) = result {
            println!("IP: {}", info.ip);
            println!("Country: {}", info.country);
            println!("Region: {}", info.region);
            println!("City: {}", info.city);
            println!("ISP: {}", info.isp);
        }
    }
}