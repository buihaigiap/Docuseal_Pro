use axum::{
    extract::{State, Path, Extension},
    http::StatusCode,
    response::{Json, IntoResponse},
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::ToSchema;
use bcrypt::{hash, DEFAULT_COST};
use uuid::Uuid;

use crate::routes::web::AppState;
use crate::database::queries::AccountQueries;
use crate::database::models::{CreateUser, DbUser};
use crate::models::user::User;
use crate::models::role::Role;
use crate::services::email::EmailService;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTeamMemberRequest {
    pub name: String,
    pub email: String,
    pub password: Option<String>,
    // All team members have admin role by default
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTeamMemberRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    // Role cannot be changed - all members are admin
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TeamMemberResponse {
    pub user: User,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TeamMembersResponse {
    pub users: Vec<User>,
}

/// Get all team members for the current user's account
#[utoipa::path(
    get,
    path = "/api/team/members",
    tag = "Team Management",
    responses(
        (status = 200, description = "Team members retrieved successfully", body = TeamMembersResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Account not found")
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn get_team_members(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
) -> Result<Json<TeamMembersResponse>, StatusCode> {
    let state_lock = state.lock().await;
    let pool = &state_lock.db_pool;
    
    // Get user to get account_id
    let db_user = crate::database::queries::UserQueries::get_user_by_id(pool, user_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let account_id = db_user.account_id
        .ok_or(StatusCode::BAD_REQUEST)?;

    let db_users = AccountQueries::get_account_users(pool, account_id, false).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let users: Vec<User> = db_users.into_iter().map(|u| u.into()).collect();

    Ok(Json(TeamMembersResponse { users }))
}

/// Get archived team members
#[utoipa::path(
    get,
    path = "/api/team/members/archived",
    tag = "Team Management",
    responses(
        (status = 200, description = "Archived team members retrieved successfully", body = TeamMembersResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn get_archived_team_members(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
) -> Result<Json<TeamMembersResponse>, StatusCode> {
    let state_lock = state.lock().await;
    let pool = &state_lock.db_pool;
    
    let db_user = crate::database::queries::UserQueries::get_user_by_id(pool, user_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let account_id = db_user.account_id
        .ok_or(StatusCode::BAD_REQUEST)?;

    let db_users = AccountQueries::get_account_users(pool, account_id, true).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let users: Vec<User> = db_users.into_iter()
        .filter(|u| u.archived_at.is_some())
        .map(|u| u.into())
        .collect();

    Ok(Json(TeamMembersResponse { users }))
}

/// Create a new team member
#[utoipa::path(
    post,
    path = "/api/team/members",
    tag = "Team Management",
    request_body = CreateTeamMemberRequest,
    responses(
        (status = 201, description = "Team member created successfully", body = TeamMemberResponse),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized"),
        (status = 409, description = "Email already exists")
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn create_team_member(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Json(payload): Json<CreateTeamMemberRequest>,
) -> Result<(StatusCode, Json<TeamMemberResponse>), StatusCode> {
    let state_lock = state.lock().await;
    let pool = &state_lock.db_pool;
    
    let db_user = crate::database::queries::UserQueries::get_user_by_id(pool, user_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let account_id = db_user.account_id
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Generate password reset token for email invitation
    let activation_token = Uuid::new_v4().to_string();

    // Generate password if not provided (user will set via email link)
    let password = payload.password.clone().unwrap_or_else(|| {
        Uuid::new_v4().to_string()
    });

    let password_hash = hash(password, DEFAULT_COST)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // All team members are admin
    let role = Role::Admin;

    let create_user = CreateUser {
        name: payload.name.clone(),
        email: payload.email.clone(),
        password_hash,
        role,
        is_active: false, // User needs to set password first
        activation_token: Some(activation_token.clone()),
        account_id: Some(account_id),
    };

    let new_user = crate::database::queries::UserQueries::create_user(pool, create_user).await
        .map_err(|e| {
            if e.to_string().contains("duplicate key") || e.to_string().contains("unique constraint") {
                StatusCode::CONFLICT
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    let user: User = new_user.into();

    // Send invitation email asynchronously (deliver_later equivalent)
    let email_service = EmailService::new();
    if let Ok(service) = email_service {
        let base_url = std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8081".to_string());
        let invitation_link = format!("{}/set-password?token={}", base_url, activation_token);
        
        let to_email = payload.email.clone();
        let to_name = payload.name.clone();
        let invited_by_name = db_user.name.clone();
        
        // Get account name (fallback to "Letmesign")
        let account_name = "Letmesign".to_string();
        
        // Spawn async task to send email without blocking
        tokio::spawn(async move {
            if let Err(e) = service.send_team_invitation_email(
                &to_email,
                &to_name,
                &invited_by_name,
                &account_name,
                &invitation_link,
            ).await {
                eprintln!("Failed to send team invitation email: {}", e);
            }
        });
    }

    Ok((StatusCode::CREATED, Json(TeamMemberResponse { user })))
}

/// Update a team member
#[utoipa::path(
    put,
    path = "/api/team/members/{id}",
    tag = "Team Management",
    params(
        ("id" = i64, Path, description = "Team member ID")
    ),
    request_body = UpdateTeamMemberRequest,
    responses(
        (status = 200, description = "Team member updated successfully", body = TeamMemberResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Team member not found")
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn update_team_member(
    State(state): State<AppState>,
    Path(member_id): Path<i64>,
    Extension(user_id): Extension<i64>,
    Json(data): Json<UpdateTeamMemberRequest>,
) -> Result<Json<TeamMemberResponse>, StatusCode> {
    // Prevent users from updating themselves
    if user_id == member_id {
        return Err(StatusCode::FORBIDDEN);
    }

    let state_lock = state.lock().await;
    let pool = &state_lock.db_pool;

    let db_user = crate::database::queries::UserQueries::get_user_by_id(pool, user_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let account_id = db_user.account_id
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Get the member to verify they're in the same account
    let member = crate::database::queries::UserQueries::get_user_by_id(pool, member_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if member.account_id != Some(account_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    // Build update query dynamically based on provided fields
    let mut query_parts = Vec::new();
    let mut param_count = 1;
    
    if data.name.is_some() {
        query_parts.push(format!("name = ${}", param_count));
        param_count += 1;
    }
    if data.email.is_some() {
        query_parts.push(format!("email = ${}", param_count));
        param_count += 1;
    }
    // Role update removed - all members are admin

    query_parts.push(format!("updated_at = ${}", param_count));

    if query_parts.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let query_str = format!(
        "UPDATE users SET {} WHERE id = ${} RETURNING id, name, email, password_hash, role, is_active, activation_token, account_id, archived_at, subscription_status, subscription_expires_at, free_usage_count, signature, initials, two_factor_secret, two_factor_enabled, created_at, updated_at",
        query_parts.join(", "),
        param_count + 1
    );

    let mut query = sqlx::query_as::<_, DbUser>(&query_str);

    if let Some(name) = &data.name {
        query = query.bind(name);
    }
    if let Some(email) = &data.email {
        query = query.bind(email);
    }
    // Role binding removed - all members are admin

    query = query.bind(chrono::Utc::now());
    query = query.bind(member_id);

    let updated_user = query.fetch_one(pool).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user: User = updated_user.into();

    Ok(Json(TeamMemberResponse { user }))
}

/// Archive a team member
#[utoipa::path(
    post,
    path = "/api/team/members/{id}/archive",
    tag = "Team Management",
    params(
        ("id" = i64, Path, description = "Team member ID")
    ),
    responses(
        (status = 200, description = "Team member archived successfully", body = TeamMemberResponse),
        (status = 400, description = "Cannot archive last user"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Team member not found")
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn archive_team_member(
    State(state): State<AppState>,
    Path(member_id): Path<i64>,
    Extension(user_id): Extension<i64>,
) -> Result<Json<TeamMemberResponse>, StatusCode> {
    // Prevent users from archiving themselves
    if user_id == member_id {
        return Err(StatusCode::FORBIDDEN);
    }

    let state_lock = state.lock().await;
    let pool = &state_lock.db_pool;

    let db_user = crate::database::queries::UserQueries::get_user_by_id(pool, user_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let account_id = db_user.account_id
        .ok_or(StatusCode::BAD_REQUEST)?;

    let archived_user = AccountQueries::archive_user(pool, member_id, account_id).await
        .map_err(|e| {
            if e.to_string().contains("last user") {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    let user: User = archived_user.into();

    Ok(Json(TeamMemberResponse { user }))
}

/// Unarchive a team member
#[utoipa::path(
    post,
    path = "/api/team/members/{id}/unarchive",
    tag = "Team Management",
    params(
        ("id" = i64, Path, description = "Team member ID")
    ),
    responses(
        (status = 200, description = "Team member unarchived successfully", body = TeamMemberResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Team member not found")
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn unarchive_team_member(
    State(state): State<AppState>,
    Path(member_id): Path<i64>,
    Extension(user_id): Extension<i64>,
) -> Result<Json<TeamMemberResponse>, StatusCode> {
    // Prevent users from unarchiving themselves (shouldn't happen but defensive)
    if user_id == member_id {
        return Err(StatusCode::FORBIDDEN);
    }

    let state_lock = state.lock().await;
    let pool = &state_lock.db_pool;

    let db_user = crate::database::queries::UserQueries::get_user_by_id(pool, user_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let account_id = db_user.account_id
        .ok_or(StatusCode::BAD_REQUEST)?;

    let unarchived_user = AccountQueries::unarchive_user(pool, member_id, account_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user: User = unarchived_user.into();

    Ok(Json(TeamMemberResponse { user }))
}

/// Delete a team member permanently
#[utoipa::path(
    delete,
    path = "/api/team/members/{id}",
    tag = "Team Management",
    params(
        ("id" = i64, Path, description = "Team member ID")
    ),
    responses(
        (status = 204, description = "Team member deleted successfully"),
        (status = 400, description = "Cannot delete last user"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Team member not found")
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn delete_team_member(
    State(state): State<AppState>,
    Path(member_id): Path<i64>,
    Extension(user_id): Extension<i64>,
) -> Result<StatusCode, StatusCode> {
    // Prevent users from deleting themselves
    if user_id == member_id {
        return Err(StatusCode::FORBIDDEN);
    }

    let state_lock = state.lock().await;
    let pool = &state_lock.db_pool;

    let db_user = crate::database::queries::UserQueries::get_user_by_id(pool, user_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let account_id = db_user.account_id
        .ok_or(StatusCode::BAD_REQUEST)?;

    AccountQueries::delete_user(pool, member_id, account_id).await
        .map_err(|e| {
            if e.to_string().contains("last user") {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Send invitation to a new team member
#[utoipa::path(
    post,
    path = "/api/team/invitations",
    tag = "Team Management",
    request_body = CreateTeamMemberRequest,
    responses(
        (status = 201, description = "Invitation sent successfully"),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized")
    ),
    security(
        ("bearer" = [])
    )
)]
pub async fn send_team_invitation(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Json(payload): Json<CreateTeamMemberRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    let state_lock = state.lock().await;
    let pool = &state_lock.db_pool;
    
    let db_user = crate::database::queries::UserQueries::get_user_by_id(pool, user_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let account_id = db_user.account_id
        .ok_or(StatusCode::BAD_REQUEST)?;

    let token = Uuid::new_v4().to_string();
    let role = Role::Admin; // All team members are admin

    // Create invitation in database
    let expires_at = chrono::Utc::now() + chrono::Duration::days(7);
    
    sqlx::query(
        r#"
        INSERT INTO user_invitations (email, name, role, invited_by_user_id, account_id, token, is_used, created_at, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#
    )
    .bind(&payload.email)
    .bind(&payload.name)
    .bind(&role)
    .bind(user_id)
    .bind(account_id)
    .bind(&token)
    .bind(false)
    .bind(chrono::Utc::now())
    .bind(expires_at)
    .execute(pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // TODO: Send invitation email with token
    // For now, just return success

    Ok((StatusCode::CREATED, Json(serde_json::json!({
        "message": "Invitation sent successfully",
        "token": token
    }))))
}

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/team/members", get(get_team_members))
        .route("/team/members", post(create_team_member))
        .route("/team/members/archived", get(get_archived_team_members))
        .route("/team/members/:id", put(update_team_member))
        .route("/team/members/:id", delete(delete_team_member))
        .route("/team/members/:id/archive", post(archive_team_member))
        .route("/team/members/:id/unarchive", post(unarchive_team_member))
        .route("/team/invitations", post(send_team_invitation))
}
