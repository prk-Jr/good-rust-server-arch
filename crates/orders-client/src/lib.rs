use std::time::Duration;

use anyhow::Context;
use orders_types::domain::order::{Order, OrderItem, OrderStatus};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::Url;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct OrdersClientBuilder {
    base: Url,
    headers: HeaderMap,
    timeout: Option<Duration>,
    client: Option<reqwest::Client>,
}

#[derive(Clone)]
pub struct OrdersClient {
    base: Url,
    client: reqwest::Client,
}

impl OrdersClient {
    pub fn new(base_url: &str) -> anyhow::Result<Self> {
        Self::builder(base_url)?.build()
    }

    pub fn builder(base_url: &str) -> anyhow::Result<OrdersClientBuilder> {
        let base = Url::parse(base_url).context("invalid base url")?;
        Ok(OrdersClientBuilder {
            base,
            headers: HeaderMap::new(),
            timeout: None,
            client: None,
        })
    }

    fn url(&self, path: &str) -> anyhow::Result<Url> {
        self.base.join(path).context("failed to join url")
    }

    pub async fn create_order(
        &self,
        req: CreateOrderRequest,
    ) -> anyhow::Result<CreateOrderResponse> {
        let res = self
            .client
            .post(self.url("orders")?)
            .json(&req)
            .send()
            .await?
            .error_for_status()?;
        Ok(res.json().await?)
    }

    pub async fn get_order(&self, id: &str) -> anyhow::Result<Order> {
        let res = self
            .client
            .get(self.url(&format!("orders/{id}"))?)
            .send()
            .await?
            .error_for_status()?;
        Ok(res.json().await?)
    }

    pub async fn list_orders(&self) -> anyhow::Result<Vec<Order>> {
        let res = self
            .client
            .get(self.url("orders")?)
            .send()
            .await?
            .error_for_status()?;
        Ok(res.json().await?)
    }

    pub async fn update_status(&self, id: &str, status: OrderStatus) -> anyhow::Result<Order> {
        let res = self
            .client
            .patch(self.url(&format!("orders/{id}/status"))?)
            .json(&UpdateStatusRequest { status })
            .send()
            .await?
            .error_for_status()?;
        Ok(res.json().await?)
    }

    pub async fn delete_order(&self, id: &str) -> anyhow::Result<()> {
        self.client
            .delete(self.url(&format!("orders/{id}"))?)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

impl OrdersClientBuilder {
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn with_header(
        mut self,
        key: impl AsRef<str>,
        value: impl AsRef<str>,
    ) -> anyhow::Result<Self> {
        let header_name =
            HeaderName::from_bytes(key.as_ref().as_bytes()).context("invalid header name")?;
        let header_value = HeaderValue::from_str(value.as_ref()).context("invalid header value")?;
        self.headers.insert(header_name, header_value);
        Ok(self)
    }

    pub fn with_reqwest_client(mut self, client: reqwest::Client) -> Self {
        self.client = Some(client);
        self
    }

    pub fn build(self) -> anyhow::Result<OrdersClient> {
        if let Some(client) = self.client {
            return Ok(OrdersClient {
                base: self.base,
                client,
            });
        }

        let mut builder = reqwest::Client::builder();
        if !self.headers.is_empty() {
            builder = builder.default_headers(self.headers);
        }
        if let Some(t) = self.timeout {
            builder = builder.timeout(t);
        }
        let client = builder.build()?;
        Ok(OrdersClient {
            base: self.base,
            client,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateOrderRequest {
    pub customer_name: String,
    pub email: String,
    pub items: Vec<OrderItem>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CreateOrderResponse {
    pub id: String,
    pub status: OrderStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct UpdateStatusRequest {
    status: OrderStatus,
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;

    fn sample_order() -> Order {
        Order {
            id: uuid::Uuid::new_v4(),
            customer_name: "User".into(),
            email: "user@example.com".into(),
            items: vec![OrderItem {
                name: "Widget".into(),
                qty: 1,
                unit_price_cents: 500,
            }],
            total_cents: 500,
            status: OrderStatus::Pending,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn create_and_get_order() {
        let server = MockServer::start();
        let order = sample_order();

        let create_mock = server.mock(|when, then| {
            when.method(POST)
                .path("/orders")
                .json_body_obj(&CreateOrderRequest {
                    customer_name: order.customer_name.clone(),
                    email: order.email.clone(),
                    items: order.items.clone(),
                });
            then.status(201).json_body_obj(&CreateOrderResponse {
                id: order.id.to_string(),
                status: OrderStatus::Pending,
            });
        });

        let get_mock: httpmock::Mock<'_> = server.mock(|when, then| {
            when.method(GET).path(format!("/orders/{}", order.id));
            then.status(200).json_body_obj(&order);
        });

        let client = OrdersClient::new(&server.base_url()).unwrap();
        let created = client
            .create_order(CreateOrderRequest {
                customer_name: order.customer_name.clone(),
                email: order.email.clone(),
                items: order.items.clone(),
            })
            .await
            .unwrap();
        assert_eq!(created.id, order.id.to_string());
        assert_eq!(created.status, OrderStatus::Pending);

        let fetched = client.get_order(&order.id.to_string()).await.unwrap();
        assert_eq!(fetched.email, order.email);

        create_mock.assert();
        get_mock.assert();
    }

    #[tokio::test]
    async fn list_update_delete() {
        let server = MockServer::start();
        let order = sample_order();

        let list_mock = server.mock(|when, then| {
            when.method(GET).path("/orders");
            then.status(200).json_body_obj(&vec![order.clone()]);
        });

        let update_mock = server.mock(|when, then| {
            when.method(httpmock::Method::PATCH)
                .path(format!("/orders/{}/status", order.id))
                .json_body_obj(&UpdateStatusRequest {
                    status: OrderStatus::Shipped,
                });
            let mut updated = order.clone();
            updated.status = OrderStatus::Shipped;
            then.status(200).json_body_obj(&updated);
        });

        let delete_mock = server.mock(|when, then| {
            when.method(DELETE).path(format!("/orders/{}", order.id));
            then.status(204);
        });

        let client = OrdersClient::new(&server.base_url()).unwrap();
        let listed = client.list_orders().await.unwrap();
        assert_eq!(listed.len(), 1);

        let updated = client
            .update_status(&order.id.to_string(), OrderStatus::Shipped)
            .await
            .unwrap();
        assert_eq!(updated.status, OrderStatus::Shipped);

        client.delete_order(&order.id.to_string()).await.unwrap();

        list_mock.assert();
        update_mock.assert();
        delete_mock.assert();
    }
}
