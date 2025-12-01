///  To run :
///  cargo r --example client_example
use orders_client::{CreateOrderRequest, OrdersClient};
use orders_hex::application::order_service::OrderService;
use orders_hex::inbound::http::{HttpServer, HttpServerConfig};
use orders_repo::build_repo;
use orders_types::domain::order::{OrderItem, OrderStatus};
use reqwest::StatusCode;
use tempfile::tempdir;

fn find_free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Start server on ephemeral port with in-memory repo.
    let port = find_free_port();
    let addr = format!("http://127.0.0.1:{port}/");

    // Use a temp file-backed SQLite DB so multiple connections see the same data.
    let tmp = tempdir()?;
    let db_path = tmp.path().join("orders.db");
    let db_url = format!("sqlite://{}", db_path.display());

    let repo = build_repo(Some(&db_url)).await?;
    let service = OrderService::new(repo);
    let server = HttpServer::new(
        service,
        HttpServerConfig {
            port: port.to_string(),
        },
    )
    .await?;

    let handle = tokio::spawn(async move {
        server.run().await.expect("server run");
    });
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Use client against the running server.
    let client = OrdersClient::new(&addr)?;
    let created = client
        .create_order(CreateOrderRequest {
            customer_name: "Example".into(),
            email: "example@example.com".into(),
            items: vec![OrderItem {
                name: "Widget".into(),
                qty: 1,
                unit_price_cents: 500,
            }],
        })
        .await?;
    println!("Created order id={}", created.id);
    assert_eq!(created.status, OrderStatus::Pending);

    let fetched = client.get_order(&created.id).await?;
    println!("Fetched status={:?}", fetched.status);
    assert_eq!(fetched.email, "example@example.com");

    let updated = client
        .update_status(&created.id, OrderStatus::Shipped)
        .await?;
    println!(
        "Updated status={:?} for id {:?}",
        updated.status, updated.id
    );
    assert_eq!(updated.status, OrderStatus::Shipped);

    // Try to delete; if the record vanished, create another and ensure delete succeeds.
    match client.delete_order(&created.id).await {
        Ok(()) => println!("Deleted order"),
        Err(err) => {
            if err
                .downcast_ref::<reqwest::Error>()
                .and_then(|e| e.status())
                == Some(StatusCode::NOT_FOUND)
            {
                println!("Delete returned 404; creating a fresh order to demonstrate delete...");
                let alt = client
                    .create_order(CreateOrderRequest {
                        customer_name: "Example2".into(),
                        email: "example2@example.com".into(),
                        items: vec![OrderItem {
                            name: "Gadget".into(),
                            qty: 1,
                            unit_price_cents: 700,
                        }],
                    })
                    .await?;
                client.delete_order(&alt.id).await?;
                println!("Deleted second order id={}", alt.id);
            } else {
                return Err(err);
            }
        }
    }

    handle.abort();
    Ok(())
}
