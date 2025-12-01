#[cfg(not(any(feature = "memory", feature = "sqlite")))]
compile_error!("Enable a repo feature: `memory` or `sqlite`.");

use orders_types::domain::order::*;
use orders_types::ports::order_repository::OrderRepository;
use orders_types::ports::order_repository::RepoError;
use uuid::Uuid;

#[cfg(feature = "memory")]
pub mod memory;
#[cfg(feature = "sqlite")]
pub mod sqlite;

pub struct Repo {
    #[cfg(feature = "memory")]
    memory: memory::InMemoryRepo,
    #[cfg(feature = "sqlite")]
    sqlite: sqlite::SqliteRepo,
}

pub async fn build_repo(url: Option<&str>) -> anyhow::Result<Repo> {
    Repo::build_repo(url).await
}

impl Repo {
    #[cfg(all(feature = "memory", not(feature = "sqlite")))]
    pub async fn build_repo(_: Option<&str>) -> anyhow::Result<Self> {
        Ok(Self {
            memory: crate::memory::InMemoryRepo::new(),
        })
    }

    #[cfg(all(feature = "sqlite", not(feature = "memory")))]
    pub async fn build_repo(database_url: Option<&str>) -> anyhow::Result<Self> {
        let url = database_url.unwrap_or("sqlite://orders.db");
        let sqlite = sqlite::SqliteRepo::new(url).await?;
        Ok(Self { sqlite })
    }

    // If both features are enabled
    #[cfg(all(feature = "sqlite", feature = "memory"))]
    pub async fn build_repo(database_url: Option<&str>) -> anyhow::Result<Self> {
        let memory = crate::memory::InMemoryRepo::new();
        let url = database_url.unwrap_or("sqlite://orders.db");
        let sqlite = sqlite::SqliteRepo::new(url).await?;
        Ok(Self { memory, sqlite })
    }
}

#[cfg(all(feature = "memory", not(feature = "sqlite")))]
#[async_trait::async_trait]
impl OrderRepository for Repo {
    async fn create(&self, order: Order) -> Result<Order, RepoError> {
        self.memory.create(order).await
    }

    async fn get(&self, id: Uuid) -> Result<Option<Order>, RepoError> {
        self.memory.get(id).await
    }

    async fn list(&self) -> Result<Vec<Order>, RepoError> {
        self.memory.list().await
    }

    async fn update_status(
        &self,
        id: Uuid,
        status: OrderStatus,
    ) -> Result<Option<Order>, RepoError> {
        self.memory.update_status(id, status).await
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepoError> {
        self.memory.delete(id).await
    }
}

#[cfg(all(feature = "sqlite", not(feature = "memory")))]
#[async_trait::async_trait]
impl OrderRepository for Repo {
    async fn create(&self, order: Order) -> Result<Order, RepoError> {
        self.sqlite.create(order).await
    }

    async fn get(&self, id: Uuid) -> Result<Option<Order>, RepoError> {
        self.sqlite.get(id).await
    }

    async fn list(&self) -> Result<Vec<Order>, RepoError> {
        self.sqlite.list().await
    }

    async fn update_status(
        &self,
        id: Uuid,
        status: OrderStatus,
    ) -> Result<Option<Order>, RepoError> {
        self.sqlite.update_status(id, status).await
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepoError> {
        self.sqlite.delete(id).await
    }
}

#[cfg(all(feature = "sqlite", feature = "memory"))]
#[async_trait::async_trait]
impl OrderRepository for Repo {
    async fn create(&self, order: Order) -> Result<Order, RepoError> {
        // let order  = self.memory.create(order).await?;
        self.sqlite.create(order).await
    }

    async fn get(&self, id: Uuid) -> Result<Option<Order>, RepoError> {
        // let order = self.memory.get(id).await?;
        // if order.is_none() {
        //     self.sqlite.get(id).await
        // } else {
        //     Ok(order)
        // }
        self.sqlite.get(id).await
    }

    async fn list(&self) -> Result<Vec<Order>, RepoError> {
        // self.memory.list().await
        self.sqlite.list().await
    }

    async fn update_status(
        &self,
        id: Uuid,
        status: OrderStatus,
    ) -> Result<Option<Order>, RepoError> {
        // self.memory.update_status(id, status).await
        self.sqlite.update_status(id, status).await
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepoError> {
        self.memory.delete(id).await
        // self.sqlite.delete(id).await
    }
}
