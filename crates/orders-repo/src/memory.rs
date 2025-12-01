use async_trait::async_trait;
use dashmap::DashMap;
use orders_types::domain::order::{Order, OrderStatus};
use orders_types::ports::order_repository::{OrderRepository, RepoError};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct InMemoryRepo {
    pub map: Arc<DashMap<Uuid, Order>>,
}

impl InMemoryRepo {
    pub fn new() -> Self {
        Self {
            map: Arc::new(DashMap::new()),
        }
    }
}

impl Default for InMemoryRepo {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl OrderRepository for InMemoryRepo {
    async fn create(&self, order: Order) -> Result<Order, RepoError> {
        self.map.insert(order.id, order.clone());
        Ok(order)
    }

    async fn get(&self, id: Uuid) -> Result<Option<Order>, RepoError> {
        Ok(self.map.get(&id).map(|r| r.clone()))
    }

    async fn list(&self) -> Result<Vec<Order>, RepoError> {
        Ok(self.map.iter().map(|kv| kv.value().clone()).collect())
    }

    async fn update_status(
        &self,
        id: Uuid,
        status: OrderStatus,
    ) -> Result<Option<Order>, RepoError> {
        if let Some(mut v) = self.map.get_mut(&id) {
            v.update_status(status);
            return Ok(Some(v.clone()));
        }
        Ok(None)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepoError> {
        Ok(self.map.remove(&id).is_some())
    }
}
