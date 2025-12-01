use orders_hex::application::order_service::OrderService;
use orders_hex::inbound::http::{HttpServer, HttpServerConfig};
use orders_repo::build_repo;
use orders_types::domain::order::{Order, OrderItem, OrderStatus};
use serde::{Deserialize, Serialize};

fn find_free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

#[derive(Serialize)]
struct OrderInput {
    customer_name: String,
    email: String,
    items: Vec<OrderItem>,
}

#[derive(Serialize)]
struct UpdateStatus {
    status: OrderStatus,
}

#[tokio::test]
async fn create_list_update_delete_over_http() {
    let port = find_free_port();
    let config = HttpServerConfig {
        port: port.to_string(),
    };

    let repo = build_repo(None).await.expect("build repo");
    let service = OrderService::new(repo);
    let server = HttpServer::new(service, config).await.unwrap();

    let addr = format!("http://127.0.0.1:{}", port);
    let handle = tokio::spawn(async move {
        server.run().await.expect("server run");
    });

    // Give the server a moment to start.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let client = reqwest::Client::new();
    let create_body = OrderInput {
        customer_name: "HttpUser".into(),
        email: "http@example.com".into(),
        items: vec![OrderItem {
            name: "Widget".into(),
            qty: 1,
            unit_price_cents: 500,
        }],
    };

    #[derive(Deserialize)]
    struct Created {
        id: String,
        status: OrderStatus,
    }

    let res = client
        .post(format!("{}/orders", addr))
        .json(&create_body)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), reqwest::StatusCode::CREATED);
    let created: Created = res.json().await.unwrap();
    let id = created.id.clone();
    assert_eq!(created.status, OrderStatus::Pending);

    let fetched: Order = client
        .get(format!("{}/orders/{}", addr, id))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(fetched.customer_name, "HttpUser");

    let list: Vec<Order> = client
        .get(format!("{}/orders", addr))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id.to_string(), id);

    let update_body = UpdateStatus {
        status: OrderStatus::Shipped,
    };
    let res = client
        .patch(format!("{}/orders/{}/status", addr, id))
        .json(&update_body)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), reqwest::StatusCode::OK);
    let updated: Order = res.json().await.unwrap();
    assert_eq!(updated.status, OrderStatus::Shipped);

    let res = client
        .delete(format!("{}/orders/{}", addr, id))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), reqwest::StatusCode::NO_CONTENT);

    // stop server task
    handle.abort();
}

#[tokio::test]
async fn bad_request_and_not_found_paths() {
    let port = find_free_port();
    let config = HttpServerConfig {
        port: port.to_string(),
    };
    let repo = build_repo(None).await.expect("build repo");
    let service = OrderService::new(repo);
    let server = HttpServer::new(service, config).await.unwrap();
    let addr = format!("http://127.0.0.1:{}", port);
    let handle = tokio::spawn(async move {
        server.run().await.expect("server run");
    });
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let client = reqwest::Client::new();
    let bad_body = OrderInput {
        customer_name: "".into(),
        email: "invalid".into(),
        items: vec![],
    };
    let res = client
        .post(format!("{}/orders", addr))
        .json(&bad_body)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), reqwest::StatusCode::BAD_REQUEST);

    let missing_id = uuid::Uuid::new_v4();
    let res = client
        .get(format!("{}/orders/{}", addr, missing_id))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), reqwest::StatusCode::NOT_FOUND);

    handle.abort();
}
