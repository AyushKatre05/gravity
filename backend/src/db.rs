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
pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .context("Failed to run migrations")?;

    Ok(())
}

pub async fn upsert_project(pool: &PgPool, name: &str, path: &str) -> Result<Project> {
    let existing = sqlx::query_as!(
        Project,
        r#"SELECT id, name, path, created_at, updated_at
           FROM projects WHERE name = $1 AND path = $2
           LIMIT 1"#,
        name,
        path
    )
    .fetch_optional(pool)
    .await?;

    if let Some(p) = existing {
        // Update timestamp
        sqlx::query!(
            "UPDATE projects SET updated_at = NOW() WHERE id = $1",
            p.id
        )
        .execute(pool)
        .await?;
        return Ok(p);
    }

    let project = sqlx::query_as!(
        Project,
        r#"INSERT INTO projects (id, name, path, created_at, updated_at)
           VALUES ($1, $2, $3, NOW(), NOW())
           RETURNING id, name, path, created_at, updated_at"#,
        Uuid::new_v4(),
        name,
        path
    )
    .fetch_one(pool)
    .await
    .context("Failed to insert project")?;

    Ok(project)
}

pub async fn save_analysis(
    pool: &PgPool,
    project_id: Uuid,
    parsed_files: &[ParsedFile],
    complexity_map: &[(String, String, usize)], // (file_path, fn_name, score)
) -> Result<()> {
    let mut tx = pool.begin().await?;

    sqlx::query!("DELETE FROM complexities WHERE project_id = $1", project_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query!("DELETE FROM functions WHERE project_id = $1", project_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query!("DELETE FROM dependencies WHERE project_id = $1", project_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query!("DELETE FROM files WHERE project_id = $1", project_id)
        .execute(&mut *tx)
        .await?;

    for parsed_file in parsed_files {
        let file_id = Uuid::new_v4();

        sqlx::query!(
            r#"INSERT INTO files (id, project_id, path, module_name, line_count, created_at)
               VALUES ($1, $2, $3, $4, $5, NOW())"#,
            file_id,
            project_id,
            parsed_file.path,
            parsed_file.module_name,
            parsed_file.line_count as i32,
        )
        .execute(&mut *tx)
        .await?;
        for func in &parsed_file.functions {
            let func_id = Uuid::new_v4();
            sqlx::query!(
                r#"INSERT INTO functions (id, project_id, file_id, name, line_start, line_end, is_public, is_async, created_at)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())"#,
                func_id,
                project_id,
                file_id,
                func.name,
                func.line_start as i32,
                func.line_end as i32,
                func.is_public,
                func.is_async,
            )
            .execute(&mut *tx)
            .await?;

            let score = complexity_map
                .iter()
                .find(|(fp, fn_name, _)| *fp == parsed_file.path && fn_name == &func.name)
                .map(|(_, _, s)| *s as i32)
                .unwrap_or(1);

            sqlx::query!(
                r#"INSERT INTO complexities (id, project_id, function_id, score, created_at)
                   VALUES ($1, $2, $3, $4, NOW())"#,
                Uuid::new_v4(),
                project_id,
                func_id,
                score,
            )
            .execute(&mut *tx)
            .await?;
        }
        for import_target in &parsed_file.imports {
            sqlx::query!(
                r#"INSERT INTO dependencies (id, project_id, file_id, source, target, kind, created_at)
                   VALUES ($1, $2, $3, $4, $5, 'use', NOW())"#,
                Uuid::new_v4(),
                project_id,
                file_id,
                parsed_file.path,
                import_target,
            )
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;
    Ok(())
}
