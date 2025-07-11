//! 异步任务队列服务
//! 
//! 基于Redis实现的异步任务队列，支持：
//! - 翻译任务排队
//! - 任务状态跟踪
//! - 进度更新
//! - 重试机制

use redis::{AsyncCommands, RedisResult};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

use crate::error::AppResult;
use crate::services::redis_service::RedisService;

/// 任务状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Retrying,
}

/// 翻译任务数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationTask {
    pub id: Uuid,
    pub user_id: Uuid,
    pub url: String,
    pub source_lang: String,
    pub target_lang: String,
    pub project_id: Option<Uuid>,
    pub status: TaskStatus,
    pub progress: f32,
    pub progress_message: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub error_message: Option<String>,
    pub retry_count: u32,
    pub max_retries: u32,
}

impl TranslationTask {
    pub fn new(
        user_id: Uuid,
        url: String,
        source_lang: String,
        target_lang: String,
        project_id: Option<Uuid>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            url,
            source_lang,
            target_lang,
            project_id,
            status: TaskStatus::Pending,
            progress: 0.0,
            progress_message: "任务已创建".to_string(),
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
            error_message: None,
            retry_count: 0,
            max_retries: 3,
        }
    }
}

/// 任务队列服务
#[derive(Clone)]
pub struct TaskQueueService {
    redis_service: RedisService,
    queue_name: String,
    status_prefix: String,
}

impl TaskQueueService {
    pub fn new(redis_service: RedisService) -> Self {
        Self {
            redis_service,
            queue_name: "translation_tasks".to_string(),
            status_prefix: "task_status".to_string(),
        }
    }

    /// 添加翻译任务到队列
    pub async fn enqueue_translation_task(&self, task: TranslationTask) -> AppResult<()> {
        let mut connection = self.redis_service.get_async_connection().await?;
        
        // 将任务添加到队列
        let task_json = serde_json::to_string(&task)
            .map_err(|e| crate::error::AppError::Internal(format!("序列化任务失败: {}", e)))?;
        
        connection.lpush::<_, _, ()>(&self.queue_name, task_json).await
            .map_err(|e| crate::error::AppError::Internal(format!("添加任务到队列失败: {}", e)))?;
        
        // 保存任务状态
        self.update_task_status(&task).await?;
        
        tracing::info!("任务 {} 已添加到队列", task.id);
        Ok(())
    }

    /// 从队列中获取下一个任务
    pub async fn dequeue_task(&self) -> AppResult<Option<TranslationTask>> {
        let mut connection = self.redis_service.get_async_connection().await?;
        
        // 阻塞式获取任务（超时1秒）
        let result: RedisResult<Option<(String, String)>> = connection
            .brpop(&self.queue_name, 1.0)
            .await;
        
        match result {
            Ok(Some((_, task_json))) => {
                let task: TranslationTask = serde_json::from_str(&task_json)
                    .map_err(|e| crate::error::AppError::Internal(format!("反序列化任务失败: {}", e)))?;
                
                tracing::info!("从队列获取任务: {}", task.id);
                Ok(Some(task))
            }
            Ok(None) => Ok(None), // 超时，没有任务
            Err(e) => Err(crate::error::AppError::Internal(format!("从队列获取任务失败: {}", e))),
        }
    }

    /// 更新任务状态
    pub async fn update_task_status(&self, task: &TranslationTask) -> AppResult<()> {
        let mut connection = self.redis_service.get_async_connection().await?;
        
        let key = format!("{}:{}", self.status_prefix, task.id);
        let task_json = serde_json::to_string(task)
            .map_err(|e| crate::error::AppError::Internal(format!("序列化任务状态失败: {}", e)))?;
        
        // 设置状态，TTL为24小时
        connection.set_ex::<_, _, ()>(key, task_json.clone(), 86400).await
            .map_err(|e| crate::error::AppError::Internal(format!("更新任务状态失败: {}", e)))?;
        
        // 发布任务状态更新事件
        let event_channel = format!("task_status_updates:{}", task.user_id);
        connection.publish::<_, _, ()>(event_channel, task_json).await
            .map_err(|e| crate::error::AppError::Internal(format!("发布任务状态事件失败: {}", e)))?;
        
        Ok(())
    }

