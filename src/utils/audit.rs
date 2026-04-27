use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppResult;

/// Audit payload for write operations.
#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub user_id: Option<Uuid>,
    pub action: String,
    pub entity: String,
    pub entity_id: String,
    pub before: Option<Value>,
    pub after: Option<Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// Persist an audit entry.
pub async fn write_audit_log(pool: &PgPool, entry: AuditEntry) -> AppResult<()> {
    sqlx::query(
        r#"INSERT INTO audit_logs (id, user_id, action, resource_type, resource_id, old_values, new_values, ip_address, user_agent, timestamp)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())"#,
    )
    .bind(Uuid::new_v4())
    .bind(entry.user_id)
    .bind(entry.action)
    .bind(entry.entity)
    .bind(entry.entity_id)
    .bind(entry.before)
    .bind(entry.after)
    .bind(entry.ip_address)
    .bind(entry.user_agent)
    .execute(pool)
    .await?;
    Ok(())
}
