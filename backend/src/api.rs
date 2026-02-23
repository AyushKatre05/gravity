use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    complexity,
    db,
    graph::DependencyGraph,
    models::{
        AnalyzeRequest, AnalyzeResponse, AnalysisSummary, ComplexityItem,
        FileEntry, GraphData,
    },
    parser,
};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub analyze_path: String,
}

pub fn build_router(state: AppState) -> Router {
    let shared = Arc::new(state);

    Router::new()
        .route("/api/analyze",    post(analyze_handler))
        .route("/api/summary",    get(summary_handler))
        .route("/api/files",      get(files_handler))
        .route("/api/graph",      get(graph_handler))
        .route("/api/complexity", get(complexity_handler))
        .route("/health",         get(health_handler))
        .with_state(shared)
}


#[derive(Debug, Deserialize)]
pub struct ProjectQuery {
    pub project_id: Option<Uuid>,
}
async fn analyze_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AnalyzeRequest>,
) -> Result<Json<AnalyzeResponse>, (StatusCode, String)> {
    let mut analyze_path = req.path.as_deref().unwrap_or(&state.analyze_path).to_owned();
    let mut project_name = req
        .project_name
        .as_deref()
        .unwrap_or("default")
        .to_owned();

    let mut temp_dir = None;

    if let Some(url) = &req.github_url {
        if !url.starts_with("https://github.com/") {
            return Err((StatusCode::BAD_REQUEST, "Only GitHub URLs (https://github.com/...) are supported".into()));
        }

        info!("Cloning GitHub repository: {url}");
        
        let id = Uuid::new_v4();
        let path = std::env::temp_dir().join(format!("gravity-clone-{id}"));
        
        // Ensure parent exists
        std::fs::create_dir_all(&path).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create temp dir: {e}")))?;

        let output = std::process::Command::new("git")
            .args(["clone", "--depth", "1", url, path.to_str().unwrap()])
            .output()
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Git clone failed to start: {e}")))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            error!("Git clone failed: {err}");
            return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Git clone failed: {err}")));
        }

        analyze_path = path.to_str().unwrap().to_owned();
        temp_dir = Some(path);

        // If project name was default, use repo name from URL
        if project_name == "default" {
            project_name = url.trim_end_matches('/').split('/').last().unwrap_or("default").to_owned();
        }
    }

    info!("Starting analysis of path: {analyze_path}");

    let result = (|| -> anyhow::Result<AnalyzeResponse> {
        let parsed_files = parser::parse_directory(&analyze_path)?;
        let _dep_graph = DependencyGraph::from_parsed(&parsed_files);
        let complexity_scores = complexity::compute_all(&parsed_files);
        let functions_found: usize = parsed_files.iter().map(|f| f.functions.len()).sum();
        
        Ok(AnalyzeResponse {
            project_id: Uuid::nil(), // Placeholder, will be updated
            files_analyzed: parsed_files.len(),
            functions_found,
            message: format!("Analyzed {} files", parsed_files.len()),
            parsed_files_internal: Some(parsed_files),
            complexity_scores_internal: Some(complexity_scores),
        })
    })();

    let mut res = result.map_err(|e| {
        error!("Analysis error: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;
    let project = db::upsert_project(&state.pool, &project_name, &analyze_path)
        .await
        .map_err(|e| {
            error!("DB upsert error: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;
    if let (Some(files), Some(scores)) = (res.parsed_files_internal.take(), res.complexity_scores_internal.take()) {
        db::save_analysis(&state.pool, project.id, &files, &scores)
            .await
            .map_err(|e| {
                error!("DB save error: {e}");
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            })?;
    }

    res.project_id = project.id;
    res.message = format!(
        "Analysis complete: {} files, {} functions",
        res.files_analyzed,
        res.functions_found
    );
    if let Some(path) = temp_dir {
        info!("Cleaning up temp directory: {:?}", path);
        let _ = std::fs::remove_dir_all(path);
    }
    Ok(Json(res))
}

async fn summary_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ProjectQuery>,
) -> Result<Json<AnalysisSummary>, (StatusCode, String)> {
    let project_id = resolve_project_id(&state.pool, params.project_id).await?;

    let summary = db::fetch_summary(&state.pool, project_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(summary))
}
async fn files_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ProjectQuery>,
) -> Result<Json<Vec<FileEntry>>, (StatusCode, String)> {
    let project_id = resolve_project_id(&state.pool, params.project_id).await?;

    let files = db::fetch_files(&state.pool, project_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(files))
}
async fn graph_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ProjectQuery>,
) -> Result<Json<GraphData>, (StatusCode, String)> {
    let project_id = resolve_project_id(&state.pool, params.project_id).await?;

    let graph = db::fetch_graph(&state.pool, project_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(graph))
}
async fn complexity_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ProjectQuery>,
) -> Result<Json<Vec<ComplexityItem>>, (StatusCode, String)> {
    let project_id = resolve_project_id(&state.pool, params.project_id).await?;

    let items = db::fetch_complexities(&state.pool, project_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(items))
}
async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn resolve_project_id(
    pool: &PgPool,
    provided: Option<Uuid>,
) -> Result<Uuid, (StatusCode, String)> {
    if let Some(id) = provided {
        return Ok(id);
    }

    sqlx::query_scalar!(
        "SELECT id FROM projects ORDER BY updated_at DESC LIMIT 1"
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            "No projects found. Run POST /api/analyze first.".into(),
        )
    })
}
