# orders-app

Binary crate that wires config + repository + HTTP server for the Orders API.

## Run

SQLite (default features):
```bash
export DATABASE_URL="sqlite://data/orders.db"
cargo run
```

Memory:
```bash
cargo run --no-default-features --features memory
```

## Health check
`GET /health` returns `{"status":"ok"}`.

## Using the client
Once the server is running, the `orders-client` crate can hit it:
```rust
use orders_client::{OrdersClient, CreateOrderRequest};
use orders_types::domain::order::OrderItem;

# #[tokio::main]
# async fn main() -> anyhow::Result<()> {
let client = OrdersClient::new("http://127.0.0.1:3000/")?;
let created = client.create_order(CreateOrderRequest {
    customer_name: "Example".into(),
    email: "example@example.com".into(),
    items: vec![OrderItem { name: "Widget".into(), qty: 1, unit_price_cents: 500 }],
}).await?;
println!("Created order id={}", created.id);
# Ok(())
# }
```
