//! Project-related database models

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Project {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateProjectRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    
    #[validate(length(max = 1000))]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateProjectRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
    
    #[validate(length(max = 1000))]
    pub description: Option<String>,
    
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub translation_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectListResponse {
    pub projects: Vec<ProjectResponse>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStatistics {
    pub total_translations: i64,
    pub total_characters_translated: i64,
    pub average_translation_time_ms: Option<f64>,
    pub most_used_source_language: Option<String>,
    pub most_used_target_language: Option<String>,
    pub recent_activity: Vec<RecentActivity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentActivity {
    pub translation_id: Uuid,
    pub url: String,
    pub title: Option<String>,
    pub source_language: String,
    pub target_language: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AddUrlsToProjectRequest {
    #[validate(length(min = 1, max = 100))]
    pub urls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectExportRequest {
    pub format: ExportFormat,
    pub include_original: bool,
    pub include_metadata: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Csv,
    Markdown,
    Zip,
}

impl From<Project> for ProjectResponse {
    fn from(project: Project) -> Self {
        Self {
            id: project.id,
            name: project.name,
            description: project.description,
            is_active: project.is_active,
            translation_count: 0, // This will be populated by the service layer
            created_at: project.created_at,
            updated_at: project.updated_at,
        }
    }
}

impl Project {
    /// Create a new project
    pub fn new(user_id: Uuid, name: String, description: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            name,
            description,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Update project details
    pub fn update(&mut self, request: UpdateProjectRequest) {
        if let Some(name) = request.name {
            self.name = name;
        }
        if let Some(description) = request.description {
            self.description = Some(description);
        }
        if let Some(is_active) = request.is_active {
            self.is_active = is_active;
        }
        self.updated_at = Utc::now();
    }
}

impl ToString for ExportFormat {
    fn to_string(&self) -> String {
        match self {
            ExportFormat::Json => "json".to_string(),
            ExportFormat::Csv => "csv".to_string(),
            ExportFormat::Markdown => "markdown".to_string(),
            ExportFormat::Zip => "zip".to_string(),
        }
    }
}