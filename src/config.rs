use std::env;

/// Application configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub ai: AiConfig,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub port: u16,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone)]
pub struct RedisConfig {
    pub url: String,
    pub cache_ttl_secs: u64,
}

#[derive(Debug, Clone)]
pub struct AiConfig {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
    pub max_tokens: u32,
}

impl Config {
    /// Load configuration from environment variables (with .env fallback).
    pub fn load() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let db_host = env::var("DB_HOST").unwrap_or_else(|_| "localhost".into());
        let db_port = env::var("DB_PORT").unwrap_or_else(|_| "5432".into());
        let db_user = env::var("DB_USER").unwrap_or_else(|_| "cardeal".into());
        let db_pass = env::var("DB_PASSWORD").expect("DB_PASSWORD must be set");
        let db_name = env::var("DB_NAME").unwrap_or_else(|_| "cardeal".into());
        let db_ssl = env::var("DB_SSLMODE").unwrap_or_else(|_| "disable".into());
        let db_url = format!(
            "postgres://{}:{}@{}:{}/{}?sslmode={}",
            db_user, db_pass, db_host, db_port, db_name, db_ssl
        );

        let redis_addr = env::var("REDIS_ADDR").unwrap_or_else(|_| "localhost:6379".into());
        let redis_pass = env::var("REDIS_PASSWORD").unwrap_or_default();
        let redis_url = if redis_pass.is_empty() {
            format!("redis://{}", redis_addr)
        } else {
            format!("redis://:{}@{}", redis_pass, redis_addr)
        };

        let cache_days: u64 = env::var("CACHE_TTL_DAYS")
            .unwrap_or_else(|_| "7".into())
            .parse()
            .unwrap_or(7);

        Ok(Config {
            server: ServerConfig {
                port: env::var("SERVER_PORT")
                    .unwrap_or_else(|_| "8080".into())
                    .parse()
                    .unwrap_or(8080),
            },
            database: DatabaseConfig {
                url: db_url,
                max_connections: env::var("DB_MAX_CONNS")
                    .unwrap_or_else(|_| "25".into())
                    .parse()
                    .unwrap_or(25),
            },
            redis: RedisConfig {
                url: redis_url,
                cache_ttl_secs: cache_days * 86400,
            },
            ai: AiConfig {
                api_key: env::var("AI_API_KEY").unwrap_or_default(),
                model: env::var("AI_MODEL").unwrap_or_else(|_| "gpt-4o".into()),
                base_url: env::var("AI_BASE_URL")
                    .unwrap_or_else(|_| "https://api.openai.com/v1".into()),
                max_tokens: env::var("AI_MAX_TOKENS")
                    .unwrap_or_else(|_| "4096".into())
                    .parse()
                    .unwrap_or(4096),
            },
        })
    }
}
