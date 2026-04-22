use fred::{
    prelude::*,
    types::{ConnectHandle, RedisConfig},
};
use anyhow::Result;

use crate::config::Settings;

/// Redis service wrapper  
pub struct RedisService {
    client: RedisClient,
}

impl RedisService {
    /// Create new Redis service
    pub async fn new(settings: &Settings) -> Result<Self> {
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());

        let config = RedisConfig::from_url(&redis_url)?;
        let client = RedisClient::new(config, None, None, None);
        
        let _handle: ConnectHandle = client.connect();
        client.wait_for_connect().await?;

        Ok(Self { client })
    }

    /// Get a string value
    pub async fn get<V>(&self, key: &str) -> Result<Option<V>>
    where
        V: FromRedis,
    {
        let result: Option<V> = self.client.get(key).await?;
        Ok(result)
    }

    /// Set a string value with expiration
    pub async fn setex<V>(&self, key: &str, value: V, seconds: u64) -> Result<()>
    where
        V: Into<RedisValue>,
    {
        self.client.setex(key, seconds as i64, value).await?;
        Ok(())
    }

    /// Delete a key
    pub async fn del(&self, key: &str) -> Result<i64> {
        let result: i64 = self.client.del(key).await?;
        Ok(result)
    }

    /// Check if key exists
    pub async fn exists(&self, key: &str) -> Result<bool> {
        let result: i64 = self.client.exists(key).await?;
        Ok(result > 0)
    }

    /// Set expiration on a key
    pub async fn expire(&self, key: &str, seconds: u64) -> Result<bool> {
        let result: i64 = self.client.expire(key, seconds as i64).await?;
        Ok(result == 1)
    }

    /// Increment a key
    pub async fn incr(&self, key: &str) -> Result<i64> {
        let result: i64 = self.client.incr(key).await?;
        Ok(result)
    }

    /// Ping Redis server
    pub async fn ping(&self) -> Result<String> {
        let result: String = self.client.ping(None).await?;
        Ok(result)
    }

    /// Get underlying client reference
    pub fn client(&self) -> &RedisClient {
        &self.client
    }
}