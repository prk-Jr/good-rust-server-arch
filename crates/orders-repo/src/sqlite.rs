use async_trait::async_trait;
use chrono::{DateTime, Utc};
use orders_types::domain::order::{Order, OrderItem, OrderStatus};
use orders_types::ports::order_repository::{OrderRepository, RepoError};
use serde_json;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{FromRow, SqlitePool};
use std::str::FromStr;
use uuid::Uuid;

pub struct SqliteRepo {
    pool: SqlitePool,
}

#[derive(FromRow)]
struct DbOrder {
    id: String,
    customer_name: String,
    email: String,
    total_cents: i64,
    status: String,
    created_at: String,
    updated_at: String,
    items_json: String,
}

impl DbOrder {
    fn into_order(self) -> Result<Order, RepoError> {
        let status = match self.status.as_str() {
            "Pending" => OrderStatus::Pending,
            "Confirmed" => OrderStatus::Confirmed,
            "Shipped" => OrderStatus::Shipped,
            "Cancelled" => OrderStatus::Cancelled,
            "Completed" => OrderStatus::Completed,
            _ => OrderStatus::Pending,
        };
        let items: Vec<OrderItem> = serde_json::from_str(&self.items_json)
            .map_err(|e| RepoError::DbError(e.to_string()))?;
        let created_at = DateTime::parse_from_rfc3339(&self.created_at)
            .map_err(|e| RepoError::DbError(e.to_string()))?
            .with_timezone(&Utc);
        let updated_at = DateTime::parse_from_rfc3339(&self.updated_at)
            .map_err(|e| RepoError::DbError(e.to_string()))?
            .with_timezone(&Utc);
        let id = Uuid::parse_str(&self.id).map_err(|e| RepoError::DbError(e.to_string()))?;
        Ok(Order {
            id,
            customer_name: self.customer_name,
            email: self.email,
            items,
            total_cents: self.total_cents,
            status,
            created_at,
            updated_at,
        })
    }
}

impl SqliteRepo {
    pub async fn new(database_url: &str) -> anyhow::Result<Self> {
        // Ensure on-disk SQLite target directory exists (no-op for in-memory).
        if let Some(path) = database_url.strip_prefix("sqlite://") {
            if path != ":memory:" {
                let p = std::path::Path::new(path);
                if let Some(parent) = p.parent() {
                    if !parent.as_os_str().is_empty() {
                        tokio::fs::create_dir_all(parent).await?;
                    }
                }
            }
        }

        let options = SqliteConnectOptions::from_str(database_url)?.create_if_missing(true);

        let pool = SqlitePool::connect_with(options).await?;

        // Run migration from migration file.
        let ddl = include_str!("../migrations/0001_create_orders.sql");
        sqlx::query(ddl).execute(&pool).await?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl OrderRepository for SqliteRepo {
    async fn create(&self, order: Order) -> Result<Order, RepoError> {
        let items_json =
            serde_json::to_string(&order.items).map_err(|e| RepoError::DbError(e.to_string()))?;
        sqlx::query(
            "INSERT INTO orders (id, customer_name, email, total_cents, status, created_at, updated_at, items_json)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(order.id.to_string())
        .bind(&order.customer_name)
        .bind(&order.email)
        .bind(order.total_cents)
        .bind(format!("{:?}", order.status))
        .bind(order.created_at.to_rfc3339())
        .bind(order.updated_at.to_rfc3339())
        .bind(items_json)
        .execute(&self.pool)
        .await
        .map_err(|e| RepoError::DbError(e.to_string()))?;
        Ok(order)
    }

    async fn get(&self, id: Uuid) -> Result<Option<Order>, RepoError> {
        let row: Option<DbOrder> = sqlx::query_as(
            "SELECT id, customer_name, email, total_cents, status, created_at, updated_at, items_json FROM orders WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepoError::DbError(e.to_string()))?;
        Ok(row.map(|r| r.into_order()).transpose()?)
    }

    async fn list(&self) -> Result<Vec<Order>, RepoError> {
        let rows: Vec<DbOrder> = sqlx::query_as(
            "SELECT id, customer_name, email, total_cents, status, created_at, updated_at, items_json FROM orders",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepoError::DbError(e.to_string()))?;

        rows.into_iter()
            .map(|r| r.into_order())
            .collect::<Result<Vec<_>, _>>()
    }

    async fn update_status(
        &self,
        id: Uuid,
        status: OrderStatus,
    ) -> Result<Option<Order>, RepoError> {
        let status_s = format!("{:?}", status);
        let updated = sqlx::query("UPDATE orders SET status = ?, updated_at = ? WHERE id = ?")
            .bind(status_s)
            .bind(Utc::now().to_rfc3339())
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| RepoError::DbError(e.to_string()))?;
        if updated.rows_affected() == 0 {
            return Ok(None);
        }
        self.get(id).await
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepoError> {
        let res = sqlx::query("DELETE FROM orders WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| RepoError::DbError(e.to_string()))?;
        Ok(res.rows_affected() > 0)
    }
}
