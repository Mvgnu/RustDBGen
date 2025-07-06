use rustdbgen::{load_schema, apply_macros};

#[test]
fn macros_expand_fields() {
    let mut ir = load_schema("schema.model.toml").unwrap();
    apply_macros(&mut ir);
    let user = ir.models.get("User").unwrap();
    assert!(user.fields.contains_key("created_at"));
    assert!(user.fields.contains_key("updated_at"));
}
