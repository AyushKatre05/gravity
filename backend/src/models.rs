use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};


#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}


#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FileEntry {
    pub id: Uuid,
    pub project_id: Uuid,
    pub path: String,
    pub module_name: Option<String>,
    pub line_count: i32,
    pub created_at: DateTime<Utc>,
}
