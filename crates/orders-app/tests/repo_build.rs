use orders_repo::{build_repo, Repo};
use orders_types::ports::order_repository::OrderRepository;
use std::env;

#[tokio::test]
async fn builds_sqlite_repo_from_env() {
    // Use a temp DB path for isolation.
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("orders-test.db");
    let url = format!("sqlite://{}", db_path.display());
    env::set_var("DATABASE_URL", &url);

    let repo: Repo = build_repo(Some(&url)).await.expect("build repo");
    // basic sanity: list should succeed and be empty
    let list = repo.list().await.expect("list");
    assert!(list.is_empty());
}
