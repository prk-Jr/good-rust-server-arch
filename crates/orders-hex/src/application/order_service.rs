use crate::errors::AppError;
use orders_types::domain::order::{Order, OrderItem, OrderStatus};
use orders_types::ports::order_repository::OrderRepository;
use uuid::Uuid;

pub struct OrderService<R: OrderRepository> {
    repo: R,
}

impl<R: OrderRepository> OrderService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create_order(
        &self,
        customer_name: String,
        email: String,
        items: Vec<OrderItem>,
    ) -> Result<Order, AppError> {
        let order = Order::new(customer_name, email, items)
            .map_err(|e| AppError::BadRequest(e.to_string()))?;
        self.repo
            .create(order.clone())
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e.to_string())))?;
        Ok(order)
    }

    pub async fn get_order(&self, id: Uuid) -> Result<Order, AppError> {
        match self
            .repo
            .get(id)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e.to_string())))?
        {
            Some(o) => Ok(o),
            None => Err(AppError::NotFound(format!("order {}", id))),
        }
    }

    pub async fn list_orders(&self) -> Result<Vec<Order>, AppError> {
        self.repo
            .list()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e.to_string())))
    }

    pub async fn update_status(&self, id: Uuid, status: OrderStatus) -> Result<Order, AppError> {
        match self
            .repo
            .update_status(id, status)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e.to_string())))?
        {
            Some(o) => Ok(o),
            None => Err(AppError::NotFound(format!("order {}", id))),
        }
    }

    pub async fn delete_order(&self, id: Uuid) -> Result<(), AppError> {
        let deleted = self
            .repo
            .delete(id)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e.to_string())))?;
        if deleted {
            Ok(())
        } else {
            Err(AppError::NotFound(format!("order {}", id)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use orders_types::domain::order::OrderItem;

    #[tokio::test]
    async fn create_and_get_order_in_memory() {
        let repo = orders_repo::memory::InMemoryRepo::new();
        let svc = OrderService::new(repo.clone());
        let items = vec![OrderItem {
            name: "Widget".into(),
            qty: 2,
            unit_price_cents: 500,
        }];
        let res = svc
            .create_order("Alice".into(), "a@b.com".into(), items.clone())
            .await;
        assert!(res.is_ok());
        let order = res.unwrap();
        let got = svc.get_order(order.id).await.unwrap();
        assert_eq!(got.customer_name, "Alice");
        assert_eq!(got.total_cents, 1000);
    }

    #[tokio::test]
    async fn update_status_and_delete() {
        let repo = orders_repo::memory::InMemoryRepo::new();
        let svc = OrderService::new(repo.clone());
        let items = vec![OrderItem {
            name: "Widget".into(),
            qty: 1,
            unit_price_cents: 250,
        }];
        let order = svc
            .create_order("Bob".into(), "bob@example.com".into(), items)
            .await
            .unwrap();

        let updated = svc
            .update_status(order.id, OrderStatus::Shipped)
            .await
            .unwrap();
        assert_eq!(updated.status, OrderStatus::Shipped);

        svc.delete_order(order.id).await.unwrap();
        let missing = svc.get_order(order.id).await;
        assert!(matches!(missing, Err(AppError::NotFound(_))));
    }

    #[tokio::test]
    async fn validation_errors_propagate() {
        let repo = orders_repo::memory::InMemoryRepo::new();
        let svc = OrderService::new(repo.clone());
        let res = svc.create_order("".into(), "invalid".into(), vec![]).await;
        assert!(matches!(res, Err(AppError::BadRequest(_))));
    }

    #[tokio::test]
    async fn not_found_paths() {
        let repo = orders_repo::memory::InMemoryRepo::new();
        let svc = OrderService::new(repo.clone());
        let missing = svc.get_order(uuid::Uuid::new_v4()).await;
        assert!(matches!(missing, Err(AppError::NotFound(_))));

        let updated = svc
            .update_status(uuid::Uuid::new_v4(), OrderStatus::Shipped)
            .await;
        assert!(matches!(updated, Err(AppError::NotFound(_))));

        let deleted = svc.delete_order(uuid::Uuid::new_v4()).await;
        assert!(matches!(deleted, Err(AppError::NotFound(_))));
    }
}
