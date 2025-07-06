use crate::ir;

pub fn generate_enhanced_main_server(ir: &ir::SchemaIR) -> String {
    let mut out = String::new();
    
    out.push_str("use axum::{\n");
    out.push_str("    http::StatusCode,\n");
    out.push_str("    response::Json,\n");
    out.push_str("    routing::get,\n");
    out.push_str("    Router,\n");
    out.push_str("};\n");
    out.push_str("use serde_json::json;\n");
    out.push_str("use std::sync::Arc;\n");
    out.push_str("use tower_http::cors::{Any, CorsLayer};\n");
    out.push_str("use crate::generated::router::create_router;\n");
    out.push_str("use crate::generated::auth::auth_middleware;\n");
    out.push_str("use crate::generated::config::Config;\n\n");
    
    out.push_str("pub struct AppState {\n");
    out.push_str(&format!("    pub pool: {},\n", get_pool_type(&ir.meta.db_backend)));
    out.push_str("    pub jwt_secret: String,\n");
    out.push_str("}\n\n");
    
    out.push_str("pub async fn main() -> Result<(), Box<dyn std::error::Error>> {\n");
    
    // Initialize tracing if enabled
    if matches!(ir.meta.observability_provider.as_deref(), Some("tracing")) {
        out.push_str("    tracing_subscriber::fmt::init();\n\n");
    }
    
    // Load configuration
    out.push_str("    let config = Config::from_env()?;\n\n");
    
    // Database connection
    out.push_str(&format!("    let pool = {}::connect(&config.database.url).await?;\n\n", get_pool_connect(&ir.meta.db_backend)));
    
    // CORS setup
    out.push_str("    let cors = CorsLayer::new()\n");
    out.push_str("        .allow_origin(Any)\n");
    out.push_str("        .allow_methods(Any)\n");
    out.push_str("        .allow_headers(Any);\n\n");
    
    // Router setup
    out.push_str("    let state = Arc::new(AppState {\n");
    out.push_str("        pool,\n");
    out.push_str("        jwt_secret: config.server.jwt_secret.clone(),\n");
    out.push_str("    });\n");
    out.push_str("    \n");
    out.push_str("    let app = Router::new()\n");
    out.push_str("        .route(\"/health\", get(health_check))\n");
    out.push_str("        .nest(\"/api\", create_router()\n");
    out.push_str("            .route_layer(axum::middleware::from_fn_with_state(\n");
    out.push_str("                state.clone(),\n");
    out.push_str("                auth_middleware\n");
    out.push_str("            )))\n");
    out.push_str("        .layer(cors)\n");
    out.push_str("        .with_state(state);\n\n");
    
    // Server startup
    out.push_str("    let addr = format!(\"127.0.0.1:{}\", config.server.port);\n");
    out.push_str("    let listener = tokio::net::TcpListener::bind(&addr).await?;\n");
    out.push_str("    println!(\"Server running on http://{}\", addr);\n");
    out.push_str("    axum::serve(listener, app).await?;\n\n");
    out.push_str("    Ok(())\n");
    out.push_str("}\n\n");
    
    // Health check handler
    out.push_str("async fn health_check() -> Json<serde_json::Value> {\n");
    out.push_str("    Json(json!({ \"status\": \"ok\" }))\n");
    out.push_str("}\n");
    
    out
}

fn get_pool_type(backend: &ir::DatabaseBackend) -> &str {
    match backend {
        ir::DatabaseBackend::Postgres => "sqlx::PgPool",
        ir::DatabaseBackend::Mysql => "sqlx::MySqlPool", 
        ir::DatabaseBackend::Sqlite => "sqlx::SqlitePool",
    }
}

fn get_pool_connect(backend: &ir::DatabaseBackend) -> &str {
    match backend {
        ir::DatabaseBackend::Postgres => "sqlx::PgPool",
        ir::DatabaseBackend::Mysql => "sqlx::MySqlPool",
        ir::DatabaseBackend::Sqlite => "sqlx::SqlitePool", 
    }
}