use axum::{
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde_json::json;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use crate::generated::router::create_router;
use crate::generated::auth::auth_middleware;
use crate::generated::config::Config;

pub struct AppState {
    pub pool: sqlx::PgPool,
    pub jwt_secret: String,
}

pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let config = Config::from_env()?;

    let pool = sqlx::PgPool::connect(&config.database.url).await?;

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let state = Arc::new(AppState {
        pool,
        jwt_secret: config.server.jwt_secret.clone(),
    });
    
    let app = Router::new()
        .route("/health", get(health_check))
        .nest("/api", create_router()
            .route_layer(axum::middleware::from_fn_with_state(
                state.clone(),
                auth_middleware
            )))
        .layer(cors)
        .with_state(state);

    let addr = format!("127.0.0.1:{}", config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("Server running on http://{}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}
