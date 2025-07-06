use rustdbgen::{
    apply_type_aliases,
    ir::{FieldDef, FieldRef, Meta, ModelDef, RelationDef, SchemaIR, TypeAlias},
    lint_schema, load_schema,
};
use std::collections::HashMap;

#[test]
fn lint_passes_for_valid_schema() {
    let mut ir: SchemaIR = load_schema("schema.model.toml").unwrap();
    let alias_text = std::fs::read_to_string("type_map.toml").unwrap();
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
    let errors = lint_schema(&ir);
    assert_eq!(
        errors,
        vec!["Exclusion constraint post_author_excl on model Post uses a raw definition that cannot be fully validated".to_string()]
    );
}

#[test]
fn lint_fails_for_invalid_relation() {
    let mut fields = HashMap::new();
    fields.insert(
        "id".to_string(),
        FieldDef {
            rust_type: "Uuid".into(),
            db_type: Some("UUID".into()),
            default: None,
            nullable: false,
            tags: Vec::new(),
            zod: None,
            rename_from: None,
            storage: None,
        },
    );
    fields.insert(
        "author_id".to_string(),
        FieldDef {
            rust_type: "Uuid".into(),
            db_type: Some("UUID".into()),
            default: None,
            nullable: false,
            tags: Vec::new(),
            zod: None,
            rename_from: None,
            storage: None,
        },
    );
    let mut relations = HashMap::new();
    relations.insert(
        "bad_rel".to_string(),
        RelationDef {
            on: "author_id".into(),
            references: FieldRef { model: "Missing".into(), field: "id".into() },
        },
    );
    let mut models = HashMap::new();
    models.insert(
        "Post".to_string(),
        ModelDef { includes: Vec::new(), 
            fields,
            indexes: HashMap::new(),
            relations,
            unique_constraints: HashMap::new(),
            check_constraints: HashMap::new(),
            exclusion_constraints: HashMap::new(),
            permissions: Default::default(),
            options: Default::default(),
        },
    );
    let ir = SchemaIR { macros: HashMap::new(), 
        routes: HashMap::new(),
        plugins: HashMap::new(),
        seeds: HashMap::new(),
        schema_version: "1.0".into(),
        meta: Meta {
            observability_provider: None,
            rust_case_style: "camel".into(),
            db_case_style: "snake".into(),
            default_derives: vec![],
            db_backend: rustdbgen::ir::DatabaseBackend::Postgres,
            auth: Default::default(),
        },
        enums: HashMap::new(),
        models,
    };
    let errors = lint_schema(&ir);
    assert!(!errors.is_empty());
}
#[test]
fn lint_fails_for_unreciprocated_relation() {
    let mut post_fields = HashMap::new();
    post_fields.insert(
        "id".to_string(),
        FieldDef {
            rust_type: "Uuid".into(),
            db_type: Some("UUID".into()),
            default: None,
            nullable: false,
            tags: Vec::new(),
            zod: None,
            rename_from: None,
            storage: None,
        },
    );
    post_fields.insert(
        "user_id".to_string(),
        FieldDef {
            rust_type: "Uuid".into(),
            db_type: Some("UUID".into()),
            default: None,
            nullable: false,
            tags: Vec::new(),
            zod: None,
            rename_from: None,
            storage: None,
        },
    );
    let mut post_rel = HashMap::new();
    post_rel.insert(
        "author".to_string(),
        RelationDef {
            on: "user_id".into(),
            references: FieldRef { model: "User".into(), field: "id".into() },
        },
    );

    let post_model = ModelDef { includes: Vec::new(), 
        fields: post_fields,
        indexes: HashMap::new(),
        relations: post_rel,
        unique_constraints: HashMap::new(),
        check_constraints: HashMap::new(),
        exclusion_constraints: HashMap::new(),
        permissions: Default::default(),
        options: Default::default(),
    };

    let mut user_fields = HashMap::new();
    user_fields.insert(
        "id".to_string(),
        FieldDef {
            rust_type: "Uuid".into(),
            db_type: Some("UUID".into()),
            default: None,
            nullable: false,
            tags: Vec::new(),
            zod: None,
            rename_from: None,
            storage: None,
        },
    );
    let user_model = ModelDef { includes: Vec::new(), 
        fields: user_fields,
        indexes: HashMap::new(),
        relations: HashMap::new(),
        unique_constraints: HashMap::new(),
        check_constraints: HashMap::new(),
        exclusion_constraints: HashMap::new(),
        permissions: Default::default(),
        options: Default::default(),
    };

    let mut models = HashMap::new();
    models.insert("Post".to_string(), post_model);
    models.insert("User".to_string(), user_model);

    let ir = SchemaIR { macros: HashMap::new(), 
        routes: HashMap::new(),
        plugins: HashMap::new(),
        seeds: HashMap::new(),
        schema_version: "1.0".into(),
        meta: Meta::default(),
        enums: HashMap::new(),
        models,
    };

    let errors = lint_schema(&ir);
    assert!(
        errors.iter().any(|e| e.contains("not reciprocated")),
        "expected reciprocal relation lint"
    );
}

