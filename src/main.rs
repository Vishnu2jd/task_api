use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use std::time::{Duration, Instant};
use axum::{
    extract::{State, FromRequestParts},
    http::{StatusCode, request::Parts},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use uuid::Uuid;
//use rand::Rng;

mod models;
use models::*;

// --- Cryptographic Helpers ---
fn hash_code(code: &str) -> String {
    bcrypt::hash(code, 4).unwrap_or_default()
}

fn verify_code(code: &str, hash: &str) -> bool {
    bcrypt::verify(code, hash).unwrap_or(false)
}

// --- JWT Claims & Auth Guard Extractor ---
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: u64,
}

pub struct AuthenticatedUser {
    pub email: String,
    pub role: Role,
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        if !auth_header.starts_with("Bearer ") {
            return Err(StatusCode::UNAUTHORIZED);
        }

        let token = &auth_header[7..];
        let key = DecodingKey::from_secret(b"assignment_super_secret_key");
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;

        let token_data = decode::<Claims>(token, &key, &validation)
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        let role = match token_data.claims.role.as_str() {
            "admin" => Role::Admin,
            "staff" => Role::Staff,
            _ => return Err(StatusCode::UNAUTHORIZED),
        };

        Ok(AuthenticatedUser {
            email: token_data.claims.sub,
            role,
        })
    }
}

// --- Shared State ---
#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::SqlitePool,
    pub view_cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    pub email_logs: Arc<Mutex<Vec<String>>>,
}

pub async fn setup_database(pool: &sqlx::SqlitePool) {
    let queries = [
        "CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            full_name TEXT NOT NULL,
            email TEXT UNIQUE NOT NULL,
            hashed_password TEXT NOT NULL,
            role TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );",
        "CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            status TEXT NOT NULL,
            priority TEXT NOT NULL,
            created_by_id TEXT NOT NULL,
            assigned_to_id TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );",
        "CREATE TABLE IF NOT EXISTS two_factor_challenges (
            challenge_id TEXT PRIMARY KEY,
            user_email TEXT NOT NULL,
            hashed_code TEXT NOT NULL,
            expires_at TEXT NOT NULL,
            is_used INTEGER NOT NULL DEFAULT 0
        );"
    ];

    for q in queries {
        sqlx::query(q).execute(pool).await.unwrap();
    }
}

// --- Route Handlers ---

