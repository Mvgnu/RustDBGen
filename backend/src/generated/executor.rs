/// A trait for database executors that can work with Postgres.
/// This allows functions to accept both connection pools and connections.
/// For transactions, use the deref pattern: &mut *transaction
pub trait PgExecutor<'c>: sqlx::Executor<'c, Database = sqlx::Postgres> + Send + Sync {}

impl<'c> PgExecutor<'c> for &'c sqlx::PgPool {}
impl<'c> PgExecutor<'c> for &'c mut sqlx::PgConnection {}