#[test]
fn lint_fails_for_duplicate_routes() {
    let mut routes = HashMap::new();
    routes.insert(
        "user".to_string(),
        rustdbgen::ir::RouteDef {
            methods: vec!["GET".into()],
            path: "/api/foo".into(),
            auth_required: false,
            permissions: Default::default(),
        },
    );
    routes.insert(
        "post".to_string(),
        rustdbgen::ir::RouteDef {
            methods: vec!["POST".into()],
            path: "/api/foo".into(),
            auth_required: false,
            permissions: Default::default(),
        },
    );

    let ir = SchemaIR { macros: HashMap::new(), 
        routes,
        plugins: HashMap::new(),
        seeds: HashMap::new(),
        schema_version: "1.0".into(),
        meta: Meta::default(),
        enums: HashMap::new(),
        models: HashMap::new(),
    };

    let errors = lint_schema(&ir);
    assert!(
        errors.iter().any(|e| e.contains("share the same path")),
        "expected duplicate route path lint"
    );
}

#[test]
fn lint_fails_for_invalid_route_method() {
    let mut routes = HashMap::new();
    routes.insert(
        "bad".to_string(),
        rustdbgen::ir::RouteDef {
            methods: vec!["FETCH".into()],
            path: "/api/bad".into(),
            auth_required: false,
            permissions: Default::default(),
        },
    );

    let ir = SchemaIR { macros: HashMap::new(), 
        routes,
        plugins: HashMap::new(),
        seeds: HashMap::new(),
        schema_version: "1.0".into(),
        meta: Meta::default(),
        enums: HashMap::new(),
        models: HashMap::new(),
    };

    let errors = lint_schema(&ir);
    assert!(errors.iter().any(|e| e.contains("unsupported HTTP method")));
}

#[test]
fn lint_fails_for_bad_check_expression() {
    let mut fields = HashMap::new();
    fields.insert(
        "id".to_string(),
        FieldDef {
            rust_type: "Uuid".into(),
            db_type: Some("UUID".into()),
            default: None,
            nullable: false,
            tags: Vec::new(),
            zod: None,
            rename_from: None,
            storage: None,
        },
    );
    let mut checks = HashMap::new();
    checks.insert(
        "bad_check".to_string(),
        rustdbgen::ir::CheckConstraintDef {
            expression: "missing_field > 0".into(),
        },
    );
    let mut models = HashMap::new();
    models.insert(
        "Thing".to_string(),
        ModelDef { includes: Vec::new(), 
            fields,
            indexes: HashMap::new(),
            relations: HashMap::new(),
            unique_constraints: HashMap::new(),
            check_constraints: checks,
            exclusion_constraints: HashMap::new(),
            permissions: Default::default(),
            options: Default::default(),
        },
    );
    let ir = SchemaIR { macros: HashMap::new(), 
        routes: HashMap::new(),
        plugins: HashMap::new(),
        seeds: HashMap::new(),
        schema_version: "1.0".into(),
        meta: Meta::default(),
        enums: HashMap::new(),
        models,
    };
    let errors = lint_schema(&ir);
    assert!(
        errors
            .iter()
            .any(|e| e.contains("references no known fields"))
    );
}

#[test]
fn lint_fails_for_unknown_role() {
    let mut fields = HashMap::new();
    fields.insert(
        "id".to_string(),
        FieldDef {
            rust_type: "Uuid".into(),
            db_type: Some("UUID".into()),
            default: None,
            nullable: false,
            tags: Vec::new(),
            zod: None,
            rename_from: None,
            storage: None,
        },
    );
    let mut models = HashMap::new();
    models.insert(
        "Thing".to_string(),
        ModelDef { includes: Vec::new(), 
            fields,
            indexes: HashMap::new(),
            relations: HashMap::new(),
            unique_constraints: HashMap::new(),
            check_constraints: HashMap::new(),
            exclusion_constraints: HashMap::new(),
            permissions: rustdbgen::ir::Permissions {
                read: vec!["unknown".into()],
                update: Vec::new(),
                delete: Vec::new(),
            },
            options: Default::default(),
        },
    );
    let mut enums = HashMap::new();
    enums.insert(
        "Role".to_string(),
        rustdbgen::ir::EnumDef {
            variants: vec!["admin".into()],
        },
    );
    let ir = SchemaIR { macros: HashMap::new(), 
        routes: HashMap::new(),
        plugins: HashMap::new(),
        seeds: HashMap::new(),
        schema_version: "1.0".into(),
        meta: Meta::default(),
        enums,
        models,
    };
    let errors = lint_schema(&ir);
    assert!(errors.iter().any(|e| e.contains("unknown role")));
}

