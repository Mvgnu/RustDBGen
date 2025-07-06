use rustdbgen::connect_any_pool;

#[tokio::test]
async fn connect_any_pool_sqlite() {
    let pool = connect_any_pool("sqlite::memory:").await.unwrap();
    let (val,): (i32,) = sqlx::query_as("SELECT 1").fetch_one(&pool).await.unwrap();
    assert_eq!(val, 1);
}
