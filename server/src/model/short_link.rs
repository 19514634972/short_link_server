use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct ShortLink {
    pub id: u32,
    pub name: String,
    pub full_url: String,
    pub short_url: String,
    pub short_code: String,
    pub visit_count: u32,
    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,

    #[serde(rename = "deletedAt")]
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct SaveLinkReq{
    pub name: String,
    pub full_url: String,
}


#[derive(Debug,Serialize,Deserialize)]
pub struct UpdateLinkReq{
    pub name:String,
    pub short_code: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,
}



#[derive(Debug,Deserialize)]
pub struct ShortLinkListReq{
    pub page:Option<u32>,
    pub pageSize:Option<u32>,
}

#[derive(Debug)]
pub struct Pagination {
    pub page: u32,
    pub size: u32,
}

