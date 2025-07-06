use pg_embed::pg_enums::PgAuthMethod;
use pg_embed::pg_fetch::{PG_V14, PgFetchSettings};
use pg_embed::postgres::{PgEmbed, PgSettings};
use rustdbgen::{
    apply_type_aliases, generate_initial_migration, generate_migration, introspect_schema,
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

fn load_db_map() -> HashMap<String, String> {
    toml::from_str::<toml::Value>(&std::fs::read_to_string("type_map.toml").unwrap())
        .unwrap()
        .get("db_types")
        .cloned()
        .and_then(|v| v.try_into().ok())
        .unwrap_or_default()
}

#[tokio::test]
#[ignore]
async fn rename_column_round_trip() {
    sqlx::any::install_default_drivers();
    // configure temporary postgres
    let pg_settings = PgSettings {
        database_dir: std::env::temp_dir().join("pg_embed_test_rename"),
        port: 5434,
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

    // initial schema
    let mut base_ir: SchemaIR = load_schema("schema.model.toml").unwrap();
    let aliases = load_aliases();
    apply_type_aliases(&mut base_ir, &aliases);
    let (up_sql, _) = generate_initial_migration(&base_ir).unwrap();
    for stmt in up_sql.split(';') {
        let trimmed = stmt.trim();
        if !trimmed.is_empty() {
            sqlx::query(trimmed).execute(&pool).await.unwrap();
        }
    }

    let db_map = load_db_map();
    let old_ir = introspect_schema(&pg.db_uri, &db_map).await.unwrap();

    // new schema with renamed column
    let mut new_ir = base_ir.clone();
    {
        let user_model = new_ir.models.get_mut("User").unwrap();
        if let Some(mut field) = user_model.fields.remove("email") {
            field.rename_from = Some("email".into());
            user_model.fields.insert("contact_email".into(), field);
        }
        if let Some(idx) = user_model.indexes.get_mut("user_email_unique") {
            idx.fields = vec!["contact_email".into()];
        }
        if let Some(ck) = user_model.check_constraints.get_mut("user_email_not_empty") {
            ck.expression = "contact_email <> ''".into();
        }
    }

    let (up, down) = generate_migration(Some(&old_ir), &new_ir).unwrap();
    assert!(up.contains("RENAME COLUMN email TO contact_email"));
    assert!(down.contains("RENAME COLUMN contact_email TO email"));

    for stmt in up.split(';') {
        let trimmed = stmt.trim();
        if !trimmed.is_empty() {
            sqlx::query(trimmed).execute(&pool).await.unwrap();
        }
    }

    let new_db_ir = introspect_schema(&pg.db_uri, &db_map).await.unwrap();
    let (up2, down2) = generate_migration(Some(&new_db_ir), &new_ir).unwrap();
    assert!(up2.trim().is_empty());
    assert!(down2.trim().is_empty());

    pg.stop_db().await.unwrap();
}
