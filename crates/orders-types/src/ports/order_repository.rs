use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::order::{Order, OrderStatus};

#[derive(thiserror::Error, Debug)]
pub enum RepoError {
    #[error("db error: {0}")]
    DbError(String),
}

#[async_trait]
pub trait OrderRepository: Send + Sync + 'static {
    async fn create(&self, order: Order) -> Result<Order, RepoError>;
    async fn get(&self, id: Uuid) -> Result<Option<Order>, RepoError>;
    async fn list(&self) -> Result<Vec<Order>, RepoError>;
    async fn update_status(
        &self,
        id: Uuid,
        status: OrderStatus,
    ) -> Result<Option<Order>, RepoError>;
    async fn delete(&self, id: Uuid) -> Result<bool, RepoError>;
}