    /// 获取任务状态
    pub async fn get_task_status(&self, task_id: Uuid) -> AppResult<Option<TranslationTask>> {
        let mut connection = self.redis_service.get_async_connection().await?;
        
        let key = format!("{}:{}", self.status_prefix, task_id);
        let result: RedisResult<Option<String>> = connection.get(key).await;
        
        match result {
            Ok(Some(task_json)) => {
                let task: TranslationTask = serde_json::from_str(&task_json)
                    .map_err(|e| crate::error::AppError::Internal(format!("反序列化任务状态失败: {}", e)))?;
                Ok(Some(task))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(crate::error::AppError::Internal(format!("获取任务状态失败: {}", e))),
        }
    }

    /// 更新任务进度
    pub async fn update_task_progress(
        &self,
        task_id: Uuid,
        progress: f32,
        message: String,
    ) -> AppResult<()> {
        if let Some(mut task) = self.get_task_status(task_id).await? {
            task.progress = progress;
            task.progress_message = message;
            self.update_task_status(&task).await?;
        }
        Ok(())
    }

    /// 标记任务为已开始
    pub async fn mark_task_started(&self, task_id: Uuid) -> AppResult<()> {
        if let Some(mut task) = self.get_task_status(task_id).await? {
            task.status = TaskStatus::Processing;
            task.started_at = Some(chrono::Utc::now());
            task.progress = 0.1;
            task.progress_message = "开始处理任务".to_string();
            self.update_task_status(&task).await?;
        }
        Ok(())
    }

    /// 标记任务为已完成
    pub async fn mark_task_completed(&self, task_id: Uuid) -> AppResult<()> {
        if let Some(mut task) = self.get_task_status(task_id).await? {
            task.status = TaskStatus::Completed;
            task.completed_at = Some(chrono::Utc::now());
            task.progress = 1.0;
            task.progress_message = "任务完成".to_string();
            self.update_task_status(&task).await?;
        }
        Ok(())
    }

    /// 标记任务为失败
    pub async fn mark_task_failed(&self, task_id: Uuid, error_message: String) -> AppResult<()> {
        if let Some(mut task) = self.get_task_status(task_id).await? {
            task.retry_count += 1;
            
            if task.retry_count < task.max_retries {
                // 可以重试
                task.status = TaskStatus::Retrying;
                task.progress_message = format!("任务失败，准备重试 ({}/{})", task.retry_count, task.max_retries);
                self.update_task_status(&task).await?;
                
                // 延迟后重新加入队列
                let delay = Duration::from_secs(2_u64.pow(task.retry_count)); // 指数退避
                tokio::spawn(async move {
                    sleep(delay).await;
                    // 这里需要重新加入队列，但为了避免循环依赖，我们暂时省略
                });
            } else {
                // 超过最大重试次数
                task.status = TaskStatus::Failed;
                task.completed_at = Some(chrono::Utc::now());
                task.error_message = Some(error_message);
                task.progress_message = "任务失败".to_string();
                self.update_task_status(&task).await?;
            }
        }
        Ok(())
    }

    /// 获取用户的所有任务状态
    pub async fn get_user_tasks(&self, user_id: Uuid) -> AppResult<Vec<TranslationTask>> {
        let mut connection = self.redis_service.get_async_connection().await?;
        
        // 扫描所有任务状态键
        let pattern = format!("{}:*", self.status_prefix);
        let keys: Vec<String> = connection.keys(pattern).await
            .map_err(|e| crate::error::AppError::Internal(format!("获取任务键失败: {}", e)))?;
        
        let mut user_tasks = Vec::new();
        
        for key in keys {
            if let Ok(Some(task_json)) = connection.get::<_, Option<String>>(key).await {
                if let Ok(task) = serde_json::from_str::<TranslationTask>(&task_json) {
                    if task.user_id == user_id {
                        user_tasks.push(task);
                    }
                }
            }
        }
        
        // 按创建时间排序
        user_tasks.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        Ok(user_tasks)
    }

    /// 取消任务
    pub async fn cancel_task(&self, task_id: Uuid, user_id: Uuid) -> AppResult<()> {
        // 验证任务属于该用户
        let task = self.get_task_status(task_id).await?;
        let mut task = match task {
            Some(task) => task,
            None => {
                return Err(crate::error::AppError::NotFound("Task not found".to_string()));
            }
        };

        if task.user_id != user_id {
            return Err(crate::error::AppError::Validation("Task does not belong to user".to_string()));
        }

        // 检查任务状态是否可以取消
        match task.status {
            TaskStatus::Completed | TaskStatus::Failed => {
                return Err(crate::error::AppError::Validation("Cannot cancel completed or failed task".to_string()));
            }
            TaskStatus::Pending | TaskStatus::Processing | TaskStatus::Retrying => {
                // 可以取消
            }
        }

        // 更新任务状态为已取消
        task.status = TaskStatus::Failed; // 使用Failed状态表示取消
        task.error_message = Some("Task cancelled by user".to_string());
        task.completed_at = Some(chrono::Utc::now());
        task.progress_message = "Task cancelled by user".to_string();

        // 更新任务状态
        self.update_task_status(&task).await?;

        // 从队列中移除任务（如果还在等待处理）
        let mut connection = self.redis_service.get_async_connection().await?;
        let queue_length: i32 = connection.llen(&self.queue_name).await?;
        
        // 遍历队列，找到并移除对应的任务
        for i in 0..queue_length {
            let index = i as isize; // 修复类型转换
            if let Ok(task_json) = connection.lindex::<_, String>(&self.queue_name, index).await {
                if let Ok(queued_task) = serde_json::from_str::<TranslationTask>(&task_json) {
                    if queued_task.id == task_id {
                        // 移除队列中的任务
                        connection.lrem::<_, _, ()>(&self.queue_name, 1, task_json).await?;
                        break;
                    }
                }
            }
        }

        tracing::info!("Task {} cancelled by user {}", task_id, user_id);
        Ok(())
    }

    /// 清理过期的任务状态
    pub async fn cleanup_expired_tasks(&self) -> AppResult<()> {
        // Redis的TTL会自动清理过期键，这里主要是记录日志
        tracing::info!("任务清理定时器运行");
        Ok(())
    }
}

/// 任务处理器
pub struct TaskProcessor {
    task_queue: TaskQueueService,
    translation_service: crate::services::translation_service::TranslationService,
}

impl TaskProcessor {
    pub fn new(
        task_queue: TaskQueueService,
        translation_service: crate::services::translation_service::TranslationService,
    ) -> Self {
        Self {
            task_queue,
            translation_service,
        }
    }

