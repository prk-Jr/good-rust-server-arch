use orders_hex::application::order_service::OrderService;
use orders_repo::memory::InMemoryRepo;
use orders_types::domain::order::{OrderItem, OrderStatus};

// End-to-end service flow against the in-memory adapter.
#[tokio::test]
async fn create_list_update_delete_flow() {
    let repo = InMemoryRepo::new();
    let svc = OrderService::new(repo.clone());

    let order = svc
        .create_order(
            "Eve".into(),
            "eve@example.com".into(),
            vec![OrderItem {
                name: "Gadget".into(),
                qty: 3,
                unit_price_cents: 700,
            }],
        )
        .await
        .unwrap();

    let list = svc.list_orders().await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, order.id);

    let updated = svc
        .update_status(order.id, OrderStatus::Confirmed)
        .await
        .unwrap();
    assert_eq!(updated.status, OrderStatus::Confirmed);

    svc.delete_order(order.id).await.unwrap();
    let after_delete = svc.list_orders().await.unwrap();
    assert!(after_delete.is_empty());
}
