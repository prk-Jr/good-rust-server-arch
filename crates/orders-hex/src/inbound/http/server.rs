use axum::{
    extract::State,
    routing::{delete, get, patch, post},
    serve, Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

use crate::application::order_service::OrderService;
use crate::errors::AppError;
use orders_types::domain::order::{OrderItem, OrderStatus};

#[derive(Clone)]
pub struct HttpServerConfig {
    pub port: String,
}

#[derive(Clone)]
pub struct HttpServer<R>
where
    R: orders_types::ports::order_repository::OrderRepository,
{
    pub service: Arc<OrderService<R>>,
    pub config: HttpServerConfig,
}

#[derive(Deserialize)]
pub struct CreateOrderRequest {
    pub customer_name: String,
    pub email: String,
    pub items: Vec<OrderItem>,
}

#[derive(Deserialize)]
pub struct UpdateStatusRequest {
    pub status: OrderStatus,
}

#[derive(Serialize)]
struct CreateOrderResponse {
    id: String,
    status: OrderStatus,
}

impl From<orders_types::domain::order::Order> for CreateOrderResponse {
    fn from(o: orders_types::domain::order::Order) -> Self {
        Self {
            id: o.id.to_string(),
            status: o.status,
        }
    }
}

impl<R> HttpServer<R>
where
    R: orders_types::ports::order_repository::OrderRepository + Send + Sync + 'static,
{
    pub async fn new(service: OrderService<R>, config: HttpServerConfig) -> anyhow::Result<Self> {
        Ok(Self {
            service: Arc::new(service),
            config,
        })
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let trace_layer = TraceLayer::new_for_http()
            .make_span_with(|request: &axum::extract::Request<_>| {
                let uri = request.uri().to_string();
                let request_id = Uuid::new_v4();
                tracing::info_span!(
                    "http_request",
                    %request_id,
                    method = %request.method(),
                    uri
                )
            })
            .on_request(
                |request: &axum::extract::Request<_>, span: &tracing::Span| {
                    tracing::info!(
                        parent: span,
                        method = %request.method(),
                        uri = %request.uri(),
                        "request"
                    );
                },
            )
            .on_response(
                |response: &axum::response::Response, latency: Duration, span: &tracing::Span| {
                    tracing::info!(
                        parent: span,
                        status = %response.status(),
                        latency_ms = %latency.as_millis(),
                        "response"
                    );
                },
            );

        let svc = self.service.clone();
        let app = Router::new()
            .route("/health", get(health))
            .route("/orders", post(create_order::<R>))
            .route("/orders", get(list_orders::<R>))
            .route("/orders/{id}", get(get_order::<R>))
            .route("/orders/{id}/status", patch(update_status::<R>))
            .route("/orders/{id}", delete(delete_order::<R>))
            .layer(trace_layer)
            .with_state(svc);

        let addr: SocketAddr = format!("0.0.0.0:{}", self.config.port).parse()?;
        tracing::info!("starting server on {}", addr);
        let listener = tokio::net::TcpListener::bind(addr).await?;
        serve(listener, app.into_make_service()).await?;
        Ok(())
    }
}

async fn health() -> (axum::http::StatusCode, Json<serde_json::Value>) {
    (
        axum::http::StatusCode::OK,
        Json(serde_json::json!({ "status": "ok" })),
    )
}

async fn create_order<R>(
    State(service): State<Arc<OrderService<R>>>,
    Json(payload): Json<CreateOrderRequest>,
) -> Result<(axum::http::StatusCode, Json<CreateOrderResponse>), AppError>
where
    R: crate::ports::order_repository::OrderRepository + Send + Sync + 'static,
{
    let order = service
        .create_order(payload.customer_name, payload.email, payload.items)
        .await?;
    let body: CreateOrderResponse = order.into();
    Ok((axum::http::StatusCode::CREATED, Json(body)))
}

async fn get_order<R>(
    State(service): State<Arc<OrderService<R>>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<orders_types::domain::order::Order>, AppError>
where
    R: orders_types::ports::order_repository::OrderRepository + Send + Sync + 'static,
{
    let uuid = Uuid::parse_str(&id).map_err(|e| AppError::BadRequest(e.to_string()))?;
    let order = service.get_order(uuid).await?;
    Ok(Json(order))
}

async fn list_orders<R>(
    State(service): State<Arc<OrderService<R>>>,
) -> Result<Json<Vec<orders_types::domain::order::Order>>, AppError>
where
    R: orders_types::ports::order_repository::OrderRepository + Send + Sync + 'static,
{
    let list = service.list_orders().await?;
    Ok(Json(list))
}

async fn update_status<R>(
    State(service): State<Arc<OrderService<R>>>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<UpdateStatusRequest>,
) -> Result<Json<orders_types::domain::order::Order>, AppError>
where
    R: orders_types::ports::order_repository::OrderRepository + Send + Sync + 'static,
{
    let uuid = Uuid::parse_str(&id).map_err(|e| AppError::BadRequest(e.to_string()))?;
    let updated = service.update_status(uuid, payload.status).await?;
    Ok(Json(updated))
}

async fn delete_order<R>(
    State(service): State<Arc<OrderService<R>>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<(axum::http::StatusCode, Json<serde_json::Value>), AppError>
where
    R: orders_types::ports::order_repository::OrderRepository + Send + Sync + 'static,
{
    let uuid = Uuid::parse_str(&id).map_err(|e| AppError::BadRequest(e.to_string()))?;
    service.delete_order(uuid).await?;
    Ok((
        axum::http::StatusCode::NO_CONTENT,
        Json(serde_json::json!({})),
    ))
}
