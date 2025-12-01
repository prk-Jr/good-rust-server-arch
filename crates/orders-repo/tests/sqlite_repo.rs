#![cfg(feature = "sqlite")]

use orders_repo::sqlite::SqliteRepo;
use orders_types::domain::order::{OrderItem, OrderStatus};
use orders_types::ports::order_repository::OrderRepository;
use std::path::PathBuf;
use uuid::Uuid;

fn temp_db_url() -> (tempfile::TempDir, String) {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut path = PathBuf::from(dir.path());
    path.push(format!("orders-{}.db", Uuid::new_v4()));
    let url = format!("sqlite://{}", path.display());
    (dir, url)
}

#[tokio::test]
async fn sqlite_repo_crud_flow() {
    let (_dir, url) = temp_db_url();
    let repo = SqliteRepo::new(&url).await.unwrap();

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
async fn sqlite_repo_handles_missing_rows() {
    let (_dir, url) = temp_db_url();
    let repo = SqliteRepo::new(&url).await.unwrap();
    let missing_id = uuid::Uuid::new_v4();

    let missing = repo.get(missing_id).await.unwrap();
    assert!(missing.is_none());

    let updated = repo
        .update_status(missing_id, OrderStatus::Shipped)
        .await
        .unwrap();
    assert!(updated.is_none());

    let deleted = repo.delete(missing_id).await.unwrap();
    assert!(!deleted);
}
