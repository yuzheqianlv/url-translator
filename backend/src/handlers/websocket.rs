//! WebSocket handlers for real-time communication

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::middleware::auth::AuthenticatedUser;
use crate::services::Services;

/// WebSocket消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    /// 订阅任务更新
    Subscribe {
        task_id: String,
    },
    /// 取消订阅任务更新
    Unsubscribe {
        task_id: String,
    },
    /// 任务状态更新
    TaskUpdate {
        task_id: String,
        status: String,
        progress: f32,
        progress_message: String,
        error_message: Option<String>,
    },
    /// 连接确认
    Connected {
        user_id: String,
    },
    /// 错误消息
    Error {
        message: String,
    },
    /// Ping/Pong保持连接
    Ping,
    Pong,
}

/// WebSocket连接管理器
#[derive(Clone)]
pub struct WebSocketManager {
    /// 存储每个用户的WebSocket发送器
    connections: Arc<RwLock<HashMap<Uuid, broadcast::Sender<WsMessage>>>>,
    /// 任务订阅映射：task_id -> [user_id]
    task_subscribers: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
}

impl WebSocketManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            task_subscribers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 添加新的WebSocket连接
    pub async fn add_connection(&self, user_id: Uuid, sender: broadcast::Sender<WsMessage>) {
        self.connections.write().await.insert(user_id, sender);
        info!("WebSocket connection added for user: {}", user_id);
    }

    /// 移除WebSocket连接
    pub async fn remove_connection(&self, user_id: Uuid) {
        self.connections.write().await.remove(&user_id);
        
        // 清理该用户的所有任务订阅
        let mut subscribers = self.task_subscribers.write().await;
        for (_task_id, user_list) in subscribers.iter_mut() {
            user_list.retain(|&id| id != user_id);
        }
        
        info!("WebSocket connection removed for user: {}", user_id);
    }

    /// 用户订阅任务更新
    pub async fn subscribe_task(&self, user_id: Uuid, task_id: String) {
        let mut subscribers = self.task_subscribers.write().await;
        subscribers
            .entry(task_id.clone())
            .or_insert_with(Vec::new)
            .push(user_id);
        
        info!("User {} subscribed to task: {}", user_id, task_id);
    }

    /// 用户取消订阅任务更新
    pub async fn unsubscribe_task(&self, user_id: Uuid, task_id: String) {
        let mut subscribers = self.task_subscribers.write().await;
        if let Some(user_list) = subscribers.get_mut(&task_id) {
            user_list.retain(|&id| id != user_id);
            if user_list.is_empty() {
                subscribers.remove(&task_id);
            }
        }
        
        info!("User {} unsubscribed from task: {}", user_id, task_id);
    }

    /// 向订阅了特定任务的用户发送更新
    pub async fn broadcast_task_update(&self, task_id: &str, message: WsMessage) {
        let subscribers = self.task_subscribers.read().await;
        if let Some(user_list) = subscribers.get(task_id) {
            let connections = self.connections.read().await;
            
            for &user_id in user_list {
                if let Some(sender) = connections.get(&user_id) {
                    if let Err(e) = sender.send(message.clone()) {
                        warn!("Failed to send message to user {}: {}", user_id, e);
                    }
                }
            }
        }
    }

    /// 向特定用户发送消息
    pub async fn send_to_user(&self, user_id: Uuid, message: WsMessage) {
        let connections = self.connections.read().await;
        if let Some(sender) = connections.get(&user_id) {
            if let Err(e) = sender.send(message) {
                warn!("Failed to send message to user {}: {}", user_id, e);
            }
        }
    }
}

/// WebSocket升级处理器
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(services): State<Services>,
    user: AuthenticatedUser,
) -> Response {
    ws.on_upgrade(move |socket| handle_websocket(socket, services, user))
}

/// 处理WebSocket连接
async fn handle_websocket(socket: WebSocket, services: Services, user: AuthenticatedUser) {
    let user_id = user.user_id;
    let (sender, mut receiver) = socket.split();
    
    // 创建广播通道
    let (tx, rx) = broadcast::channel(100);
    
    // 将连接添加到管理器
    services.websocket_manager.add_connection(user_id, tx.clone()).await;
    
    // 发送连接确认
    let connected_msg = WsMessage::Connected {
        user_id: user_id.to_string(),
    };
    if let Err(e) = tx.send(connected_msg) {
        error!("Failed to send connected message: {}", e);
    }
    
    // 启动消息发送任务
    let send_task = {
        let mut rx = rx.resubscribe();
        let mut sender = sender;
        
        tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                let json_msg = serde_json::to_string(&msg).unwrap_or_else(|_| {
                    r#"{"type":"Error","message":"Failed to serialize message"}"#.to_string()
                });
                
                if sender.send(Message::Text(json_msg)).await.is_err() {
                    break;
                }
            }
        })
    };
    
    // 启动消息接收任务
    let recv_task = {
        let services = services.clone();
        let tx = tx.clone();
        
        tokio::spawn(async move {
            while let Some(msg) = receiver.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                            handle_websocket_message(ws_msg, user_id, &services, &tx).await;
                        } else {
                            warn!("Received invalid JSON message: {}", text);
                        }
                    }
                    Ok(Message::Binary(_)) => {
                        warn!("Received binary message, ignoring");
                    }
                    Ok(Message::Close(_)) => {
                        info!("WebSocket connection closed by client");
                        break;
                    }
                    Ok(Message::Ping(_data)) => {
                        // 发送Pong响应
                        let pong_msg = WsMessage::Pong;
                        if let Err(e) = tx.send(pong_msg) {
                            error!("Failed to send pong: {}", e);
                        }
                    }
                    Ok(Message::Pong(_)) => {
                        // 忽略Pong消息
                    }
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                }
            }
        })
    };
    
    // 等待任务完成
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
    
    // 清理连接
    services.websocket_manager.remove_connection(user_id).await;
}

/// 处理WebSocket消息
async fn handle_websocket_message(
    message: WsMessage,
    user_id: Uuid,
    services: &Services,
    sender: &broadcast::Sender<WsMessage>,
) {
    match message {
        WsMessage::Subscribe { task_id } => {
            // 验证用户是否有权限访问该任务
            if let Ok(task_id_uuid) = Uuid::parse_str(&task_id) {
                if let Ok(Some(task)) = services.task_queue.get_task_status(task_id_uuid).await {
                    if task.user_id == user_id {
                        services.websocket_manager.subscribe_task(user_id, task_id).await;
                    } else {
                        let error_msg = WsMessage::Error {
                            message: "Unauthorized to access this task".to_string(),
                        };
                        if let Err(e) = sender.send(error_msg) {
                            error!("Failed to send error message: {}", e);
                        }
                    }
                } else {
                    let error_msg = WsMessage::Error {
                        message: "Task not found".to_string(),
                    };
                    if let Err(e) = sender.send(error_msg) {
                        error!("Failed to send error message: {}", e);
                    }
                }
            } else {
                let error_msg = WsMessage::Error {
                    message: "Invalid task ID format".to_string(),
                };
                if let Err(e) = sender.send(error_msg) {
                    error!("Failed to send error message: {}", e);
                }
            }
        }
        WsMessage::Unsubscribe { task_id } => {
            services.websocket_manager.unsubscribe_task(user_id, task_id).await;
        }
        WsMessage::Ping => {
            let pong_msg = WsMessage::Pong;
            if let Err(e) = sender.send(pong_msg) {
                error!("Failed to send pong: {}", e);
            }
        }
        _ => {
            warn!("Received unexpected message type from client");
        }
    }
}