#[test]
fn lint_allows_public_role() {
    let mut fields = HashMap::new();
    fields.insert(
        "id".to_string(),
        FieldDef {
            rust_type: "Uuid".into(),
            db_type: Some("UUID".into()),
            default: None,
            nullable: false,
            tags: Vec::new(),
            zod: None,
            rename_from: None,
            storage: None,
        },
    );
    let mut models = HashMap::new();
    models.insert(
        "Thing".to_string(),
        ModelDef { includes: Vec::new(), 
            fields,
            indexes: HashMap::new(),
            relations: HashMap::new(),
            unique_constraints: HashMap::new(),
            check_constraints: HashMap::new(),
            exclusion_constraints: HashMap::new(),
            permissions: rustdbgen::ir::Permissions {
                read: vec!["viewer".into()],
                update: Vec::new(),
                delete: Vec::new(),
            },
            options: Default::default(),
        },
    );
    let mut meta = Meta::default();
    meta.auth.public_role = "viewer".into();
    let ir = SchemaIR { macros: HashMap::new(), 
        routes: HashMap::new(),
        plugins: HashMap::new(),
        seeds: HashMap::new(),
        schema_version: "1.0".into(),
        meta,
        enums: HashMap::new(),
        models,
    };
    let errors = lint_schema(&ir);
    assert!(errors.is_empty());
}

#[test]
fn lint_fails_for_seed_unknown_model() {
    let mut seeds = HashMap::new();
    seeds.insert(
        "Missing".to_string(),
        rustdbgen::ir::SeedDef {
            rows: vec![HashMap::new()],
        },
    );
    let ir = SchemaIR { macros: HashMap::new(), 
        routes: HashMap::new(),
        plugins: HashMap::new(),
        seeds,
        schema_version: "1.0".into(),
        meta: Meta::default(),
        enums: HashMap::new(),
        models: HashMap::new(),
    };
    let errors = lint_schema(&ir);
    assert!(errors.iter().any(|e| e.contains("unknown model")));
}

#[test]
fn lint_fails_for_seed_unknown_field() {
    let mut fields = HashMap::new();
    fields.insert(
        "id".to_string(),
        FieldDef {
            rust_type: "Uuid".into(),
            db_type: Some("UUID".into()),
            default: None,
            nullable: false,
            rename_from: None,
            tags: Vec::new(),
            zod: None,
            storage: None,
        },
    );
    let model = ModelDef { includes: Vec::new(), 
        fields: fields.clone(),
        indexes: HashMap::new(),
        relations: HashMap::new(),
        unique_constraints: HashMap::new(),
        check_constraints: HashMap::new(),
        exclusion_constraints: HashMap::new(),
        permissions: Default::default(),
        options: Default::default(),
    };
    let mut models = HashMap::new();
    models.insert("User".to_string(), model);
    let mut row = HashMap::new();
    row.insert("bad_field".to_string(), toml::Value::String("val".into()));
    let mut seeds = HashMap::new();
    seeds.insert(
        "User".to_string(),
        rustdbgen::ir::SeedDef { rows: vec![row] },
    );
    let ir = SchemaIR { macros: HashMap::new(), 
        routes: HashMap::new(),
        plugins: HashMap::new(),
        seeds,
        schema_version: "1.0".into(),
        meta: Meta::default(),
        enums: HashMap::new(),
        models,
    };
    let errors = lint_schema(&ir);
    assert!(errors.iter().any(|e| e.contains("unknown field")));
}
#[test]
fn lint_warns_for_exclusion_definition() {
    use rustdbgen::ir::ExclusionConstraintDef;
    let mut fields = HashMap::new();
    fields.insert(
        "id".to_string(),
        FieldDef {
            rust_type: "Uuid".into(),
            db_type: Some("UUID".into()),
            default: None,
            nullable: false,
            rename_from: None,
            tags: Vec::new(),
            zod: None,
            storage: None,
        },
    );
    let mut exs = HashMap::new();
    exs.insert(
        "excl".to_string(),
        ExclusionConstraintDef {
            definition: "USING gist (id WITH =)".into(),
        },
    );
    let mut models = HashMap::new();
    models.insert(
        "Thing".to_string(),
        ModelDef { includes: Vec::new(), 
            fields,
            indexes: HashMap::new(),
            relations: HashMap::new(),
            unique_constraints: HashMap::new(),
            check_constraints: HashMap::new(),
            exclusion_constraints: exs,
            permissions: Default::default(),
            options: Default::default(),
        },
    );
    let ir = SchemaIR { macros: HashMap::new(), 
        routes: HashMap::new(),
        plugins: HashMap::new(),
        seeds: HashMap::new(),
        schema_version: "1.0".into(),
        meta: Meta::default(),
        enums: HashMap::new(),
        models,
    };
    let errors = lint_schema(&ir);
    assert!(
        errors.iter().any(|e| e.contains("raw definition")),
        "expected exclusion constraint lint"
    );
}