pub async fn seed_users(State(state): State<Arc<AppState>>) -> StatusCode {
    let mut cache = state.view_cache.write().await;
    cache.clear();
    {
        let mut logs = state.email_logs.lock().unwrap();
        logs.clear();
    }

    let _ = sqlx::query("DELETE FROM users").execute(&state.db).await;
    let _ = sqlx::query("DELETE FROM tasks").execute(&state.db).await;
    let _ = sqlx::query("DELETE FROM two_factor_challenges").execute(&state.db).await;

    let now = chrono::Utc::now().to_rfc3339();
    let admin_hashed = bcrypt::hash("password123", 4).unwrap();
    let jb_hashed = bcrypt::hash("doubleseven007", 4).unwrap();

    let _ = sqlx::query("INSERT INTO users (id, full_name, email, hashed_password, role, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(Uuid::new_v4().to_string())
        .bind("Admin User")
        .bind("admin@example.com")
        .bind(admin_hashed)
        .bind("admin")
        .bind(&now)
        .bind(&now)
        .execute(&state.db)
        .await;

    let _ = sqlx::query("INSERT INTO users (id, full_name, email, hashed_password, role, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(Uuid::new_v4().to_string())
        .bind("James Bond")
        .bind("jamesbond@example.com")
        .bind(jb_hashed)
        .bind("staff")
        .bind(&now)
        .bind(&now)
        .execute(&state.db)
        .await;

    StatusCode::OK
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let user: DbUser = sqlx::query_as::<_, DbUser>("SELECT * FROM users WHERE email = ?")
        .bind(&payload.email)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    if !verify_code(&payload.password, &user.hashed_password) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let challenge_id = format!("chal_{}", Uuid::new_v4().simple());
    let nanosecs = chrono::Utc::now().timestamp_subsec_nanos();
    let verification_code = format!("{:06}", nanosecs % 1000000);

    {
        let mut logs = state.email_logs.lock().unwrap();
        logs.push(verification_code.clone());
    }

    let hashed_code = hash_code(&verification_code);
    let expires_at = (chrono::Utc::now() + chrono::Duration::minutes(5)).to_rfc3339();

    let _ = sqlx::query("INSERT INTO two_factor_challenges (challenge_id, user_email, hashed_code, expires_at, is_used) VALUES (?, ?, ?, ?, 0)")
        .bind(&challenge_id)
        .bind(&payload.email)
        .bind(hashed_code)
        .bind(expires_at)
        .execute(&state.db)
        .await;

    Ok(Json(LoginResponse { login_challenge_id: challenge_id }))
}

pub async fn get_latest_email_log(
    State(state): State<Arc<AppState>>,
) -> Result<Json<LogResponse>, StatusCode> {
    let logs = state.email_logs.lock().unwrap();
    match logs.last() {
        Some(code) => Ok(Json(LogResponse { latest_code: code.clone() })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn verify_2fa(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Verify2faRequest>,
) -> Result<Json<TokenResponse>, StatusCode> {
    let challenge: TwoFactorChallenge = sqlx::query_as::<_, TwoFactorChallenge>("SELECT * FROM two_factor_challenges WHERE challenge_id = ?")
        .bind(&payload.login_challenge_id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    if challenge.is_used == 1 {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let expires = chrono::DateTime::parse_from_rfc3339(&challenge.expires_at)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if chrono::Utc::now() > expires {
        return Err(StatusCode::UNAUTHORIZED);
    }

    if !verify_code(&payload.code, &challenge.hashed_code) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let _ = sqlx::query("UPDATE two_factor_challenges SET is_used = 1 WHERE challenge_id = ?")
        .bind(&payload.login_challenge_id)
        .execute(&state.db)
        .await;

    let user: DbUser = sqlx::query_as::<_, DbUser>("SELECT * FROM users WHERE email = ?")
        .bind(&challenge.user_email)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let expiration = chrono::Utc::now().timestamp() + (24 * 3600);
    let claims = Claims {
        sub: user.email,
        role: user.role,
        exp: expiration as u64,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"assignment_super_secret_key"),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(TokenResponse { token }))
}

pub async fn create_task(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<TaskResponseItem>), StatusCode> {
    if user.role != Role::Admin {
        return Err(StatusCode::FORBIDDEN);
    }

    let admin: DbUser = sqlx::query_as::<_, DbUser>("SELECT * FROM users WHERE email = ?")
        .bind(&user.email)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let now = chrono::Utc::now().to_rfc3339();

    let result = sqlx::query("INSERT INTO tasks (title, description, status, priority, created_by_id, assigned_to_id, created_at, updated_at) VALUES (?, ?, 'todo', ?, ?, NULL, ?, ?)")
        .bind(&payload.title)
        .bind(&payload.description)
        .bind(&payload.priority)
        .bind(&admin.id)
        .bind(&now)
        .bind(&now)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let task_id = result.last_insert_rowid();

    Ok((StatusCode::CREATED, Json(TaskResponseItem {
        id: task_id.to_string(),
        title: payload.title,
        status: "todo".to_string(),
        priority: payload.priority,
        assigned_to: "".to_string(),
    })))
}

pub async fn assign_tasks(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Json(payload): Json<AssignTaskRequest>,
) -> Result<StatusCode, StatusCode> {
    if user.role != Role::Admin {
        return Err(StatusCode::FORBIDDEN);
    }

    let target: DbUser = sqlx::query_as::<_, DbUser>("SELECT * FROM users WHERE email = ?")
        .bind(&payload.assign_to_email)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let now = chrono::Utc::now().to_rfc3339();

    for id in &payload.task_ids {
        let rows_affected = sqlx::query("UPDATE tasks SET assigned_to_id = ?, updated_at = ? WHERE id = ?")
            .bind(&target.id)
            .bind(&now)
            .bind(id)
            .execute(&state.db)
            .await
            .map(|r| r.rows_affected())
            .unwrap_or(0);

        if rows_affected == 0 {
            return Err(StatusCode::NOT_FOUND);
        }
    }

    let mut cache = state.view_cache.write().await;
    cache.remove(&payload.assign_to_email);

    Ok(StatusCode::OK)
}

pub async fn view_my_tasks(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
) -> Result<Json<TaskViewResponse>, StatusCode> {
    let role_str = match user.role {
        Role::Admin => "admin",
        Role::Staff => "staff",
    };

    {
        let cache = state.view_cache.read().await;
        if let Some(entry) = cache.get(&user.email) {
            let summary = Summary {
                total_assigned_tasks: entry.data.len(),
            };
            return Ok(Json(TaskViewResponse {
                user: UserDetailsResponse { email: user.email.clone(), role: role_str.to_string() },
                tasks: entry.data.clone(),
                summary,
                cache: CacheMetadata { hit: true },
            }));
        }
    }

    let db_user: DbUser = sqlx::query_as::<_, DbUser>("SELECT * FROM users WHERE email = ?")
        .bind(&user.email)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let db_tasks: Vec<DbTask> = sqlx::query_as::<_, DbTask>("SELECT * FROM tasks WHERE assigned_to_id = ?")
        .bind(&db_user.id)
        .fetch_all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let tasks: Vec<TaskResponseItem> = db_tasks
        .into_iter()
        .map(|t| TaskResponseItem {
            id: t.id.to_string(),
            title: t.title,
            status: t.status,
            priority: t.priority,
            assigned_to: user.email.clone(),
        })
        .collect();

    {
        let mut cache = state.view_cache.write().await;
        cache.insert(user.email.clone(), CacheEntry { data: tasks.clone(), cached_at: Instant::now() });
    }

    let summary = Summary {
        total_assigned_tasks: tasks.len(),
    };

    Ok(Json(TaskViewResponse {
        user: UserDetailsResponse { email: user.email, role: role_str.to_string() },
        tasks,
        summary,
        cache: CacheMetadata { hit: false },
    }))
}

// --- Main Setup ---
#[tokio::main]
async fn main() {
    let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
    setup_database(&db).await;

    let state = Arc::new(AppState {
        db,
        view_cache: Arc::new(RwLock::new(HashMap::new())),
        email_logs: Arc::new(Mutex::new(Vec::new())),
    });

    let app = Router::new()
        .route("/seed/users", post(seed_users))
        .route("/auth/login", post(login))
        .route("/dev/email-logs/latest", get(get_latest_email_log))
        .route("/auth/verify-2fa", post(verify_2fa))
        .route("/tasks", post(create_task))
        .route("/tasks/assign", post(assign_tasks))
        .route("/tasks/view-my-tasks", get(view_my_tasks))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Server running on http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}
