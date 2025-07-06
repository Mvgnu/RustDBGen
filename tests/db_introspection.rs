use pg_embed::pg_enums::PgAuthMethod;
use pg_embed::pg_fetch::{PG_V14, PgFetchSettings};
use pg_embed::postgres::{PgEmbed, PgSettings};
use rustdbgen::{
    apply_type_aliases, generate_initial_migration, generate_migration, introspect_schema,
    ir::{SchemaIR, TypeAlias},
    load_schema,
};
use sqlx::AnyPool;
use tempfile::NamedTempFile;
use std::collections::HashMap;
use std::fs;

fn load_db_map() -> HashMap<String, String> {
    toml::from_str::<toml::Value>(&std::fs::read_to_string("type_map.toml").unwrap())
        .unwrap()
        .get("db_types")
        .cloned()
        .and_then(|v| v.try_into().ok())
        .unwrap_or_default()
}

fn load_aliases() -> HashMap<String, TypeAlias> {
    toml::from_str::<toml::Value>(&fs::read_to_string("type_map.toml").unwrap())
        .unwrap()
        .as_table()
        .unwrap()
        .iter()
        .filter_map(|(k, v)| {
            if k == "db_types" {
                None
            } else {
                toml::Value::try_into(v.clone()).ok().map(|a| (k.clone(), a))
            }
        })
        .collect()
}

#[tokio::test]
#[ignore]
async fn introspect_round_trip() {
    sqlx::any::install_default_drivers();
    // configure temporary postgres
    let pg_settings = PgSettings {
        database_dir: std::env::temp_dir().join("pg_embed_test"),
        port: 5433,
        user: "postgres".into(),
        password: "password".into(),
        auth_method: PgAuthMethod::Plain,
        persistent: false,
        timeout: Some(std::time::Duration::from_secs(15)),
        migration_dir: None,
    };
    let fetch_settings = PgFetchSettings {
        version: PG_V14,
        ..Default::default()
    };
    let mut pg = PgEmbed::new(pg_settings, fetch_settings).await.unwrap();
    pg.setup().await.unwrap();
    pg.start_db().await.unwrap();

    let pool = AnyPool::connect(&pg.db_uri).await.unwrap();

    // create schema in db
    let mut ir: SchemaIR = load_schema("schema.model.toml").unwrap();
    let aliases = load_aliases();
    apply_type_aliases(&mut ir, &aliases);
    let (up_sql, _) = generate_initial_migration(&ir).unwrap();
    for stmt in up_sql.split(';') {
        let trimmed = stmt.trim();
        if !trimmed.is_empty() {
            sqlx::query(trimmed).execute(&pool).await.unwrap();
        }
    }

    let db_map = load_db_map();
    let introspected = introspect_schema(&pg.db_uri, &db_map).await.unwrap();
    let (up, down) = generate_migration(Some(&introspected), &ir).unwrap();
    assert!(up.trim().is_empty());
    assert!(down.trim().is_empty());

    pg.stop_db().await.unwrap();
}

#[tokio::test]
#[ignore]
async fn sqlite_introspection_round_trip() {
    sqlx::any::install_default_drivers();
    let tmp = NamedTempFile::new().unwrap();
    let url = format!("sqlite://{}", tmp.path().to_string_lossy());
    let pool = AnyPool::connect(&url).await.unwrap();

    let mut ir: SchemaIR = load_schema("schema.model.toml").unwrap();
    let aliases = load_aliases();
    apply_type_aliases(&mut ir, &aliases);
    let (up_sql, _) = generate_initial_migration(&ir).unwrap();
    for stmt in up_sql.split(';') {
        let trimmed = stmt.trim();
        if !trimmed.is_empty() {
            sqlx::query(trimmed).execute(&pool).await.unwrap();
        }
    }

    let db_map = load_db_map();
    let introspected = introspect_schema(&url, &db_map).await.unwrap();
    let (up, down) = generate_migration(Some(&introspected), &ir).unwrap();
    assert!(up.trim().is_empty());
    assert!(down.trim().is_empty());
}
