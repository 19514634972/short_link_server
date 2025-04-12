use  getset::{Getters,Setters,MutGetters};
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, Clone)]
pub struct SysConfig{
    pub  host: String,
    pub port: String,
    pub user: String,
    pub password: String,
}

#[derive(
    Debug, PartialEq, serde::Serialize, serde::Deserialize, Clone, Getters, Setters, MutGetters,
)]
pub struct LogCnf{
    pub log_dir: String,
    pub log_level: String,
}


#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, Clone)]
pub struct AccessToken{
    pub access_token_private_key:String,
    pub access_token_public_key: String,
    pub access_token_expired_in: String,
    pub access_token_maxage: String,
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, Clone)]
pub struct RefreshToken{
    pub refresh_token_private_key:String,
    pub refresh_token_public_key: String,
    pub refresh_token_expired_in: String,
    pub refresh_token_maxage: String,
}


#[derive(
    Debug, PartialEq, serde::Serialize, serde::Deserialize, Clone, Getters, Setters, MutGetters,
)]
#[getset(get_mut = "pub", get = "pub", set = "pub")]
pub struct ApplicationConfig {
    pub system:SysConfig,

    pub log: LogCnf,

    pub database_url: String,

    pub redis_url: String,
    pub redirect_url: String,

    pub access_token: AccessToken,

    pub refresh_token: RefreshToken,

}


impl ApplicationConfig {
    pub fn new(yml_data: &str) -> Self {
        let config = match serde_yaml::from_str(yml_data) {
            Ok(e) => e,
            Err(e) => panic!("{}", e),
        };
        config
    }
}
