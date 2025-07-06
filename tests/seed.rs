use rustdbgen::{generate_seed_sql, ir::SchemaIR, load_schema};

#[test]
fn seed_sql_contains_inserts() {
    let ir: SchemaIR = load_schema("schema.model.toml").unwrap();
    let sql = generate_seed_sql(&ir);
    println!("{}", sql);
    assert!(sql.contains("INSERT INTO user"));
    assert!(sql.contains("admin@example.com"));
}
