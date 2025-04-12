use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, sqlx::FromRow, Serialize, Clone)]
pub struct ShortVisitRecord {
    pub id: u32,
    pub short_link_id: u32,
    pub device_type: String,
    pub sys_type: String,
    pub browser_type: String,
    pub addr: String,
    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(rename = "deletedAt")]
    pub deleted_at: Option<DateTime<Utc>>,
}

