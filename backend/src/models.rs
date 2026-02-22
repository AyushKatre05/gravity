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

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FunctionEntry {
    pub id: Uuid,
    pub project_id: Uuid,
    pub file_id: Uuid,
    pub name: String,
    pub line_start: i32,
    pub line_end: i32,
    pub is_public: bool,
    pub is_async: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Dependency {
    pub id: Uuid,
    pub project_id: Uuid,
    pub file_id: Uuid,
    pub source: String,
    pub target: String,
    pub kind: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ComplexityEntry {
    pub id: Uuid,
    pub project_id: Uuid,
    pub function_id: Uuid,
    pub score: i32,
    pub created_at: DateTime<Utc>,
}

// ─── Parsed (in-memory, pre-DB) ────────────────────────────────────────────

/// Parsed file data before persisting to the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedFile {
    pub path: String,
    pub module_name: Option<String>,
    pub line_count: usize,
    pub functions: Vec<ParsedFunction>,
    pub imports: Vec<String>,
    pub structs: Vec<String>,
}

/// Parsed function data before persisting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedFunction {
    pub name: String,
    pub line_start: usize,
    pub line_end: usize,
    pub is_public: bool,
    pub is_async: bool,
    pub body_source: String,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSummary {
    pub project_id: Uuid,
    pub project_name: String,
    pub total_files: i64,
    pub total_functions: i64,
    pub total_structs: i64,
    pub total_imports: i64,
    pub avg_complexity: f64,
    pub dead_code_candidates: Vec<String>,
    pub architecture_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub kind: String, // "file" | "module" | "extern"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityItem {
    pub function_name: String,
    pub file_path: String,
    pub score: i32,
    pub line_start: i32,
    pub line_end: i32,
}
