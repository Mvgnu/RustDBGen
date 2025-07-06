use rustdbgen::{
    apply_macros, apply_model_options, apply_type_aliases, generate_code,
    ir::{SchemaIR, TypeAlias},
    load_schema,
};
use std::fs;

#[test]
fn generate_includes_new_and_update_structs() {
    let ir: SchemaIR = load_schema("schema.model.toml").unwrap();
    let code = generate_code(&ir);
    assert!(code.contains("struct UserNew"));
    assert!(code.contains("struct UserUpdate"));
    assert!(code.contains("QueryBuilder"));
}

#[test]
fn enums_are_generated() {
    let ir: SchemaIR = load_schema("schema.model.toml").unwrap();
    let code = generate_code(&ir);
    assert!(code.contains("enum PostStatus"));
    assert!(code.contains("Draft"));
}

#[test]
fn model_options_add_fields() {
    let mut ir: SchemaIR = load_schema("schema.model.toml").unwrap();
    apply_macros(&mut ir);
    apply_model_options(&mut ir);
    assert!(ir.models["User"].fields.contains_key("created_at"));
    assert!(ir.models["User"].fields.contains_key("deleted_at"));
    assert!(ir.models["Post"].fields.contains_key("updated_at"));
}

#[test]
fn type_aliases_expand() {
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
    assert_eq!(ir.models["User"].fields["email"].rust_type, "String");
    assert_eq!(
        ir.models["User"].fields["email"].db_type.as_deref(),
        Some("TEXT")
    );
}

#[test]
fn storage_options_parse() {
    let ir: SchemaIR = load_schema("schema.model.toml").unwrap();
    let field = &ir.models["User"].fields["profile_pic"];
    let storage = field.storage.as_ref().expect("storage options");
    assert_eq!(storage.backend, "s3");
    assert_eq!(storage.allowed_types, ["image/png", "image/jpeg"]);
    assert_eq!(storage.path.as_deref(), Some("uploads/users/{id}/"));
}

#[test]
fn permissions_parse() {
    let ir: SchemaIR = load_schema("schema.model.toml").unwrap();
    let perms = &ir.models["User"].permissions;
    assert_eq!(perms.read, vec!["admin", "member"]);
    assert_eq!(perms.update, vec!["admin"]);
    assert_eq!(perms.delete, vec!["admin"]);
}

#[test]
fn routes_generate_constants() {
    let ir: SchemaIR = load_schema("schema.model.toml").unwrap();
    let code = generate_code(&ir);
    assert!(code.contains("pub mod routes"));
    assert!(code.contains("pub const USER"));
    assert!(code.contains("pub const POST"));
}

#[test]
fn route_permissions_parse() {
    let ir: SchemaIR = load_schema("schema.model.toml").unwrap();
    let route = &ir.routes["User"];
    assert_eq!(route.permissions.read, vec!["admin", "member"]);
    assert_eq!(route.permissions.update, vec!["admin"]);
    assert_eq!(route.permissions.delete, vec!["admin"]);
}

#[test]
fn typescript_generation() {
    let ir: SchemaIR = load_schema("schema.model.toml").unwrap();
    let ts = rustdbgen::generate_typescript(&ir);
    assert!(ts.contains("interface User"));
    assert!(ts.contains("routes"));
    assert!(ts.contains("UserSchema"));
    assert!(ts.contains("z.string().email()"));
    assert!(ts.contains("permissions"));
}

#[test]
fn graphql_generation() {
    let ir: SchemaIR = load_schema("schema.model.toml").unwrap();
    let gql = rustdbgen::generate_graphql_schema(&ir);
    assert!(gql.contains("type Post"));
    assert!(gql.contains("author: User"));
    assert!(gql.contains("type Query"));
    assert!(gql.contains("createUser"));
    assert!(gql.contains("updatePost"));
}

#[test]
fn tracing_instrumentation() {
    let ir: SchemaIR = load_schema("schema.model.toml").unwrap();
    let code = generate_code(&ir);
    assert!(code.contains("#[tracing::instrument]"));
}

#[test]
fn typescript_client_generation() {
    let ir: SchemaIR = load_schema("schema.model.toml").unwrap();
    let client = rustdbgen::generate_ts_client(&ir);
    assert!(client.contains("export const user"));
    assert!(client.contains("call(routes.user"));
}

#[test]
fn crud_sql_generation() {
    let ir: SchemaIR = load_schema("schema.model.toml").unwrap();
    let code = generate_code(&ir);
    assert!(code.contains("INSERT INTO user"));
    assert!(code.contains("DELETE FROM user"));
    assert!(code.contains("SELECT * FROM user WHERE id"));
}

#[test]
fn auth_helpers_generated() {
    let ir: SchemaIR = load_schema("schema.model.toml").unwrap();
    let code = generate_code(&ir);
    assert!(code.contains("ANONYMOUS_ROLE"));
    assert!(code.contains("PUBLIC_ROLE"));
    assert!(code.contains("has_permission"));
}

#[test]
fn meta_backend_default() {
    let ir: SchemaIR = load_schema("schema.model.toml").unwrap();
    assert!(matches!(
        ir.meta.db_backend,
        rustdbgen::ir::DatabaseBackend::Postgres
    ));
}

#[test]
fn auth_provider_enum_parses() {
    let ir: SchemaIR = load_schema("schema.model.toml").unwrap();
    assert!(matches!(ir.meta.auth.provider, rustdbgen::ir::AuthProvider::None));
    assert_eq!(ir.meta.auth.role_claim, "role");
}

#[test]
fn custom_public_role_constant() {
    let mut ir: SchemaIR = load_schema("schema.model.toml").unwrap();
    ir.meta.auth.public_role = "visitor".into();
    let code = generate_code(&ir);
    assert!(code.contains("pub const PUBLIC_ROLE: &str = \"visitor\""));
}

#[test]
fn typed_error_enum_generated() {
    let ir: SchemaIR = load_schema("schema.model.toml").unwrap();
    let code = generate_code(&ir);
    assert!(code.contains("enum UserCreateError"));
    assert!(code.contains("PostsFk"));
    assert!(code.contains("enum PostCreateError"));
    assert!(code.contains("PostTitleAuthorUnique"));
    assert!(code.contains("AuthorFk"));
}

#[test]
fn pagination_list_generated() {
    let ir: SchemaIR = load_schema("schema.model.toml").unwrap();
    let code = generate_code(&ir);
    assert!(code.contains("struct Pagination"));
    assert!(code.contains("pagination: Option<Pagination>"));
}
