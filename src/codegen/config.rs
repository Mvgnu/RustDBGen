use crate::ir;

pub fn generate_config_struct(_ir: &ir::SchemaIR) -> String {
    let mut out = String::new();
    
    out.push_str("use serde::Deserialize;\n\n");
    
    out.push_str("#[derive(Debug, Deserialize, Clone)]\n");
    out.push_str("pub struct Config {\n");
    out.push_str("    pub database: DatabaseConfig,\n");
    out.push_str("    pub server: ServerConfig,\n");
    out.push_str("}\n\n");
    
    out.push_str("#[derive(Debug, Deserialize, Clone)]\n");
    out.push_str("pub struct DatabaseConfig {\n");
    out.push_str("    pub url: String,\n");
    out.push_str("}\n\n");
    
    out.push_str("#[derive(Debug, Deserialize, Clone)]\n");
    out.push_str("pub struct ServerConfig {\n");
    out.push_str("    pub port: u16,\n");
    out.push_str("    pub jwt_secret: String,\n");
    out.push_str("}\n\n");
    
    out.push_str("impl Config {\n");
    out.push_str("    /// Loads configuration from environment variables.\n");
    out.push_str("    pub fn from_env() -> anyhow::Result<Self> {\n");
    out.push_str("        dotenvy::dotenv().ok();\n\n");
    out.push_str("        let database_url = std::env::var(\"DATABASE_URL\")\n");
    out.push_str("            .map_err(|_| anyhow::anyhow!(\"DATABASE_URL must be set\"))?;\n\n");
    out.push_str("        let port = std::env::var(\"PORT\").unwrap_or_else(|_| \"3000\".to_string()).parse::<u16>()?;\n\n");
    out.push_str("        let jwt_secret = std::env::var(\"JWT_SECRET\")\n");
    out.push_str("            .map_err(|_| anyhow::anyhow!(\"JWT_SECRET must be set\"))?;\n\n");
    out.push_str("        Ok(Self {\n");
    out.push_str("            database: DatabaseConfig { url: database_url },\n");
    out.push_str("            server: ServerConfig { port, jwt_secret },\n");
    out.push_str("        })\n");
    out.push_str("    }\n");
    out.push_str("}\n");
    
    out
}