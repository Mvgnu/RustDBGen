use crate::ir;

pub fn generate_executor_trait(ir: &ir::SchemaIR) -> String {
    let mut out = String::new();
    
    match ir.meta.db_backend {
        ir::DatabaseBackend::Postgres => {
            out.push_str("/// A trait for database executors that can work with Postgres.\n");
            out.push_str("/// This allows functions to accept both connection pools and connections.\n");
            out.push_str("/// For transactions, use the deref pattern: &mut *transaction\n");
            out.push_str("pub trait PgExecutor<'c>: sqlx::Executor<'c, Database = sqlx::Postgres> + Send + Sync {}\n\n");
            out.push_str("impl<'c> PgExecutor<'c> for &'c sqlx::PgPool {}\n");
            out.push_str("impl<'c> PgExecutor<'c> for &'c mut sqlx::PgConnection {}\n");
        }
        ir::DatabaseBackend::Mysql => {
            out.push_str("use sqlx::{MySql, Pool, Transaction};\n\n");
            out.push_str("/// A trait abstracting over a MySqlPool or a MySql Transaction.\n");
            out.push_str("/// This allows generated helpers to be used in both standalone queries\n");
            out.push_str("/// and within larger database transactions.\n");
            out.push_str("pub trait MySqlExecutor<'a>: sqlx::Executor<'a, Database = MySql> {}\n\n");
            out.push_str("impl<'a> MySqlExecutor<'a> for &'a Pool<MySql> {}\n");
            out.push_str("impl<'a> MySqlExecutor<'a> for &'a mut Transaction<'_, MySql> {}\n");
        }
        ir::DatabaseBackend::Sqlite => {
            out.push_str("use sqlx::{Sqlite, Pool, Transaction};\n\n");
            out.push_str("/// A trait abstracting over a SqlitePool or a Sqlite Transaction.\n");
            out.push_str("/// This allows generated helpers to be used in both standalone queries\n");
            out.push_str("/// and within larger database transactions.\n");
            out.push_str("pub trait SqliteExecutor<'a>: sqlx::Executor<'a, Database = Sqlite> {}\n\n");
            out.push_str("impl<'a> SqliteExecutor<'a> for &'a Pool<Sqlite> {}\n");
            out.push_str("impl<'a> SqliteExecutor<'a> for &'a mut Transaction<'_, Sqlite> {}\n");
        }
    }
    
    out
}