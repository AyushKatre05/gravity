use anyhow::{Context, Result};
use sqlx::{PgPool, postgres::PgPoolOptions};
use uuid::Uuid;

use crate::models::{
    AnalysisSummary, ComplexityItem, Dependency, FileEntry,
    FunctionEntry, GraphData, GraphEdge, GraphNode, ParsedFile,
    ParsedFunction, Project,
};
pub async fn init_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .context("Failed to connect to Postgres")?;

    Ok(pool)
}
