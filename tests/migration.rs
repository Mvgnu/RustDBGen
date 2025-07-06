use rustdbgen::{
    apply_type_aliases, generate_initial_migration,
    ir::{SchemaIR, TypeAlias},
    load_schema,
};
use std::fs;

#[test]
fn initial_migration_contains_create_table() {
    let mut ir: SchemaIR = load_schema("schema.model.toml").unwrap();
    let alias_text = fs::read_to_string("type_map.toml").unwrap();
    let val: toml::Value = toml::from_str(&alias_text).unwrap();
    let aliases: std::collections::HashMap<String, TypeAlias> = val
        .as_table()
        .unwrap()
        .iter()
        .filter_map(|(k, v)| {
            if k == "db_types" {
                return None;
            }
            toml::Value::try_into(v.clone())
                .ok()
                .map(|a| (k.clone(), a))
        })
        .collect();
    apply_type_aliases(&mut ir, &aliases);
    let (up, down) = generate_initial_migration(&ir).unwrap();
    assert!(up.starts_with("BEGIN;"));
    assert!(up.trim_end().ends_with("COMMIT;"));
    assert!(down.starts_with("BEGIN;"));
    assert!(down.trim_end().ends_with("COMMIT;"));
    assert!(up.contains("CREATE TABLE user"));
    assert!(down.contains("DROP TABLE user"));
    assert!(up.contains("CREATE TYPE poststatus"));
    assert!(up.contains("ADD CONSTRAINT post_title_author_unique UNIQUE"));
    assert!(up.contains("ADD CONSTRAINT post_title_length CHECK"));
    assert!(up.contains("ADD CONSTRAINT post_author_excl EXCLUDE"));
}
