use redis::aio::ConnectionManager;
use redis::AsyncCommands;

use crate::config::RedisConfig;

/// Redis cache wrapper for vehicle reports.
#[derive(Clone)]
pub struct RedisCache {
    conn: ConnectionManager,
    ttl_secs: u64,
}

impl RedisCache {
    /// Connect to Redis and return a cache instance.
    pub async fn new(cfg: &RedisConfig) -> anyhow::Result<Self> {
        let client = redis::Client::open(cfg.url.as_str())?;
        let conn = ConnectionManager::new(client).await?;
        tracing::info!("connected to Redis");
        Ok(Self {
            conn,
            ttl_secs: cfg.cache_ttl_secs,
        })
    }

    /// Get a cached vehicle report JSON by plate.
    pub async fn get(&self, plate: &str) -> anyhow::Result<Option<String>> {
        let key = format!("vehicle:{}", plate);
        let mut conn = self.conn.clone();
        let val: Option<String> = conn.get(&key).await?;
        Ok(val)
    }

    /// Set a vehicle report JSON in cache with TTL.
    pub async fn set(&self, plate: &str, json_data: &str) -> anyhow::Result<()> {
        let key = format!("vehicle:{}", plate);
        let mut conn = self.conn.clone();
        let _: () = conn.set_ex(&key, json_data, self.ttl_secs).await?;
        Ok(())
    }

    /// Delete a cached vehicle report.
    pub async fn delete(&self, plate: &str) -> anyhow::Result<()> {
        let key = format!("vehicle:{}", plate);
        let mut conn = self.conn.clone();
        let _: () = conn.del(&key).await?;
        Ok(())
    }
}
