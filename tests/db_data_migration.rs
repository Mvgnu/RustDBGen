use pg_embed::pg_enums::PgAuthMethod;
use pg_embed::pg_fetch::{PG_V14, PgFetchSettings};
use pg_embed::postgres::{PgEmbed, PgSettings};
use rustdbgen::{
    apply_macros, apply_migrations, apply_model_options, apply_type_aliases, generate_initial_migration,
    ir::{SchemaIR, TypeAlias},
    load_schema,
};
use sqlx::AnyPool;
use std::collections::HashMap;
use std::fs;
use tempfile::tempdir;

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
async fn data_migration_runs_sql() {
    sqlx::any::install_default_drivers();
    let pg_settings = PgSettings {
        database_dir: std::env::temp_dir().join("pg_embed_test_data"),
        port: 5460,
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

    let mut ir: SchemaIR = load_schema("schema.model.toml").unwrap();
    apply_macros(&mut ir);
    let aliases = load_aliases();
    apply_type_aliases(&mut ir, &aliases);
    apply_model_options(&mut ir);
    let (up_sql, _) = generate_initial_migration(&ir).unwrap();

    let dir = tempdir().unwrap();
    fs::write(dir.path().join("0001_init.up.sql"), up_sql).unwrap();
    fs::write(
        dir.path().join("0002_seed_admin.data.sql"),
        "INSERT INTO \"user\" (id, email) VALUES ('00000000-0000-0000-0000-000000000001', 'admin@example.com');",
    )
    .unwrap();

    apply_migrations(&pool, dir.path().to_str().unwrap())
        .await
        .unwrap();

    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM \"user\"")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1);

    pg.stop_db().await.unwrap();
}
