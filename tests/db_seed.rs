use pg_embed::pg_enums::PgAuthMethod;
use pg_embed::pg_fetch::{PG_V14, PgFetchSettings};
use pg_embed::postgres::{PgEmbed, PgSettings};
use rustdbgen::{
    apply_macros, apply_model_options, apply_seed_data, apply_type_aliases, generate_initial_migration,
    ir::{SchemaIR, TypeAlias},
    load_schema,
};
use sqlx::AnyPool;
use std::collections::HashMap;
use std::fs;

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
async fn apply_seed_inserts_rows() {
    sqlx::any::install_default_drivers();
    let pg_settings = PgSettings {
        database_dir: std::env::temp_dir().join("pg_embed_test_seed"),
        port: 5440,
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
    for stmt in up_sql.split(';') {
        let trimmed = stmt.trim();
        if !trimmed.is_empty() {
            sqlx::query(trimmed).execute(&pool).await.unwrap();
        }
    }

    apply_seed_data(&pool, &ir).await.unwrap();

    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM \"user\"")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(count > 0);

    pg.stop_db().await.unwrap();
}
