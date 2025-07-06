use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub jwt_secret: String,
}

impl Config {
    /// Loads configuration from environment variables.
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| anyhow::anyhow!("DATABASE_URL must be set"))?;

        let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string()).parse::<u16>()?;

        let jwt_secret = std::env::var("JWT_SECRET")
            .map_err(|_| anyhow::anyhow!("JWT_SECRET must be set"))?;

        Ok(Self {
            database: DatabaseConfig { url: database_url },
            server: ServerConfig { port, jwt_secret },
        })
    }
}
