//! Database models for the URL Translator backend

pub mod user;
pub mod translation;
pub mod project;
pub mod search;
pub mod config;

pub use user::*;
// pub use translation::*;
// pub use project::*;
// pub use search::*;
// pub use config::*;

// Re-export common types
pub use chrono::{DateTime, Utc};
pub use serde::{Deserialize, Serialize};
pub use sqlx::FromRow;
pub use uuid::Uuid;
pub use validator::Validate;