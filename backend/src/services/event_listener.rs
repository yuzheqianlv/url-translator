//! Redis事件监听服务
//! 
//! 监听任务状态更新事件并通过WebSocket发送给客户端

use futures_util::StreamExt;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::handlers::websocket::{WebSocketManager, WsMessage};
use crate::services::{redis_service::RedisService, task_queue::TranslationTask};

#[derive(Clone)]
pub struct EventListener {
    redis_service: RedisService,
    websocket_manager: WebSocketManager,
}

impl EventListener {
    pub fn new(redis_service: RedisService, websocket_manager: WebSocketManager) -> Self {
        Self {
            redis_service,
            websocket_manager,
        }
    }

    /// 启动事件监听器
    pub async fn start(&self) {
        info!("启动Redis事件监听器");
        
        loop {
            match self.listen_for_events().await {
                Ok(()) => {
                    // 连接断开，等待重连
                    warn!("Redis事件监听连接断开，1秒后重连");
                    sleep(Duration::from_secs(1)).await;
                }
                Err(e) => {
                    error!("Redis事件监听错误: {}, 1秒后重试", e);
                    sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }

    /// 监听Redis发布/订阅事件
    async fn listen_for_events(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = self.redis_service.get_client();
        let mut pubsub = client.get_async_connection().await?.into_pubsub();

        // 订阅所有任务状态更新频道
        pubsub.psubscribe("task_status_updates:*").await?;
        
        info!("已订阅任务状态更新事件");

        let mut stream = pubsub.on_message();
        while let Some(msg) = stream.next().await {
            if let Err(e) = self.handle_message(msg).await {
                error!("处理消息时出错: {}", e);
            }
        }

        Ok(())
    }

    /// 处理接收到的消息
    async fn handle_message(&self, msg: redis::Msg) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let channel: String = msg.get_channel_name().to_string();
        let payload: String = msg.get_payload()?;

        // 解析频道名获取用户ID
        if let Some(user_id_str) = channel.strip_prefix("task_status_updates:") {
            if let Ok(user_id) = Uuid::parse_str(user_id_str) {
                // 解析任务数据
                if let Ok(task) = serde_json::from_str::<TranslationTask>(&payload) {
                    // 创建WebSocket消息
                    let ws_message = WsMessage::TaskUpdate {
                        task_id: task.id.to_string(),
                        status: format!("{:?}", task.status),
                        progress: task.progress,
                        progress_message: task.progress_message.clone(),
                        error_message: task.error_message.clone(),
                    };

                    // 发送WebSocket通知
                    self.websocket_manager.broadcast_task_update(&task.id.to_string(), ws_message).await;
                }
            }
        }

        Ok(())
    }
}