#![cfg(feature = "memory")]

use orders_repo::memory::InMemoryRepo;
use orders_types::domain::order::{OrderItem, OrderStatus};
use orders_types::ports::order_repository::OrderRepository;

#[tokio::test]
async fn memory_repo_crud_flow() {
    let repo = InMemoryRepo::new();
    let order = orders_types::domain::order::Order::new(
        "Test".into(),
        "test@example.com".into(),
        vec![OrderItem {
            name: "Widget".into(),
            qty: 2,
            unit_price_cents: 500,
        }],
    )
    .unwrap();

    let created = repo.create(order.clone()).await.unwrap();
    assert_eq!(created.id, order.id);

    let fetched = repo.get(order.id).await.unwrap().unwrap();
    assert_eq!(fetched.customer_name, "Test");

    let listed = repo.list().await.unwrap();
    assert_eq!(listed.len(), 1);

    let updated = repo
        .update_status(order.id, OrderStatus::Shipped)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated.status, OrderStatus::Shipped);

    let deleted = repo.delete(order.id).await.unwrap();
    assert!(deleted);
    assert!(repo.get(order.id).await.unwrap().is_none());
}

#[tokio::test]
async fn memory_repo_handles_missing_rows() {
    let repo = InMemoryRepo::new();
    let missing = repo.get(uuid::Uuid::new_v4()).await.unwrap();
    assert!(missing.is_none());

    let updated = repo
        .update_status(uuid::Uuid::new_v4(), OrderStatus::Shipped)
        .await
        .unwrap();
    assert!(updated.is_none());

    let deleted = repo.delete(uuid::Uuid::new_v4()).await.unwrap();
    assert!(!deleted);
}
