//! Backend API客户端服务
//! 
//! 这个模块提供了与后端API的完整集成，包括：
//! - 用户认证和管理
//! - 翻译功能
//! - 历史记录管理
//! - 搜索功能
//! - 项目管理

use reqwest::{Client, header::HeaderMap};
use serde::{Deserialize, Serialize};
use crate::config::EnvConfig;

/// API客户端配置
#[derive(Clone, Debug)]
pub struct ApiConfig {
    pub base_url: String,
    pub timeout_seconds: u64,
}

impl Default for ApiConfig {
    fn default() -> Self {
        let env_config = EnvConfig::global();
        Self {
            base_url: env_config.api_base_url.clone(),
            timeout_seconds: env_config.api_timeout_seconds,
        }
    }
}

impl From<&EnvConfig> for ApiConfig {
    fn from(env_config: &EnvConfig) -> Self {
        Self {
            base_url: env_config.api_base_url.clone(),
            timeout_seconds: env_config.api_timeout_seconds,
        }
    }
}

/// API客户端
#[derive(Clone)]
pub struct ApiClient {
    client: Client,
    config: ApiConfig,
    auth_token: Option<String>,
}

impl ApiClient {
    /// 创建新的API客户端
    pub fn new(config: ApiConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            config,
            auth_token: None,
        }
    }

    /// 设置认证token
    pub fn set_auth_token(&mut self, token: Option<String>) {
        self.auth_token = token;
    }

    /// 获取认证headers
    fn get_auth_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());
        
        if let Some(token) = &self.auth_token {
            headers.insert(
                "Authorization", 
                format!("Bearer {}", token).parse().unwrap()
            );
        }
        
        headers
    }

    /// 构建完整URL
    fn build_url(&self, endpoint: &str) -> String {
        format!("{}{}", self.config.base_url, endpoint)
    }
}

// ============== 认证相关数据结构 ==============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: String,
    pub username: String,
    pub email: String,
    pub is_active: bool,
    pub created_at: String,
    pub last_login_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub user: UserProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub data: Option<T>,
    pub error: Option<ApiError>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: u16,
    pub message: String,
}

