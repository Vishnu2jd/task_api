use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    Staff,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DbUser {
    pub id: String,
    pub full_name: String,
    pub email: String,
    pub hashed_password: String,
    pub role: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DbTask {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub created_by_id: String,
    pub assigned_to_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TwoFactorChallenge {
    pub challenge_id: String,
    pub user_email: String,
    pub hashed_code: String,
    pub expires_at: String,
    pub is_used: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskResponseItem {
    pub id: String,
    pub title: String,
    pub status: String,
    pub priority: String,
    pub assigned_to: String,
}

#[derive(Clone)]
pub struct CacheEntry {
    pub data: Vec<TaskResponseItem>,
    pub cached_at: Instant,
}

// --- Request / Response Payloads ---

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub login_challenge_id: String,
}

#[derive(Deserialize)]
pub struct Verify2faRequest {
    pub login_challenge_id: String,
    pub code: String,
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub token: String,
}

#[derive(Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: String,
    pub priority: String,
}

#[derive(Deserialize)]
pub struct AssignTaskRequest {
    pub task_ids: Vec<i64>,
    pub assign_to_email: String,
}

#[derive(Serialize, Clone)]
pub struct Summary {
    pub total_assigned_tasks: usize,
}

#[derive(Serialize, Clone)]
pub struct CacheMetadata {
    pub hit: bool,
}

#[derive(Serialize, Clone)]
pub struct UserDetailsResponse {
    pub email: String,
    pub role: String,
}

#[derive(Serialize, Clone)]
pub struct TaskViewResponse {
    pub user: UserDetailsResponse,
    pub tasks: Vec<TaskResponseItem>,
    pub summary: Summary,
    pub cache: CacheMetadata,
}

#[derive(Serialize)]
pub struct LogResponse {
    pub latest_code: String,
}
