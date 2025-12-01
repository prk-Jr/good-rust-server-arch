# Orders API

A production-grade Rust implementation of a Hexagonal Architecture (Ports & Adapters) for a simple Orders API.

## Workspace layout
- `crates/orders-types` - domain types + port traits
- `crates/orders-repo` - database adapters (memory / sqlite)
- `crates/orders-hex` - application layer + HTTP inbound adapter
- `crates/orders-app` - binary crate wiring config + repo + server
- `crates/orders-client` - typed HTTP client

## Features & architecture
- Hexagonal design: domain logic isolated behind ports; adapters implement the ports
- Repository is a port; select adapter via Cargo features
- Two DB adapters:
  - `memory`: DashMap-based repository
  - `sqlite`: SQLx adapter with auto-applied migrations
- HTTP inbound adapter built on Axum 0.8 (+ tower-http tracing)
- Errors map cleanly into structured HTTP responses
- Feature-gated dependencies keep builds lean and tests fast
  - Defaults: `orders-app` -> `sqlite`, `orders-repo` -> `memory`
  - Prefer enabling exactly one repo feature (`memory` or `sqlite`)

## Running the API
### In-memory repository (default for tests)
```bash
cargo run --no-default-features --features memory
```
Runs on port 3000 unless `SERVER_PORT` is set.

### SQLite repository (default for `orders-app`)
```bash
export DATABASE_URL="sqlite://data/orders.db"
cargo run                  # uses sqlite (default feature of orders-app)
# or:
cargo run --no-default-features --features sqlite
```
Migrations live in `crates/orders-repo/migrations/` and are applied on startup.

## Testing
- Domain & ports: `cargo test -p orders-types`
- Repo adapters: `cargo test -p orders-repo` (memory default) / `cargo test -p orders-repo --features sqlite`
- Application + HTTP: `cargo test -p orders-hex`
- App wiring: `cargo test -p orders-app` (sqlite) / `cargo test -p orders-app --no-default-features --features memory`
- Run everything: `cargo test --all`
- Full validation: `./validate_all.sh` (checks, clippy, feature-matrix tests, release builds)

## API endpoints
- `POST /orders` - create order
- `GET /orders/{id}` - get order by ID
- `GET /orders` - list all orders
- `PATCH /orders/{id}/status` - update order status
- `DELETE /orders/{id}` - delete an order
- `GET /health` - health check

## Example requests
Create order:
```bash
curl -X POST http://127.0.0.1:3000/orders \
  -H "Content-Type: application/json" \
  -d '{
    "customer_name": "Alice",
    "email": "alice@example.com",
    "items":[{"name":"Widget","qty":2,"unit_price_cents":500}]
  }'
```

List:
```bash
curl http://127.0.0.1:3000/orders
```

Update status:
```bash
curl -X PATCH http://127.0.0.1:3000/orders/<id>/status \
  -H "Content-Type: application/json" \
  -d '{"status":"Shipped"}'
```

Delete:
```bash
curl -X DELETE http://127.0.0.1:3000/orders/<id>
```

## HTTP client (`orders-client`)
```rust
use orders_client::{OrdersClient, CreateOrderRequest};
use orders_types::domain::order::OrderItem;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = OrdersClient::new("http://127.0.0.1:3000/")?;

    let created = client
        .create_order(CreateOrderRequest {
            customer_name: "Alice".into(),
            email: "alice@example.com".into(),
            items: vec![
                OrderItem {
                    name: "Widget".into(),
                    qty: 2,
                    unit_price_cents: 500,
                },
            ],
        })
        .await?;

    println!("created id={}", created.id);
    Ok(())
}
```

## Design notes
- Domain validation lives in `orders-types`; application layer orchestrates interactions
- Compile-time adapter selection via features (`memory` vs `sqlite`)
- Structured tracing with per-request IDs (`RUST_LOG` defaults to `debug` if unset)
- SQLite adapter applies migrations from `crates/orders-repo/migrations/0001_create_orders.sql` on startup
