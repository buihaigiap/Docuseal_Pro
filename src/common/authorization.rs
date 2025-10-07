use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use crate::models::role::Role;

/// Authorization middleware that checks if the user has one of the required roles
pub async fn require_roles(
    required_roles: Vec<Role>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get role from request extensions (set by auth_middleware)
    let user_role = request.extensions().get::<Role>().cloned();

    match user_role {
        Some(role) => {
            if required_roles.contains(&role) {
                Ok(next.run(request).await)
            } else {
                Err(StatusCode::FORBIDDEN)
            }
        }
        None => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Convenience middleware for admin-only access
pub async fn require_admin(request: Request, next: Next) -> Result<Response, StatusCode> {
    require_roles(vec![Role::Admin], request, next).await
}

/// Convenience middleware for admin and team member access
pub async fn require_admin_or_team_member(request: Request, next: Next) -> Result<Response, StatusCode> {
    require_roles(vec![Role::Admin, Role::TeamMember], request, next).await
}

/// Convenience middleware for team member and recipient access
pub async fn require_team_member_or_recipient(request: Request, next: Next) -> Result<Response, StatusCode> {
    require_roles(vec![Role::TeamMember, Role::Recipient], request, next).await
}