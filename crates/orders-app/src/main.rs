use orders_hex::application::order_service::OrderService;
use orders_hex::config::Config;
use orders_hex::inbound::http::{HttpServer, HttpServerConfig};
use orders_repo::{build_repo, Repo};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env for DATABASE_URL / SERVER_PORT when present.
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "debug".to_string()))
        .init();

    let config = Config::from_env()?;
    let repo: Repo = build_repo(config.database_url.as_deref()).await?;
    let service = OrderService::new(repo);

    let server_cfg = HttpServerConfig {
        port: config.server_port.clone(),
    };

    let http = HttpServer::new(service, server_cfg).await?;
    http.run().await
}
