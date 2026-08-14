use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::fmt;

/// 异步任务状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "pending"),
            TaskStatus::Running => write!(f, "running"),
            TaskStatus::Completed => write!(f, "completed"),
            TaskStatus::Failed => write!(f, "failed"),
            TaskStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl std::str::FromStr for TaskStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(TaskStatus::Pending),
            "running" => Ok(TaskStatus::Running),
            "completed" => Ok(TaskStatus::Completed),
            "failed" => Ok(TaskStatus::Failed),
            "cancelled" => Ok(TaskStatus::Cancelled),
            _ => Err(format!("Invalid task status: {}", s)),
        }
    }
}

/// 异步任务类型
pub mod task_types {
    pub const EMBED_ELEMENT: &str = "embed_element";
    pub const EMBED_CHAPTER: &str = "embed_chapter";
    pub const SYNC_EMBEDDINGS: &str = "sync_embeddings";
    pub const GENERATE_SUMMARY: &str = "generate_summary";
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AsyncTask {
    pub id: String,
    pub task_type: String,
    pub project_id: String,
    pub target_type: Option<String>,
    pub target_id: Option<String>,
    pub content_hash: Option<String>,
    pub payload_json: Option<String>,
    
    pub status: String,
    pub progress_current: i64,
    pub progress_total: i64,
    
    pub result_json: Option<String>,
    pub error_message: Option<String>,
    
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTaskRequest {
    pub task_type: String,
    pub project_id: String,
    pub target_type: Option<String>,
    pub target_id: Option<String>,
    pub content_hash: Option<String>,
    pub payload_json: Option<serde_json::Value>,
}