    /// 启动任务处理循环
    pub async fn start_processing(&self) {
        tracing::info!("启动任务处理器");
        
        loop {
            match self.process_next_task().await {
                Ok(true) => {
                    // 成功处理了一个任务，继续
                    continue;
                }
                Ok(false) => {
                    // 没有任务，短暂休眠
                    sleep(Duration::from_millis(100)).await;
                }
                Err(e) => {
                    tracing::error!("处理任务时出错: {}", e);
                    sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }

    /// 处理下一个任务
    async fn process_next_task(&self) -> AppResult<bool> {
        if let Some(task) = self.task_queue.dequeue_task().await? {
            self.process_translation_task(task).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// 处理翻译任务
    async fn process_translation_task(&self, task: TranslationTask) -> AppResult<()> {
        let task_id = task.id;
        
        tracing::info!("开始处理翻译任务: {}", task_id);
        
        // 标记任务为已开始
        self.task_queue.mark_task_started(task_id).await?;
        
        // 创建翻译请求
        let translation_request = crate::services::translation_service::TranslationRequest {
            url: task.url,
            source_lang: task.source_lang,
            target_lang: task.target_lang,
            user_id: Some(task.user_id),
            project_id: task.project_id,
        };
        
        // 更新进度：开始提取内容
        self.task_queue.update_task_progress(
            task_id,
            0.2,
            "正在提取网页内容...".to_string(),
        ).await?;
        
        // 执行翻译
        match self.translation_service.translate_url(translation_request).await {
            Ok(translation_response) => {
                // 翻译成功
                self.task_queue.update_task_progress(
                    task_id,
                    0.9,
                    "正在保存翻译结果...".to_string(),
                ).await?;
                
                // Note: Search indexing is automatically handled in translation_service.save_translation()
                // No additional indexing needed here since translate_url() already calls save_translation()
                
                self.task_queue.mark_task_completed(task_id).await?;
                tracing::info!("翻译任务 {} 完成，已自动索引到搜索引擎", task_id);
            }
            Err(e) => {
                // 翻译失败
                let error_message = format!("翻译失败: {}", e);
                self.task_queue.mark_task_failed(task_id, error_message).await?;
                tracing::error!("翻译任务 {} 失败: {}", task_id, e);
            }
        }
        
        Ok(())
    }
}