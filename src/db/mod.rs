use sqlx::{PgPool, Row};
use std::collections::HashMap;

/// Database utilities and common operations
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Execute a health check query
    pub async fn health_check(&self) -> Result<bool, sqlx::Error> {
        let row = sqlx::query("SELECT 1 as health")
            .fetch_one(&self.pool)
            .await?;
        
        let health: i32 = row.get("health");
        Ok(health == 1)
    }

    /// Get database statistics
    pub async fn get_stats(&self) -> Result<DatabaseStats, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT 
                pg_database_size(current_database()) as db_size,
                (SELECT count(*) FROM pg_stat_activity WHERE state = 'active') as active_connections,
                (SELECT count(*) FROM users) as user_count,
                (SELECT count(*) FROM refresh_tokens WHERE expires_at > NOW()) as active_tokens
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(DatabaseStats {
            db_size: row.get::<i64, _>("db_size") as u64,
            active_connections: row.get::<i64, _>("active_connections") as u32,
            user_count: row.get::<i64, _>("user_count") as u64,
            active_tokens: row.get::<i64, _>("active_tokens") as u64,
        })
    }

    /// Cleanup expired tokens
    pub async fn cleanup_expired_tokens(&self) -> Result<CleanupResult, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        // Delete expired refresh tokens
        let refresh_tokens_deleted = sqlx::query(
            "DELETE FROM refresh_tokens WHERE expires_at < NOW()"
        )
        .execute(&mut *tx)
        .await?
        .rows_affected();

        // Delete expired blacklisted tokens
        let blacklisted_tokens_deleted = sqlx::query(
            "DELETE FROM blacklisted_tokens WHERE expires_at < NOW()"
        )
        .execute(&mut *tx)
        .await?
        .rows_affected();

        // Delete old audit logs (older than 1 year)
        let audit_logs_deleted = sqlx::query(
            "DELETE FROM audit_logs WHERE timestamp < NOW() - INTERVAL '1 year'"
        )
        .execute(&mut *tx)
        .await?
        .rows_affected();

        // Delete old cron job logs (keep only last 1000 per job)
        let cron_logs_deleted = sqlx::query(
            r#"
            DELETE FROM cron_job_logs 
            WHERE id NOT IN (
                SELECT id FROM (
                    SELECT id, ROW_NUMBER() OVER (PARTITION BY job_name ORDER BY started_at DESC) as rn
                    FROM cron_job_logs
                ) t WHERE t.rn <= 1000
            )
            "#
        )
        .execute(&mut *tx)
        .await?
        .rows_affected();

        tx.commit().await?;

        Ok(CleanupResult {
            refresh_tokens_deleted,
            blacklisted_tokens_deleted,
            audit_logs_deleted,
            cron_logs_deleted,
        })
    }

    /// Get connection pool reference
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Execute a transaction
    pub async fn with_transaction<F, R, E>(&self, callback: F) -> Result<R, E>
    where
        F: FnOnce(&mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<R, E>,
        E: From<sqlx::Error>,
    {
        let mut tx = self.pool.begin().await?;
        let result = callback(&mut tx)?;
        tx.commit().await?;
        Ok(result)
    }
}

#[derive(Debug)]
pub struct DatabaseStats {
    pub db_size: u64,
    pub active_connections: u32,
    pub user_count: u64,
    pub active_tokens: u64,
}

#[derive(Debug)]
pub struct CleanupResult {
    pub refresh_tokens_deleted: u64,
    pub blacklisted_tokens_deleted: u64,
    pub audit_logs_deleted: u64,
    pub cron_logs_deleted: u64,
}

impl CleanupResult {
    pub fn total_deleted(&self) -> u64 {
        self.refresh_tokens_deleted + 
        self.blacklisted_tokens_deleted + 
        self.audit_logs_deleted + 
        self.cron_logs_deleted
    }
}

/// Query builder helper for common database operations
pub struct QueryBuilder {
    conditions: Vec<String>,
    params: HashMap<String, serde_json::Value>,
    order_by: Vec<String>,
    limit: Option<u32>,
    offset: Option<u32>,
}

impl QueryBuilder {
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
            params: HashMap::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
        }
    }

    pub fn add_condition(mut self, condition: &str, param_name: &str, value: serde_json::Value) -> Self {
        self.conditions.push(condition.to_string());
        self.params.insert(param_name.to_string(), value);
        self
    }

    pub fn order_by(mut self, column: &str, direction: &str) -> Self {
        self.order_by.push(format!("{} {}", column, direction));
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn build_where_clause(&self) -> String {
        if self.conditions.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", self.conditions.join(" AND "))
        }
    }

    pub fn build_order_clause(&self) -> String {
        if self.order_by.is_empty() {
            String::new()
        } else {
            format!(" ORDER BY {}", self.order_by.join(", "))
        }
    }

    pub fn build_limit_clause(&self) -> String {
        let mut clause = String::new();
        if let Some(limit) = self.limit {
            clause.push_str(&format!(" LIMIT {}", limit));
        }
        if let Some(offset) = self.offset {
            clause.push_str(&format!(" OFFSET {}", offset));
        }
        clause
    }

    pub fn params(&self) -> &HashMap<String, serde_json::Value> {
        &self.params
    }
}

impl Default for QueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_builder() {
        let builder = QueryBuilder::new()
            .add_condition("name = $1", "name", serde_json::Value::String("test".to_string()))
            .add_condition("age > $2", "age", serde_json::Value::Number(18.into()))
            .order_by("created_at", "DESC")
            .limit(10)
            .offset(0);

        assert_eq!(builder.build_where_clause(), " WHERE name = $1 AND age > $2");
        assert_eq!(builder.build_order_clause(), " ORDER BY created_at DESC");
        assert_eq!(builder.build_limit_clause(), " LIMIT 10 OFFSET 0");
        assert_eq!(builder.params().len(), 2);
    }

    #[test]
    fn test_cleanup_result() {
        let result = CleanupResult {
            refresh_tokens_deleted: 10,
            blacklisted_tokens_deleted: 5,
            audit_logs_deleted: 100,
            cron_logs_deleted: 20,
        };

        assert_eq!(result.total_deleted(), 135);
    }
}