// ============== 翻译相关数据结构 ==============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslateUrlRequest {
    pub url: String,
    pub source_language: String,
    pub target_language: String,
    pub project_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResponse {
    pub id: String,
    pub url: String,
    pub title: Option<String>,
    pub original_content: String,
    pub translated_content: String,
    pub source_language: String,
    pub target_language: String,
    pub translation_time_ms: Option<i32>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationListResponse {
    pub translations: Vec<TranslationResponse>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

// ============== 项目管理数据结构 ==============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
    pub source_language: String,
    pub target_language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: String,
    pub description: Option<String>,
    pub source_language: String,
    pub target_language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub source_language: String,
    pub target_language: String,
    pub is_active: bool,
    pub translation_count: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectResponse {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub source_language: String,
    pub target_language: String,
    pub is_active: bool,
    pub translation_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectListResponse {
    pub projects: Vec<Project>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

// ============== 搜索相关数据结构 ==============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub project_id: Option<String>,
    pub source_language: Option<String>,
    pub target_language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub translation_id: String,
    pub url: String,
    pub title: Option<String>,
    pub content_snippet: String,
    pub source_language: String,
    pub target_language: String,
    pub project_name: Option<String>,
    pub created_at: String,
    pub relevance_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
    pub query: String,
    pub search_time_ms: u64,
    pub suggestions: Vec<String>,
}

// ============== 用户配置数据结构 ==============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfigResponse {
    pub deeplx_api_url: Option<String>,
    pub jina_api_url: String,
    pub default_source_lang: String,
    pub default_target_lang: String,
    pub max_requests_per_second: i32,
    pub max_text_length: i32,
    pub max_paragraphs_per_request: i32,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserConfigRequest {
    pub deeplx_api_url: Option<String>,
    pub jina_api_url: Option<String>,
    pub default_source_lang: Option<String>,
    pub default_target_lang: Option<String>,
    pub max_requests_per_second: Option<i32>,
    pub max_text_length: Option<i32>,
    pub max_paragraphs_per_request: Option<i32>,
}

// ============== API客户端实现 ==============

impl ApiClient {
    // ============== 认证API ==============

    /// 用户登录
    pub async fn login(&self, request: LoginRequest) -> Result<LoginResponse, String> {
        let url = self.build_url("/auth/login");
        
        let response = self.client
            .post(&url)
            .headers(self.get_auth_headers())
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("登录请求失败: {}", e))?;

        if response.status().is_success() {
            let login_response: LoginResponse = response
                .json()
                .await
                .map_err(|e| format!("解析登录响应失败: {}", e))?;
            Ok(login_response)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "未知错误".to_string());
            Err(format!("登录失败: {}", error_text))
        }
    }

    /// 用户注册
    pub async fn register(&self, request: RegisterRequest) -> Result<UserProfile, String> {
        let url = self.build_url("/auth/register");
        
        let response = self.client
            .post(&url)
            .headers(self.get_auth_headers())
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("注册请求失败: {}", e))?;

        if response.status().is_success() {
            let api_response: serde_json::Value = response
                .json()
                .await
                .map_err(|e| format!("解析注册响应失败: {}", e))?;
            
            let user: UserProfile = serde_json::from_value(api_response["user"].clone())
                .map_err(|e| format!("解析用户信息失败: {}", e))?;
            
            Ok(user)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "未知错误".to_string());
            Err(format!("注册失败: {}", error_text))
        }
    }

    /// 获取用户配置
    pub async fn get_user_config(&self) -> Result<UserConfigResponse, String> {
        let url = self.build_url("/users/config");
        
        let response = self.client
            .get(&url)
            .headers(self.get_auth_headers())
            .send()
            .await
            .map_err(|e| format!("获取用户配置失败: {}", e))?;

        if response.status().is_success() {
            let config: UserConfigResponse = response
                .json()
                .await
                .map_err(|e| format!("解析用户配置失败: {}", e))?;
            Ok(config)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "未知错误".to_string());
            Err(format!("获取用户配置失败: {}", error_text))
        }
    }

    /// 更新用户配置
    pub async fn update_user_config(&self, request: UpdateUserConfigRequest) -> Result<(), String> {
        let url = self.build_url("/users/config");
        
        let response = self.client
            .put(&url)
            .headers(self.get_auth_headers())
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("更新用户配置失败: {}", e))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "未知错误".to_string());
            Err(format!("更新用户配置失败: {}", error_text))
        }
    }

    // ============== 翻译API ==============

    /// 翻译URL
    pub async fn translate_url(&self, request: TranslateUrlRequest) -> Result<TranslationResponse, String> {
        let url = self.build_url("/translations/translate");
        
        let response = self.client
            .post(&url)
            .headers(self.get_auth_headers())
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("翻译请求失败: {}", e))?;

        if response.status().is_success() {
            let translation: TranslationResponse = response
                .json()
                .await
                .map_err(|e| format!("解析翻译响应失败: {}", e))?;
            Ok(translation)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "未知错误".to_string());
            Err(format!("翻译失败: {}", error_text))
        }
    }

    /// 获取翻译历史
    pub async fn get_translation_history(&self, page: Option<u32>, per_page: Option<u32>) -> Result<TranslationListResponse, String> {
        let mut url = self.build_url("/translations/history");
        
        let mut params = Vec::new();
        if let Some(p) = page {
            params.push(format!("page={}", p));
        }
        if let Some(pp) = per_page {
            params.push(format!("per_page={}", pp));
        }
        
        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }
        
        let response = self.client
            .get(&url)
            .headers(self.get_auth_headers())
            .send()
            .await
            .map_err(|e| format!("获取翻译历史失败: {}", e))?;

        if response.status().is_success() {
            let history: TranslationListResponse = response
                .json()
                .await
                .map_err(|e| format!("解析翻译历史失败: {}", e))?;
            Ok(history)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "未知错误".to_string());
            Err(format!("获取翻译历史失败: {}", error_text))
        }
    }

    /// 获取特定翻译
    pub async fn get_translation(&self, translation_id: &str) -> Result<TranslationResponse, String> {
        let url = self.build_url(&format!("/translations/history/{}", translation_id));
        
        let response = self.client
            .get(&url)
            .headers(self.get_auth_headers())
            .send()
            .await
            .map_err(|e| format!("获取翻译详情失败: {}", e))?;

        if response.status().is_success() {
            let translation: TranslationResponse = response
                .json()
                .await
                .map_err(|e| format!("解析翻译详情失败: {}", e))?;
            Ok(translation)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "未知错误".to_string());
            Err(format!("获取翻译详情失败: {}", error_text))
        }
    }

    /// 删除翻译
    pub async fn delete_translation(&self, translation_id: &str) -> Result<(), String> {
        let url = self.build_url(&format!("/translations/history/{}", translation_id));
        
        let response = self.client
            .delete(&url)
            .headers(self.get_auth_headers())
            .send()
            .await
            .map_err(|e| format!("删除翻译失败: {}", e))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "未知错误".to_string());
            Err(format!("删除翻译失败: {}", error_text))
        }
    }

    // ============== 项目管理API ==============

    /// 获取项目列表
    pub async fn get_projects(&self, page: Option<u32>, per_page: Option<u32>) -> Result<ProjectListResponse, String> {
        let mut url = self.build_url("/projects");
        
        let mut params = Vec::new();
        if let Some(p) = page {
            params.push(format!("page={}", p));
        }
        if let Some(pp) = per_page {
            params.push(format!("per_page={}", pp));
        }
        
        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }
        
        let response = self.client
            .get(&url)
            .headers(self.get_auth_headers())
            .send()
            .await
            .map_err(|e| format!("获取项目列表失败: {}", e))?;

        if response.status().is_success() {
            let projects: ProjectListResponse = response
                .json()
                .await
                .map_err(|e| format!("解析项目列表失败: {}", e))?;
            Ok(projects)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "未知错误".to_string());
            Err(format!("获取项目列表失败: {}", error_text))
        }
    }

    /// 创建项目
    pub async fn create_project(&self, request: CreateProjectRequest) -> Result<ProjectResponse, String> {
        let url = self.build_url("/projects");
        
        let response = self.client
            .post(&url)
            .headers(self.get_auth_headers())
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("创建项目失败: {}", e))?;

        if response.status().is_success() {
            let project: ProjectResponse = response
                .json()
                .await
                .map_err(|e| format!("解析项目响应失败: {}", e))?;
            Ok(project)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "未知错误".to_string());
            Err(format!("创建项目失败: {}", error_text))
        }
    }

    /// 更新项目
    pub async fn update_project(&self, project_id: i64, request: UpdateProjectRequest) -> Result<ProjectResponse, String> {
        let url = self.build_url(&format!("/projects/{}", project_id));
        
        let response = self.client
            .put(&url)
            .headers(self.get_auth_headers())
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("更新项目失败: {}", e))?;

        if response.status().is_success() {
            let project: ProjectResponse = response
                .json()
                .await
                .map_err(|e| format!("解析项目响应失败: {}", e))?;
            Ok(project)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "未知错误".to_string());
            Err(format!("更新项目失败: {}", error_text))
        }
    }

    /// 删除项目
    pub async fn delete_project(&self, project_id: i64) -> Result<(), String> {
        let url = self.build_url(&format!("/projects/{}", project_id));
        
        let response = self.client
            .delete(&url)
            .headers(self.get_auth_headers())
            .send()
            .await
            .map_err(|e| format!("删除项目失败: {}", e))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "未知错误".to_string());
            Err(format!("删除项目失败: {}", error_text))
        }
    }

    // ============== 搜索API ==============

    /// 搜索翻译
    pub async fn search_translations(&self, request: SearchRequest) -> Result<SearchResponse, String> {
        let mut url = self.build_url("/search");
        
        let mut params = Vec::new();
        params.push(format!("query={}", urlencoding::encode(&request.query)));
        
        if let Some(page) = request.page {
            params.push(format!("page={}", page));
        }
        if let Some(per_page) = request.per_page {
            params.push(format!("per_page={}", per_page));
        }
        if let Some(project_id) = &request.project_id {
            params.push(format!("project_id={}", project_id));
        }
        if let Some(source_lang) = &request.source_language {
            params.push(format!("source_language={}", source_lang));
        }
        if let Some(target_lang) = &request.target_language {
            params.push(format!("target_language={}", target_lang));
        }
        
        url.push('?');
        url.push_str(&params.join("&"));
        
        let response = self.client
            .get(&url)
            .headers(self.get_auth_headers())
            .send()
            .await
            .map_err(|e| format!("搜索失败: {}", e))?;

        if response.status().is_success() {
            let search_result: SearchResponse = response
                .json()
                .await
                .map_err(|e| format!("解析搜索结果失败: {}", e))?;
            Ok(search_result)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "未知错误".to_string());
            Err(format!("搜索失败: {}", error_text))
        }
    }

    /// 获取搜索建议
    pub async fn get_search_suggestions(&self, query: &str, limit: Option<u32>) -> Result<Vec<String>, String> {
        let mut url = self.build_url("/search/suggestions");
        
        let mut params = Vec::new();
        params.push(format!("query={}", urlencoding::encode(query)));
        if let Some(l) = limit {
            params.push(format!("limit={}", l));
        }
        
        url.push('?');
        url.push_str(&params.join("&"));
        
        let response = self.client
            .get(&url)
            .headers(self.get_auth_headers())
            .send()
            .await
            .map_err(|e| format!("获取搜索建议失败: {}", e))?;

        if response.status().is_success() {
            let suggestions_response: serde_json::Value = response
                .json()
                .await
                .map_err(|e| format!("解析搜索建议失败: {}", e))?;
            
            let suggestions: Vec<String> = serde_json::from_value(suggestions_response["suggestions"].clone())
                .map_err(|e| format!("解析搜索建议数据失败: {}", e))?;
            
            Ok(suggestions)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "未知错误".to_string());
            Err(format!("获取搜索建议失败: {}", error_text))
        }
    }
}