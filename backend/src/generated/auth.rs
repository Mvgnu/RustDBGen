use axum::{
    extract::{MatchedPath, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::generated::main::AppState;
use crate::generated::routes::routes;
use crate::generated::permissions::{route_has_permission, Route};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: uuid::Uuid,
    pub role: String,
    pub exp: usize,
}

pub async fn auth_middleware(
    matched_path: MatchedPath,
    State(state): State<Arc<AppState>>,
    auth_header: Option<TypedHeader<Authorization<Bearer>>>,
    mut request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = matched_path.as_str();

    // Map the matched path to route definition
    let route: Option<&Route> = match path {
        "/api/accounts" | "/api/accounts/:id" => Some(&routes::ACCOUNT),
        "/api/budgets" | "/api/budgets/:id" => Some(&routes::BUDGET),
        "/api/categories" | "/api/categories/:id" => Some(&routes::CATEGORY),
        "/api/goals" | "/api/goals/:id" => Some(&routes::GOAL),
        "/api/recurring-transactions" | "/api/recurring-transactions/:id" => Some(&routes::RECURRINGTRANSACTION),
        "/api/transactions" | "/api/transactions/:id" => Some(&routes::TRANSACTION),
        "/api/users" | "/api/users/:id" => Some(&routes::USER),
        _ => None,
    };

    let auth_is_required = route.map_or(true, |r| r.auth_required);

    if !auth_is_required {
        // This is a public route, let it through without any checks
        return Ok(next.run(request).await);
    }

    // Extract and validate JWT token
    let token = auth_header
        .ok_or(StatusCode::UNAUTHORIZED)?
        .token()
        .to_string();

    let claims = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(state.jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?
    .claims;

    // Check role-based permissions
    if let Some(r) = route {
        if !route_has_permission(r, request.method().as_str(), &claims.role) {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    request.extensions_mut().insert(claims);
    Ok(next.run(request).await)
}

use axum::extract::FromRequestParts;
use axum::http::request::Parts;

#[axum::async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts.extensions
            .get::<Claims>()
            .cloned()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)
    }
}
