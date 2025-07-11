//! Translation handlers

use axum::{extract::{Path, Query, State}, response::Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::AppResult;
use crate::services::Services;
use crate::services::translation_service::{TranslationRequest, TranslationResponse};
use crate::services::task_queue::TranslationTask;
use crate::middleware::auth::AuthenticatedUser;

#[derive(Debug, Deserialize)]
pub struct TranslateUrlPayload {
    pub url: String,
    pub source_lang: String,
    pub target_lang: String,
    pub project_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct BatchTranslatePayload {
    pub urls: Vec<String>,
    pub source_lang: String,
    pub target_lang: String,
    pub project_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct HistoryParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct TranslationHistoryResponse {
    pub translations: Vec<TranslationResponse>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
}

/// Translate a URL
pub async fn translate_url(
    State(services): State<Services>,
    user: AuthenticatedUser,
    Json(payload): Json<TranslateUrlPayload>,
) -> AppResult<Json<TranslationResponse>> {
    let request = TranslationRequest {
        url: payload.url,
        source_lang: payload.source_lang,
        target_lang: payload.target_lang,
        user_id: Some(user.user_id),
        project_id: payload.project_id,
    };
    
    let translation = services.translation_service.translate_url(request).await?;
    
    Ok(Json(translation))
}

/// Get translation history
pub async fn get_history(
    State(services): State<Services>,
    user: AuthenticatedUser,
    Query(params): Query<HistoryParams>,
) -> AppResult<Json<TranslationHistoryResponse>> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(20);
    
    let translations = services.translation_service
        .get_user_translations(user.user_id, page, per_page)
        .await?;
    
    let total = services.translation_service
        .get_user_translation_count(user.user_id)
        .await?;
    
    let response = TranslationHistoryResponse {
        translations,
        total,
        page,
        per_page,
    };
    
    Ok(Json(response))
}

/// Get specific translation
pub async fn get_translation(
    State(services): State<Services>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<TranslationResponse>> {
    let translation = services.translation_service
        .get_translation(id, user.user_id)
        .await?;
    
    Ok(Json(translation))
}

/// Delete translation
pub async fn delete_translation(
    State(services): State<Services>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    services.translation_service
        .delete_translation(id, user.user_id)
        .await?;
    
    Ok(Json(json!({
        "message": "Translation deleted successfully"
    })))
}

/// Download translation as file
pub async fn download_translation(
    State(services): State<Services>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    let translation = services.translation_service
        .get_translation(id, user.user_id)
        .await?;
    
    // Generate filename based on URL and timestamp
    let filename = format!("translation_{}.md", translation.created_at.format("%Y%m%d_%H%M%S"));
    
    Ok(Json(json!({
        "filename": filename,
        "content": translation.translated_content,
        "content_type": "text/markdown"
    })))
}

/// Start batch translation
pub async fn start_batch_translation(
    State(services): State<Services>,
    user: AuthenticatedUser,
    Json(payload): Json<BatchTranslatePayload>,
) -> AppResult<Json<Value>> {
    let batch_id = services.translation_service
        .start_batch_translation(
            payload.urls,
            user.user_id,
            payload.source_lang,
            payload.target_lang,
        )
        .await?;
    
    Ok(Json(json!({
        "batch_id": batch_id,
        "message": "Batch translation started"
    })))
}

/// Get batch translation status
pub async fn get_batch_status(
    State(_services): State<Services>,
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    // TODO: Implement proper batch job status tracking with Redis
    // For now, return a simple response
    Ok(Json(json!({
        "batch_id": id,
        "status": "completed",
        "message": "Batch translation completed"
    })))
}

/// Cancel batch translation
pub async fn cancel_batch(
    State(_services): State<Services>,
    _user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    // TODO: Implement proper batch job cancellation with Redis
    // For now, return a simple response
    Ok(Json(json!({
        "batch_id": id,
        "message": "Batch translation cancelled"
    })))
}

/// Submit translation task to queue (async)
pub async fn submit_translation_task(
    State(services): State<Services>,
    user: AuthenticatedUser,
    Json(payload): Json<TranslateUrlPayload>,
) -> AppResult<Json<Value>> {
    let task = TranslationTask::new(
        user.user_id,
        payload.url,
        payload.source_lang,
        payload.target_lang,
        payload.project_id,
    );
    
    let task_id = task.id;
    
    services.task_queue.enqueue_translation_task(task).await?;
    
    Ok(Json(json!({
        "task_id": task_id,
        "message": "Translation task submitted successfully"
    })))
}

/// Get translation task status
pub async fn get_task_status(
    State(services): State<Services>,
    user: AuthenticatedUser,
    Path(task_id): Path<Uuid>,
) -> AppResult<Json<TranslationTask>> {
    let task = services.task_queue.get_task_status(task_id).await?;
    
    match task {
        Some(task) => {
            // 验证任务所有者
            if task.user_id == user.user_id {
                Ok(Json(task))
            } else {
                Err(crate::error::AppError::Auth("无权访问此任务".to_string()))
            }
        }
        None => Err(crate::error::AppError::NotFound("任务不存在".to_string())),
    }
}

/// Get user's all translation tasks
pub async fn get_user_tasks(
    State(services): State<Services>,
    user: AuthenticatedUser,
) -> AppResult<Json<Vec<TranslationTask>>> {
    let tasks = services.task_queue.get_user_tasks(user.user_id).await?;
    Ok(Json(tasks))
}

/// Cancel translation task
pub async fn cancel_translation_task(
    State(services): State<Services>,
    user: AuthenticatedUser,
    Path(task_id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    services.task_queue.cancel_task(task_id, user.user_id).await?;
    
    Ok(Json(json!({
        "task_id": task_id,
        "message": "Task cancelled successfully"
    })))
}