# orders-hex workspace

Hexagonal Orders API split into:
- `crates/orders-types` — shared domain types and port traits.
- `crates/orders-repo` — database adapters behind features (`memory` / `sqlite`) exposing `Repo` and `build_repo`.
- `crates/orders-hex` — application/service layer and HTTP adapter. Depends on `orders-types`.
- `crates/orders-app` — binary crate wiring config + repo + HTTP server. Default workspace member, so `cargo run` targets it.
- `crates/orders-client` — typed HTTP client for the Orders API (reqwest-based).

## Build & Run (memory)
```bash
cargo run --no-default-features --features memory
# Server binds to port 3000 by default (set SERVER_PORT env to change)
```

## Build & Run (sqlite)

```bash
export DATABASE_URL="sqlite://data/orders.db"  # parent dir auto-created; file auto-created
cargo run                     # default feature on orders-app is sqlite
# or explicitly:
# cargo run --no-default-features --features sqlite
```
# Migrations live at `crates/orders-repo/migrations` (applied automatically by the SQLite adapter).

Note: `orders-repo` features are mutually exclusive; do not enable both `memory` and `sqlite`.

## Tests

Run library + integration tests (uses in-memory repo via dev-deps):

```bash
cargo test -p orders-hex
```

Crate-specific:
- Domain/ports: `cargo test -p orders-types`
- Repo adapters (memory default): `cargo test -p orders-repo`
- Repo sqlite adapter: `cargo test -p orders-repo --features sqlite`
- App wiring (sqlite default): `cargo test -p orders-app`
- App wiring (memory): `cargo test -p orders-app --no-default-features --features memory`

Full suite:
- `./validate_all.sh` runs checks, clippy, feature-matrix tests, and release builds.

Logging:
- `orders-app` defaults `RUST_LOG=debug` if unset. Set `RUST_LOG=info` for quieter logs.

Feature defaults:
- `orders-app` default features enable `sqlite`.
- `orders-repo` default features enable `memory`; use `--no-default-features --features sqlite` to flip.

Migrations:
- SQLite adapter auto-applies `crates/orders-repo/migrations/0001_create_orders.sql` on startup.

## Endpoints

* `POST /orders` -> create order  
  JSON body: `{ "customer_name": "...", "email": "...", "items":[{"name":"...", "qty":1, "unit_price_cents":100}] }`
* `GET /health` -> liveness probe
* `GET /orders/{id}` -> fetch order
* `GET /orders` -> list orders
* `PATCH /orders/{id}/status` -> update status `{"status":"Shipped"}`
* `DELETE /orders/{id}` -> delete

## Design notes

* Hexagonal: domain + application + ports + adapters
* DB adaptors are compile-time optional (memory / sqlite)
* Domain validation lives in `domain::order`
* Service layer returns `AppError` mapped to HTTP responses
* Feature flags keep deps lean: `orders-app` defaults to sqlite; `orders-repo` defaults to memory for tests.
* Structured logging with per-request IDs; health endpoint at `/health`.

## Quick usage examples

Create order:
```bash
curl -X POST http://127.0.0.1:3000/orders \
  -H "Content-Type: application/json" \
  -d '{"customer_name":"Alice","email":"alice@example.com","items":[{"name":"Widget","qty":2,"unit_price_cents":500}]}'
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
            items: vec![OrderItem { name: "Widget".into(), qty: 2, unit_price_cents: 500 }],
        })
        .await?;
    println!("created id={}", created.id);
    Ok(())
}
```
For an end-to-end run, launch `orders-app` and point `orders-client` at it; see the client README for usage.
