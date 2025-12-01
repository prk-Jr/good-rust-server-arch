# orders-client

Typed HTTP client for the Orders API.

## Usage

```rust
use orders_client::{OrdersClient, CreateOrderRequest};
use orders_types::domain::order::OrderItem;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = OrdersClient::builder("http://127.0.0.1:3000/")?.build()?;
    let created = client
        .create_order(CreateOrderRequest {
            customer_name: "Alice".into(),
            email: "alice@example.com".into(),
            items: vec![OrderItem { name: "Widget".into(), qty: 2, unit_price_cents: 500 }],
        })
        .await?;
    println!("created id={}", created.id);
    Ok(())
}
```

## Builder options
- `with_timeout(Duration)`: set HTTP request timeout.
- `with_header(key, value)`: add a default header (e.g., auth).
- `with_reqwest_client(reqwest::Client)`: supply a preconfigured client.

## End-to-end example

Run the server (e.g., `cargo run` in `orders-app` with sqlite or memory), then use `OrdersClient` in your app or in an example to hit it. The workspace `orders-app/examples` shows a quick in-process demo pattern.
