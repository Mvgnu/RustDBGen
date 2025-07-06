use crate::ir;

pub fn generate_cargo_toml(ir: &ir::SchemaIR) -> String {
    let mut out = String::new();
    
    out.push_str("[package]\n");
    out.push_str("name = \"finance-backend\"\n");
    out.push_str("version = \"0.1.0\"\n");
    out.push_str("edition = \"2021\"\n\n");
    
    out.push_str("[dependencies]\n");
    out.push_str("tokio = { version = \"1.0\", features = [\"full\"] }\n");
    out.push_str("axum = \"0.7\"\n");
    out.push_str("tower-http = { version = \"0.5\", features = [\"cors\"] }\n");
    out.push_str("serde = { version = \"1.0\", features = [\"derive\"] }\n");
    out.push_str("serde_json = \"1.0\"\n");
    out.push_str("uuid = { version = \"1.0\", features = [\"v4\", \"serde\"] }\n");
    out.push_str("chrono = { version = \"0.4\", features = [\"serde\"] }\n");
    out.push_str("thiserror = \"1.0\"\n");
    out.push_str("anyhow = \"1.0\"\n");
    out.push_str("jsonwebtoken = \"9.0\"\n");
    out.push_str("axum-extra = { version = \"0.9\", features = [\"typed-header\"] }\n");
    out.push_str("argon2 = \"0.5\"\n");
    out.push_str("password-hash = { version = \"0.5\", features = [\"rand_core\"] }\n");
    out.push_str("rust_decimal = { version = \"1.0\", features = [\"serde\"] }\n");
    out.push_str("dotenvy = \"0.15\"\n");
    
    // Database dependencies based on backend
    match ir.meta.db_backend {
        ir::DatabaseBackend::Postgres => {
            out.push_str("sqlx = { version = \"0.7\", features = [\"runtime-tokio-rustls\", \"postgres\", \"uuid\", \"chrono\", \"json\", \"rust_decimal\"] }\n");
        }
        ir::DatabaseBackend::Mysql => {
            out.push_str("sqlx = { version = \"0.7\", features = [\"runtime-tokio-rustls\", \"mysql\", \"uuid\", \"chrono\", \"json\", \"rust_decimal\"] }\n");
        }
        ir::DatabaseBackend::Sqlite => {
            out.push_str("sqlx = { version = \"0.7\", features = [\"runtime-tokio-rustls\", \"sqlite\", \"uuid\", \"chrono\", \"json\", \"rust_decimal\"] }\n");
        }
    }
    
    // Tracing if enabled
    if matches!(ir.meta.observability_provider.as_deref(), Some("tracing")) {
        out.push_str("tracing = \"0.1\"\n");
        out.push_str("tracing-subscriber = \"0.3\"\n");
    }
    
    out
} 