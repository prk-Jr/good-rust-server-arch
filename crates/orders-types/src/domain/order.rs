use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Shipped,
    Cancelled,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    pub name: String,
    pub qty: u32,
    pub unit_price_cents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub customer_name: String,
    pub email: String,
    pub items: Vec<OrderItem>,
    pub total_cents: i64,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Order {
    pub fn new(
        customer_name: String,
        email: String,
        items: Vec<OrderItem>,
    ) -> anyhow::Result<Self> {
        if customer_name.trim().is_empty() {
            anyhow::bail!("customer_name empty");
        }
        if !email.contains('@') {
            anyhow::bail!("invalid email");
        }
        if items.is_empty() {
            anyhow::bail!("items empty");
        }
        for it in &items {
            if it.qty == 0 {
                anyhow::bail!("item qty must be > 0");
            }
        }
        let total = items
            .iter()
            .map(|it| (it.qty as i64) * it.unit_price_cents)
            .sum();
        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            customer_name,
            email,
            items,
            total_cents: total,
            status: OrderStatus::Pending,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn update_status(&mut self, status: OrderStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_order_computes_total_and_defaults_pending() {
        let items = vec![
            OrderItem {
                name: "A".into(),
                qty: 2,
                unit_price_cents: 500,
            },
            OrderItem {
                name: "B".into(),
                qty: 1,
                unit_price_cents: 250,
            },
        ];
        let order = Order::new("Alice".into(), "a@b.com".into(), items).unwrap();
        assert_eq!(order.total_cents, 1250);
        assert_eq!(order.status, OrderStatus::Pending);
    }

    #[test]
    fn validation_errors() {
        let empty_name = Order::new(
            "".into(),
            "a@b.com".into(),
            vec![OrderItem {
                name: "A".into(),
                qty: 1,
                unit_price_cents: 100,
            }],
        );
        assert!(empty_name.is_err());

        let bad_email = Order::new(
            "Bob".into(),
            "invalid".into(),
            vec![OrderItem {
                name: "A".into(),
                qty: 1,
                unit_price_cents: 100,
            }],
        );
        assert!(bad_email.is_err());

        let empty_items = Order::new("Bob".into(), "b@c.com".into(), vec![]);
        assert!(empty_items.is_err());

        let zero_qty = Order::new(
            "Bob".into(),
            "b@c.com".into(),
            vec![OrderItem {
                name: "A".into(),
                qty: 0,
                unit_price_cents: 100,
            }],
        );
        assert!(zero_qty.is_err());
    }

    #[test]
    fn update_status_mutates_timestamp() {
        let mut order = Order::new(
            "Carol".into(),
            "c@d.com".into(),
            vec![OrderItem {
                name: "A".into(),
                qty: 1,
                unit_price_cents: 100,
            }],
        )
        .unwrap();
        let before = order.updated_at;
        order.update_status(OrderStatus::Shipped);
        assert_eq!(order.status, OrderStatus::Shipped);
        assert!(order.updated_at > before);
    }
}
