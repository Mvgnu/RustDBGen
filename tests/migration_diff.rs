use rustdbgen::{
    generate_migration,
    ir::{
        CheckConstraintDef, ExclusionConstraintDef, FieldDef, FieldRef, IndexDef, Meta, ModelDef,
        RelationDef, SchemaIR, UniqueConstraintDef,
    },
};
use std::collections::HashMap;

#[test]
fn diff_migration_adds_column() {
    // old schema with only id
    let mut old_models = HashMap::new();
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
    old_models.insert(
        "User".to_string(),
        ModelDef { includes: Vec::new(), 
            fields: user_fields.clone(),
            indexes: HashMap::new(),
            relations: HashMap::new(),
            unique_constraints: HashMap::new(),
            check_constraints: HashMap::new(),
            exclusion_constraints: HashMap::new(),
            permissions: Default::default(),
            options: Default::default(),
        },
    );
    let old = SchemaIR { macros: HashMap::new(), 
        routes: HashMap::new(),
        schema_version: "1.0".into(),
        meta: Meta {
            auth: Default::default(),
            observability_provider: None,
            rust_case_style: "camel".into(),
            db_case_style: "snake".into(),
            default_derives: vec![],
            db_backend: rustdbgen::ir::DatabaseBackend::Postgres,
        },
        enums: HashMap::new(),
        models: old_models,
        plugins: HashMap::new(),
        seeds: HashMap::new(),
    };

    // new schema with id and name
    let mut new_fields = user_fields.clone();
    new_fields.insert(
        "name".to_string(),
        FieldDef {
            rust_type: "String".into(),
            db_type: Some("TEXT".into()),
            default: None,
            nullable: false,
            tags: Vec::new(),
            zod: None,
            rename_from: None,
            storage: None,
        },
    );
    let mut new_models = HashMap::new();
    new_models.insert(
        "User".to_string(),
        ModelDef { includes: Vec::new(), 
            fields: new_fields,
            indexes: HashMap::new(),
            relations: HashMap::new(),
            unique_constraints: HashMap::new(),
            check_constraints: HashMap::new(),
            exclusion_constraints: HashMap::new(),
            permissions: Default::default(),
            options: Default::default(),
        },
    );
    let new_ir = SchemaIR { macros: HashMap::new(), 
        routes: HashMap::new(),
        schema_version: "1.0".into(),
        meta: Meta {
            auth: Default::default(),
            observability_provider: None,
            rust_case_style: "camel".into(),
            db_case_style: "snake".into(),
            default_derives: vec![],
            db_backend: rustdbgen::ir::DatabaseBackend::Postgres,
        },
        enums: HashMap::new(),
        models: new_models,
        plugins: HashMap::new(),
        seeds: HashMap::new(),
    };

    let (up, down) = generate_migration(Some(&old), &new_ir).unwrap();
    assert!(up.contains("ADD COLUMN name TEXT"));
    assert!(down.contains("DROP COLUMN name"));
}

#[test]
fn diff_migration_changes_column_type() {
    // old schema with name TEXT
    let mut old_fields = HashMap::new();
    old_fields.insert(
        "name".to_string(),
        FieldDef {
            rust_type: "String".into(),
            db_type: Some("TEXT".into()),
            default: None,
            nullable: false,
            tags: Vec::new(),
            zod: None,
            rename_from: None,
            storage: None,
        },
    );
    let mut old_models = HashMap::new();
    old_models.insert(
        "User".to_string(),
        ModelDef { includes: Vec::new(), 
            fields: old_fields,
            indexes: HashMap::new(),
            relations: HashMap::new(),
            unique_constraints: HashMap::new(),
            check_constraints: HashMap::new(),
            exclusion_constraints: HashMap::new(),
            permissions: Default::default(),
            options: Default::default(),
        },
    );
    let old = SchemaIR { macros: HashMap::new(), 
        routes: HashMap::new(),
        schema_version: "1.0".into(),
        meta: Meta {
            auth: Default::default(),
            observability_provider: None,
            rust_case_style: "camel".into(),
            db_case_style: "snake".into(),
            default_derives: vec![],
            db_backend: rustdbgen::ir::DatabaseBackend::Postgres,
        },
        enums: HashMap::new(),
        models: old_models,
        plugins: HashMap::new(),
        seeds: HashMap::new(),
    };

    // new schema with name VARCHAR(255)
    let mut new_fields = HashMap::new();
    new_fields.insert(
        "name".to_string(),
        FieldDef {
            rust_type: "String".into(),
            db_type: Some("VARCHAR(255)".into()),
            default: None,
            nullable: false,
            tags: Vec::new(),
            zod: None,
            rename_from: None,
            storage: None,
        },
    );
    let mut new_models = HashMap::new();
    new_models.insert(
        "User".to_string(),
        ModelDef { includes: Vec::new(), 
            fields: new_fields,
            indexes: HashMap::new(),
            relations: HashMap::new(),
            unique_constraints: HashMap::new(),
            check_constraints: HashMap::new(),
            exclusion_constraints: HashMap::new(),
            permissions: Default::default(),
            options: Default::default(),
        },
    );
    let new = SchemaIR { macros: HashMap::new(), 
        routes: HashMap::new(),
        schema_version: "1.0".into(),
        meta: Meta {
            auth: Default::default(),
            observability_provider: None,
            rust_case_style: "camel".into(),
            db_case_style: "snake".into(),
            default_derives: vec![],
            db_backend: rustdbgen::ir::DatabaseBackend::Postgres,
        },
        enums: HashMap::new(),
        models: new_models,
        plugins: HashMap::new(),
        seeds: HashMap::new(),
    };

    let (up, down) = generate_migration(Some(&old), &new).unwrap();
    assert!(up.contains("ALTER COLUMN name TYPE VARCHAR(255)"));
    assert!(down.contains("ALTER COLUMN name TYPE TEXT"));
}

#[test]
fn diff_migration_changes_column_default() {
    // old schema with default 1
    let mut old_fields = HashMap::new();
    old_fields.insert(
        "count".to_string(),
        FieldDef {
            rust_type: "i32".into(),
            db_type: Some("INTEGER".into()),
            default: Some("1".into()),
            nullable: false,
            tags: Vec::new(),
            zod: None,
            rename_from: None,
            storage: None,
        },
    );
    let mut old_models = HashMap::new();
    old_models.insert(
        "Item".to_string(),
        ModelDef { includes: Vec::new(), 
            fields: old_fields,
            indexes: HashMap::new(),
            relations: HashMap::new(),
            unique_constraints: HashMap::new(),
            check_constraints: HashMap::new(),
            exclusion_constraints: HashMap::new(),
            permissions: Default::default(),
            options: Default::default(),
        },
    );
    let old = SchemaIR { macros: HashMap::new(), 
        routes: HashMap::new(),
        schema_version: "1.0".into(),
        meta: Meta {
            auth: Default::default(),
            observability_provider: None,
            rust_case_style: "camel".into(),
            db_case_style: "snake".into(),
            default_derives: vec![],
            db_backend: rustdbgen::ir::DatabaseBackend::Postgres,
        },
        enums: HashMap::new(),
        models: old_models,
        plugins: HashMap::new(),
        seeds: HashMap::new(),
    };

    // new schema with default 2
    let mut new_fields = HashMap::new();
    new_fields.insert(
        "count".to_string(),
        FieldDef {
            rust_type: "i32".into(),
            db_type: Some("INTEGER".into()),
            default: Some("2".into()),
            nullable: false,
            tags: Vec::new(),
            zod: None,
            rename_from: None,
            storage: None,
        },
    );
    let mut new_models = HashMap::new();
    new_models.insert(
        "Item".to_string(),
        ModelDef { includes: Vec::new(), 
            fields: new_fields,
            indexes: HashMap::new(),
            relations: HashMap::new(),
            unique_constraints: HashMap::new(),
            check_constraints: HashMap::new(),
            exclusion_constraints: HashMap::new(),
            permissions: Default::default(),
            options: Default::default(),
        },
    );
    let new = SchemaIR { macros: HashMap::new(), 
        routes: HashMap::new(),
        schema_version: "1.0".into(),
        meta: Meta {
            auth: Default::default(),
            observability_provider: None,
            rust_case_style: "camel".into(),
            db_case_style: "snake".into(),
            default_derives: vec![],
            db_backend: rustdbgen::ir::DatabaseBackend::Postgres,
        },
        enums: HashMap::new(),
        models: new_models,
        plugins: HashMap::new(),
        seeds: HashMap::new(),
    };

    let (up, down) = generate_migration(Some(&old), &new).unwrap();
    assert!(up.contains("SET DEFAULT 2"));
    assert!(down.contains("SET DEFAULT 1"));
}

#[test]
fn diff_migration_changes_column_nullability() {
    // old schema nullable = false
    let mut old_fields = HashMap::new();
    old_fields.insert(
        "title".to_string(),
        FieldDef {
            rust_type: "String".into(),
            db_type: Some("TEXT".into()),
            default: None,
            nullable: false,
            tags: Vec::new(),
            zod: None,
            rename_from: None,
            storage: None,
        },
    );
    let mut old_models = HashMap::new();
    old_models.insert(
        "Post".to_string(),
        ModelDef { includes: Vec::new(), 
            fields: old_fields,
            indexes: HashMap::new(),
            relations: HashMap::new(),
            unique_constraints: HashMap::new(),
            check_constraints: HashMap::new(),
            exclusion_constraints: HashMap::new(),
            permissions: Default::default(),
            options: Default::default(),
        },
    );
    let old = SchemaIR { macros: HashMap::new(), 
        routes: HashMap::new(),
        schema_version: "1.0".into(),
        meta: Meta {
            auth: Default::default(),
            observability_provider: None,
            rust_case_style: "camel".into(),
            db_case_style: "snake".into(),
            default_derives: vec![],
            db_backend: rustdbgen::ir::DatabaseBackend::Postgres,
        },
        enums: HashMap::new(),
        models: old_models,
        plugins: HashMap::new(),
        seeds: HashMap::new(),
    };

    // new schema nullable = true
    let mut new_fields = HashMap::new();
    new_fields.insert(
        "title".to_string(),
        FieldDef {
            rust_type: "String".into(),
            db_type: Some("TEXT".into()),
            default: None,
            nullable: true,
            tags: Vec::new(),
            zod: None,
            rename_from: None,
            storage: None,
        },
    );
    let mut new_models = HashMap::new();
    new_models.insert(
        "Post".to_string(),
        ModelDef { includes: Vec::new(), 
            fields: new_fields,
            indexes: HashMap::new(),
            relations: HashMap::new(),
            unique_constraints: HashMap::new(),
            check_constraints: HashMap::new(),
            exclusion_constraints: HashMap::new(),
            permissions: Default::default(),
            options: Default::default(),
        },
    );
    let new = SchemaIR { macros: HashMap::new(), 
        routes: HashMap::new(),
        schema_version: "1.0".into(),
        meta: Meta {
            auth: Default::default(),
            observability_provider: None,
            rust_case_style: "camel".into(),
            db_case_style: "snake".into(),
            default_derives: vec![],
            db_backend: rustdbgen::ir::DatabaseBackend::Postgres,
        },
        enums: HashMap::new(),
        models: new_models,
        plugins: HashMap::new(),
        seeds: HashMap::new(),
    };

    let (up, down) = generate_migration(Some(&old), &new).unwrap();
    assert!(up.contains("DROP NOT NULL"));
    assert!(down.contains("SET NOT NULL"));
}

#[test]
fn diff_migration_adds_index() {
    let mut old_fields = HashMap::new();
    old_fields.insert(
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
    let mut old_models = HashMap::new();
    old_models.insert(
        "Post".to_string(),
        ModelDef { includes: Vec::new(), 
            fields: old_fields.clone(),
            indexes: HashMap::new(),
            relations: HashMap::new(),
            unique_constraints: HashMap::new(),
            check_constraints: HashMap::new(),
            exclusion_constraints: HashMap::new(),
            permissions: Default::default(),
            options: Default::default(),
        },
    );
    let old = SchemaIR { macros: HashMap::new(), 
        routes: HashMap::new(),
        schema_version: "1.0".into(),
        meta: Meta {
            auth: Default::default(),
            observability_provider: None,
            rust_case_style: "camel".into(),
            db_case_style: "snake".into(),
            default_derives: vec![],
            db_backend: rustdbgen::ir::DatabaseBackend::Postgres,
        },
        enums: HashMap::new(),
        models: old_models,
        plugins: HashMap::new(),
        seeds: HashMap::new(),
    };

    // new schema with index
    let mut new_models = HashMap::new();
    let mut indexes = HashMap::new();
    indexes.insert(
        "post_author_idx".to_string(),
        IndexDef {
            fields: vec!["author_id".into()],
            unique: false,
        },
    );
    new_models.insert(
        "Post".to_string(),
        ModelDef { includes: Vec::new(), 
            fields: old_fields,
            indexes,
            relations: HashMap::new(),
            unique_constraints: HashMap::new(),
            check_constraints: HashMap::new(),
            exclusion_constraints: HashMap::new(),
            permissions: Default::default(),
            options: Default::default(),
        },
    );
    let new = SchemaIR { macros: HashMap::new(), 
        routes: HashMap::new(),
        schema_version: "1.0".into(),
        meta: Meta {
            auth: Default::default(),
            observability_provider: None,
            rust_case_style: "camel".into(),
            db_case_style: "snake".into(),
            default_derives: vec![],
            db_backend: rustdbgen::ir::DatabaseBackend::Postgres,
        },
        enums: HashMap::new(),
        models: new_models,
        plugins: HashMap::new(),
        seeds: HashMap::new(),
    };

    let (up, down) = generate_migration(Some(&old), &new).unwrap();
    assert!(up.contains("CREATE INDEX post_author_idx"));
    assert!(down.contains("DROP INDEX post_author_idx"));
}

#[test]
fn diff_migration_changes_index_uniqueness() {
    let mut fields = HashMap::new();
    fields.insert(
        "email".to_string(),
        FieldDef {
            rust_type: "String".into(),
            db_type: Some("TEXT".into()),
            default: None,
            nullable: false,
            tags: Vec::new(),
            zod: None,
            rename_from: None,
            storage: None,
        },
    );

    // old model with non-unique index
    let mut old_indexes = HashMap::new();
    old_indexes.insert(
        "user_email_idx".to_string(),
        IndexDef {
            fields: vec!["email".into()],
            unique: false,
        },
    );
    let mut old_models = HashMap::new();
    old_models.insert(
        "User".to_string(),
        ModelDef { includes: Vec::new(), 
            fields: fields.clone(),
            indexes: old_indexes,
            relations: HashMap::new(),
            unique_constraints: HashMap::new(),
            check_constraints: HashMap::new(),
            exclusion_constraints: HashMap::new(),
            permissions: Default::default(),
            options: Default::default(),
        },
    );
    let old = SchemaIR { macros: HashMap::new(), 
        routes: HashMap::new(),
        schema_version: "1.0".into(),
        meta: Meta {
            auth: Default::default(),
            observability_provider: None,
            rust_case_style: "camel".into(),
            db_case_style: "snake".into(),
            default_derives: vec![],
            db_backend: rustdbgen::ir::DatabaseBackend::Postgres,
        },
        enums: HashMap::new(),
        models: old_models,
        plugins: HashMap::new(),
        seeds: HashMap::new(),
    };

    // new model with unique index
    let mut new_indexes = HashMap::new();
    new_indexes.insert(
        "user_email_idx".to_string(),
        IndexDef {
            fields: vec!["email".into()],
            unique: true,
        },
    );
    let mut new_models = HashMap::new();
    new_models.insert(
        "User".to_string(),
        ModelDef { includes: Vec::new(), 
            fields,
            indexes: new_indexes,
            relations: HashMap::new(),
            unique_constraints: HashMap::new(),
            check_constraints: HashMap::new(),
            exclusion_constraints: HashMap::new(),
            permissions: Default::default(),
            options: Default::default(),
        },
    );
    let new = SchemaIR { macros: HashMap::new(), 
        routes: HashMap::new(),
        schema_version: "1.0".into(),
        meta: Meta {
            auth: Default::default(),
            observability_provider: None,
            rust_case_style: "camel".into(),
            db_case_style: "snake".into(),
            default_derives: vec![],
            db_backend: rustdbgen::ir::DatabaseBackend::Postgres,
        },
        enums: HashMap::new(),
        models: new_models,
        plugins: HashMap::new(),
        seeds: HashMap::new(),
    };

    let (up, down) = generate_migration(Some(&old), &new).unwrap();
    assert!(up.contains("DROP INDEX user_email_idx"));
    assert!(up.contains("CREATE UNIQUE INDEX user_email_idx"));
    assert!(down.contains("DROP INDEX user_email_idx"));
    assert!(down.contains("CREATE INDEX user_email_idx"));
}

#[test]
fn diff_migration_adds_unique_constraint() {
    let mut fields = HashMap::new();
    fields.insert(
        "title".to_string(),
        FieldDef {
            rust_type: "String".into(),
            db_type: Some("TEXT".into()),
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

    let mut old_models = HashMap::new();
    old_models.insert(
        "Post".to_string(),
        ModelDef { includes: Vec::new(), 
            fields: fields.clone(),
            indexes: HashMap::new(),
            relations: HashMap::new(),
            unique_constraints: HashMap::new(),
            check_constraints: HashMap::new(),
            exclusion_constraints: HashMap::new(),
            permissions: Default::default(),
            options: Default::default(),
        },
    );
    let old = SchemaIR { macros: HashMap::new(), 
        routes: HashMap::new(),
        schema_version: "1.0".into(),
        meta: Meta {
            auth: Default::default(),
            observability_provider: None,
            rust_case_style: "camel".into(),
            db_case_style: "snake".into(),
            default_derives: vec![],
            db_backend: rustdbgen::ir::DatabaseBackend::Postgres,
        },
        enums: HashMap::new(),
        models: old_models,
        plugins: HashMap::new(),
        seeds: HashMap::new(),
    };

    let mut new_models = HashMap::new();
    let mut ucs = HashMap::new();
    ucs.insert(
        "post_title_author_unique".to_string(),
        UniqueConstraintDef {
            fields: vec!["title".into(), "author_id".into()],
        },
    );
    new_models.insert(
        "Post".to_string(),
        ModelDef { includes: Vec::new(), 
            fields: fields,
            indexes: HashMap::new(),
            relations: HashMap::new(),
            unique_constraints: ucs,
            check_constraints: HashMap::new(),
            exclusion_constraints: HashMap::new(),
            permissions: Default::default(),
            options: Default::default(),
        },
    );
    let new = SchemaIR { macros: HashMap::new(), 
        routes: HashMap::new(),
        schema_version: "1.0".into(),
        meta: Meta {
            auth: Default::default(),
            observability_provider: None,
            rust_case_style: "camel".into(),
            db_case_style: "snake".into(),
            default_derives: vec![],
            db_backend: rustdbgen::ir::DatabaseBackend::Postgres,
        },
        enums: HashMap::new(),
        models: new_models,
        plugins: HashMap::new(),
        seeds: HashMap::new(),
    };

    let (up, down) = generate_migration(Some(&old), &new).unwrap();
    assert!(up.contains("ADD CONSTRAINT post_title_author_unique UNIQUE"));
    assert!(down.contains("DROP CONSTRAINT post_title_author_unique"));
}

#[test]
fn diff_migration_changes_unique_constraint_fields() {
    let mut fields = HashMap::new();
    fields.insert(
        "title".to_string(),
        FieldDef {
            rust_type: "String".into(),
            db_type: Some("TEXT".into()),
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

    let mut old_uc = HashMap::new();
    old_uc.insert(
        "post_uc".to_string(),
        UniqueConstraintDef {
            fields: vec!["title".into(), "author_id".into()],
        },
    );
    let mut old_models = HashMap::new();
    old_models.insert(
        "Post".to_string(),
        ModelDef { includes: Vec::new(), 
            fields: fields.clone(),
            indexes: HashMap::new(),
            relations: HashMap::new(),
            unique_constraints: old_uc,
            check_constraints: HashMap::new(),
            exclusion_constraints: HashMap::new(),
            permissions: Default::default(),
            options: Default::default(),
        },
    );
    let old = SchemaIR { macros: HashMap::new(), 
        routes: HashMap::new(),
        schema_version: "1.0".into(),
        meta: Meta {
            auth: Default::default(),
            observability_provider: None,
            rust_case_style: "camel".into(),
            db_case_style: "snake".into(),
            default_derives: vec![],
            db_backend: rustdbgen::ir::DatabaseBackend::Postgres,
        },
        enums: HashMap::new(),
        models: old_models,
        plugins: HashMap::new(),
        seeds: HashMap::new(),
    };

    let mut new_uc = HashMap::new();
    new_uc.insert(
        "post_uc".to_string(),
        UniqueConstraintDef {
            fields: vec!["title".into()],
        },
    );
    let mut new_models = HashMap::new();
    new_models.insert(
        "Post".to_string(),
        ModelDef { includes: Vec::new(), 
            fields: fields,
            indexes: HashMap::new(),
            relations: HashMap::new(),
            unique_constraints: new_uc,
            check_constraints: HashMap::new(),
            exclusion_constraints: HashMap::new(),
            permissions: Default::default(),
            options: Default::default(),
        },
    );
    let new = SchemaIR { macros: HashMap::new(), 
        routes: HashMap::new(),
        schema_version: "1.0".into(),
        meta: Meta {
            auth: Default::default(),
            observability_provider: None,
            rust_case_style: "camel".into(),
            db_case_style: "snake".into(),
            default_derives: vec![],
            db_backend: rustdbgen::ir::DatabaseBackend::Postgres,
        },
        enums: HashMap::new(),
        models: new_models,
        plugins: HashMap::new(),
        seeds: HashMap::new(),
    };

    let (up, down) = generate_migration(Some(&old), &new).unwrap();
    assert!(up.contains("DROP CONSTRAINT post_uc"));
    assert!(up.contains("ADD CONSTRAINT post_uc UNIQUE (title)"));
    assert!(down.contains("ADD CONSTRAINT post_uc UNIQUE (title, author_id)"));
}

#[test]
fn diff_migration_adds_foreign_key() {
    let mut old_fields = HashMap::new();
    old_fields.insert(
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
    let mut old_models = HashMap::new();
    old_models.insert(
        "Post".to_string(),
        ModelDef { includes: Vec::new(), 
            fields: old_fields.clone(),
            indexes: HashMap::new(),
            relations: HashMap::new(),
            unique_constraints: HashMap::new(),
            check_constraints: HashMap::new(),
            exclusion_constraints: HashMap::new(),
            permissions: Default::default(),
            options: Default::default(),
        },
    );
    let old = SchemaIR { macros: HashMap::new(), 
        routes: HashMap::new(),
        schema_version: "1.0".into(),
        meta: Meta {
            auth: Default::default(),
            observability_provider: None,
            rust_case_style: "camel".into(),
            db_case_style: "snake".into(),
            default_derives: vec![],
            db_backend: rustdbgen::ir::DatabaseBackend::Postgres,
        },
        enums: HashMap::new(),
        models: old_models,
        plugins: HashMap::new(),
        seeds: HashMap::new(),
    };

    // new schema with relation
    let mut new_models = HashMap::new();
    let mut relations = HashMap::new();
    relations.insert(
        "post_author_fk".to_string(),
        RelationDef {
            on: "author_id".into(),
            references: FieldRef { model: "User".into(), field: "id".into() },
        },
    );
    new_models.insert(
        "Post".to_string(),
        ModelDef { includes: Vec::new(), 
            fields: old_fields,
            indexes: HashMap::new(),
            relations,
            unique_constraints: HashMap::new(),
            check_constraints: HashMap::new(),
            exclusion_constraints: HashMap::new(),
            permissions: Default::default(),
            options: Default::default(),
        },
    );
    let new = SchemaIR { macros: HashMap::new(), 
        routes: HashMap::new(),
        schema_version: "1.0".into(),
        meta: Meta {
            auth: Default::default(),
            observability_provider: None,
            rust_case_style: "camel".into(),
            db_case_style: "snake".into(),
            default_derives: vec![],
            db_backend: rustdbgen::ir::DatabaseBackend::Postgres,
        },
        enums: HashMap::new(),
        models: new_models,
        plugins: HashMap::new(),
        seeds: HashMap::new(),
    };

    let (up, down) = generate_migration(Some(&old), &new).unwrap();
    assert!(up.contains("ADD CONSTRAINT post_author_fk"));
    assert!(down.contains("DROP CONSTRAINT post_author_fk"));
}

#[test]
fn diff_migration_changes_foreign_key() {
    let mut fields = HashMap::new();
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

    // old schema with relation to User
    let mut relations_old = HashMap::new();
    relations_old.insert(
        "post_author_fk".to_string(),
        RelationDef {
            on: "author_id".into(),
            references: FieldRef { model: "User".into(), field: "id".into() },
        },
    );

    let mut old_models = HashMap::new();
    old_models.insert(
        "User".to_string(),
        ModelDef { includes: Vec::new(), 
            fields: {
                let mut f = HashMap::new();
                f.insert(
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
                f
            },
            indexes: HashMap::new(),
            relations: HashMap::new(),
            unique_constraints: HashMap::new(),
            check_constraints: HashMap::new(),
            exclusion_constraints: HashMap::new(),
            permissions: Default::default(),
            options: Default::default(),
        },
    );
    old_models.insert(
        "Account".to_string(),
        ModelDef { includes: Vec::new(), 
            fields: {
                let mut f = HashMap::new();
                f.insert(
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
                f
            },
            indexes: HashMap::new(),
            relations: HashMap::new(),
            unique_constraints: HashMap::new(),
            check_constraints: HashMap::new(),
            exclusion_constraints: HashMap::new(),
            permissions: Default::default(),
            options: Default::default(),
        },
    );
    old_models.insert(
        "Post".to_string(),
        ModelDef { includes: Vec::new(), 
            fields: fields.clone(),
            indexes: HashMap::new(),
            relations: relations_old,
            unique_constraints: HashMap::new(),
            check_constraints: HashMap::new(),
            exclusion_constraints: HashMap::new(),
            permissions: Default::default(),
            options: Default::default(),
        },
    );
    let old = SchemaIR { macros: HashMap::new(), 
        routes: HashMap::new(),
        schema_version: "1.0".into(),
        meta: Meta {
            auth: Default::default(),
            observability_provider: None,
            rust_case_style: "camel".into(),
            db_case_style: "snake".into(),
            default_derives: vec![],
            db_backend: rustdbgen::ir::DatabaseBackend::Postgres,
        },
        enums: HashMap::new(),
        models: old_models,
        plugins: HashMap::new(),
        seeds: HashMap::new(),
    };

    // new schema with relation to Account
    let mut relations_new = HashMap::new();
    relations_new.insert(
        "post_author_fk".to_string(),
        RelationDef {
            on: "author_id".into(),
            references: FieldRef { model: "Account".into(), field: "id".into() },
        },
    );

    let mut new_models = HashMap::new();
    new_models.insert(
        "User".to_string(),
        ModelDef { includes: Vec::new(), 
            fields: {
                let mut f = HashMap::new();
                f.insert(
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
                f
            },
            indexes: HashMap::new(),
            relations: HashMap::new(),
            unique_constraints: HashMap::new(),
            check_constraints: HashMap::new(),
            exclusion_constraints: HashMap::new(),
            permissions: Default::default(),
            options: Default::default(),
        },
    );
    new_models.insert(
        "Account".to_string(),
        ModelDef { includes: Vec::new(), 
            fields: {
                let mut f = HashMap::new();
                f.insert(
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
                f
            },
            indexes: HashMap::new(),
            relations: HashMap::new(),
            unique_constraints: HashMap::new(),
            check_constraints: HashMap::new(),
            exclusion_constraints: HashMap::new(),
            permissions: Default::default(),
            options: Default::default(),
        },
    );
    new_models.insert(
        "Post".to_string(),
        ModelDef { includes: Vec::new(), 
            fields: fields,
            indexes: HashMap::new(),
            relations: relations_new,
            unique_constraints: HashMap::new(),
            check_constraints: HashMap::new(),
            exclusion_constraints: HashMap::new(),
            permissions: Default::default(),
            options: Default::default(),
        },
    );
    let new = SchemaIR { macros: HashMap::new(), 
        routes: HashMap::new(),
        schema_version: "1.0".into(),
        meta: Meta {
            auth: Default::default(),
            observability_provider: None,
            rust_case_style: "camel".into(),
            db_case_style: "snake".into(),
            default_derives: vec![],
            db_backend: rustdbgen::ir::DatabaseBackend::Postgres,
        },
        enums: HashMap::new(),
        models: new_models,
        plugins: HashMap::new(),
        seeds: HashMap::new(),
    };

    let (up, down) = generate_migration(Some(&old), &new).unwrap();
    assert!(up.contains("DROP CONSTRAINT post_author_fk"));
    assert!(up.contains("REFERENCES account.id"));
    assert!(down.contains("REFERENCES user.id"));
}

#[test]
fn diff_migration_renames_column() {
    let mut old_fields = HashMap::new();
    old_fields.insert(
        "username".to_string(),
        FieldDef {
            rust_type: "String".into(),
            db_type: Some("TEXT".into()),
            default: None,
            nullable: false,
            tags: Vec::new(),
            zod: None,
            rename_from: None,
            storage: None,
        },
    );
    let mut old_models = HashMap::new();
    old_models.insert(
        "User".to_string(),
        ModelDef { includes: Vec::new(), 
            fields: old_fields,
            indexes: HashMap::new(),
            relations: HashMap::new(),
            unique_constraints: HashMap::new(),
            check_constraints: HashMap::new(),
            exclusion_constraints: HashMap::new(),
            permissions: Default::default(),
            options: Default::default(),
        },
    );
    let old = SchemaIR { macros: HashMap::new(), 
        routes: HashMap::new(),
        schema_version: "1.0".into(),
        meta: Meta {
            auth: Default::default(),
            rust_case_style: "camel".into(),
            db_case_style: "snake".into(),
            default_derives: vec![],
            db_backend: rustdbgen::ir::DatabaseBackend::Postgres,
            observability_provider: None,
        },
        enums: HashMap::new(),
        models: old_models,
        plugins: HashMap::new(),
        seeds: HashMap::new(),
    };

    let mut new_fields = HashMap::new();
    new_fields.insert(
        "name".to_string(),
        FieldDef {
            rust_type: "String".into(),
            db_type: Some("TEXT".into()),
            default: None,
            nullable: false,
            rename_from: Some("username".into()),
            tags: Vec::new(),
            zod: None,
            storage: None,
        },
    );
    let mut new_models = HashMap::new();
    new_models.insert(
        "User".to_string(),
        ModelDef { includes: Vec::new(), 
            fields: new_fields,
            indexes: HashMap::new(),
            relations: HashMap::new(),
            unique_constraints: HashMap::new(),
            check_constraints: HashMap::new(),
            exclusion_constraints: HashMap::new(),
            permissions: Default::default(),
            options: Default::default(),
        },
    );
    let new = SchemaIR { macros: HashMap::new(), 
        routes: HashMap::new(),
        schema_version: "1.0".into(),
        meta: Meta {
            auth: Default::default(),
            rust_case_style: "camel".into(),
            db_case_style: "snake".into(),
            default_derives: vec![],
            db_backend: rustdbgen::ir::DatabaseBackend::Postgres,
            observability_provider: None,
        },
        enums: HashMap::new(),
        models: new_models,
        plugins: HashMap::new(),
        seeds: HashMap::new(),
    };

    let (up, down) = generate_migration(Some(&old), &new).unwrap();
    assert!(up.contains("RENAME COLUMN username TO name"));
    assert!(down.contains("RENAME COLUMN name TO username"));
}

#[test]
fn diff_migration_adds_check_constraint() {
    let mut fields = HashMap::new();
    fields.insert(
        "title".to_string(),
        FieldDef {
            rust_type: "String".into(),
            db_type: Some("TEXT".into()),
            default: None,
            nullable: false,
            tags: Vec::new(),
            zod: None,
            rename_from: None,
            storage: None,
        },
    );

    let mut old_models = HashMap::new();
    old_models.insert(
        "Post".to_string(),
        ModelDef { includes: Vec::new(), 
            fields: fields.clone(),
            indexes: HashMap::new(),
            relations: HashMap::new(),
            unique_constraints: HashMap::new(),
            check_constraints: HashMap::new(),
            exclusion_constraints: HashMap::new(),
            permissions: Default::default(),
            options: Default::default(),
        },
    );
    let old = SchemaIR { macros: HashMap::new(), 
        routes: HashMap::new(),
        schema_version: "1.0".into(),
        meta: Meta {
            auth: Default::default(),
            observability_provider: None,
            rust_case_style: "camel".into(),
            db_case_style: "snake".into(),
            default_derives: vec![],
            db_backend: rustdbgen::ir::DatabaseBackend::Postgres,
        },
        enums: HashMap::new(),
        models: old_models,
        plugins: HashMap::new(),
        seeds: HashMap::new(),
    };

    let mut new_models = HashMap::new();
    let mut checks = HashMap::new();
    checks.insert(
        "post_title_len".to_string(),
        CheckConstraintDef {
            expression: "char_length(title) > 0".into(),
        },
    );
    new_models.insert(
        "Post".to_string(),
        ModelDef { includes: Vec::new(), 
            fields: fields,
            indexes: HashMap::new(),
            relations: HashMap::new(),
            unique_constraints: HashMap::new(),
            check_constraints: checks,
            exclusion_constraints: HashMap::new(),
            permissions: Default::default(),
            options: Default::default(),
        },
    );
    let new = SchemaIR { macros: HashMap::new(), 
        routes: HashMap::new(),
        schema_version: "1.0".into(),
        meta: Meta {
            auth: Default::default(),
            observability_provider: None,
            rust_case_style: "camel".into(),
            db_case_style: "snake".into(),
            default_derives: vec![],
            db_backend: rustdbgen::ir::DatabaseBackend::Postgres,
        },
        enums: HashMap::new(),
        models: new_models,
        plugins: HashMap::new(),
        seeds: HashMap::new(),
    };

    let (up, down) = generate_migration(Some(&old), &new).unwrap();
    assert!(up.contains("ADD CONSTRAINT post_title_len CHECK"));
    assert!(down.contains("DROP CONSTRAINT post_title_len"));
}

#[test]
fn diff_migration_changes_check_constraint() {
    let mut fields = HashMap::new();
    fields.insert(
        "title".to_string(),
        FieldDef {
            rust_type: "String".into(),
            db_type: Some("TEXT".into()),
            default: None,
            nullable: false,
            tags: Vec::new(),
            zod: None,
            rename_from: None,
            storage: None,
        },
    );

    let mut old_checks = HashMap::new();
    old_checks.insert(
        "post_title_len".to_string(),
        CheckConstraintDef {
            expression: "char_length(title) > 0".into(),
        },
    );
    let mut old_models = HashMap::new();
    old_models.insert(
        "Post".to_string(),
        ModelDef { includes: Vec::new(), 
            fields: fields.clone(),
            indexes: HashMap::new(),
            relations: HashMap::new(),
            unique_constraints: HashMap::new(),
            check_constraints: old_checks,
            exclusion_constraints: HashMap::new(),
            permissions: Default::default(),
            options: Default::default(),
        },
    );
    let old = SchemaIR { macros: HashMap::new(), 
        routes: HashMap::new(),
        schema_version: "1.0".into(),
        meta: Meta {
            auth: Default::default(),
            observability_provider: None,
            rust_case_style: "camel".into(),
            db_case_style: "snake".into(),
            default_derives: vec![],
            db_backend: rustdbgen::ir::DatabaseBackend::Postgres,
        },
        enums: HashMap::new(),
        models: old_models,
        plugins: HashMap::new(),
        seeds: HashMap::new(),
    };

    let mut new_checks = HashMap::new();
    new_checks.insert(
        "post_title_len".to_string(),
        CheckConstraintDef {
            expression: "char_length(title) > 3".into(),
        },
    );
    let mut new_models = HashMap::new();
    new_models.insert(
        "Post".to_string(),
        ModelDef { includes: Vec::new(), 
            fields: fields,
            indexes: HashMap::new(),
            relations: HashMap::new(),
            unique_constraints: HashMap::new(),
            check_constraints: new_checks,
            exclusion_constraints: HashMap::new(),
            permissions: Default::default(),
            options: Default::default(),
        },
    );
    let new = SchemaIR { macros: HashMap::new(), 
        routes: HashMap::new(),
        schema_version: "1.0".into(),
        meta: Meta {
            auth: Default::default(),
            observability_provider: None,
            rust_case_style: "camel".into(),
            db_case_style: "snake".into(),
            default_derives: vec![],
            db_backend: rustdbgen::ir::DatabaseBackend::Postgres,
        },
        enums: HashMap::new(),
        models: new_models,
        plugins: HashMap::new(),
        seeds: HashMap::new(),
    };

    let (up, down) = generate_migration(Some(&old), &new).unwrap();
    assert!(up.contains("DROP CONSTRAINT post_title_len"));
    assert!(up.contains("ADD CONSTRAINT post_title_len CHECK"));
    assert!(down.contains("ADD CONSTRAINT post_title_len CHECK (char_length(title) > 0)"));
}

#[test]
fn diff_migration_adds_exclusion_constraint() {
    let mut fields = HashMap::new();
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

    let mut old_models = HashMap::new();
    old_models.insert(
        "Post".to_string(),
        ModelDef { includes: Vec::new(), 
            fields: fields.clone(),
            indexes: HashMap::new(),
            relations: HashMap::new(),
            unique_constraints: HashMap::new(),
            check_constraints: HashMap::new(),
            exclusion_constraints: HashMap::new(),
            permissions: Default::default(),
            options: Default::default(),
        },
    );
    let old = SchemaIR { macros: HashMap::new(), 
        routes: HashMap::new(),
        schema_version: "1.0".into(),
        meta: Meta {
            auth: Default::default(),
            observability_provider: None,
            rust_case_style: "camel".into(),
            db_case_style: "snake".into(),
            default_derives: vec![],
            db_backend: rustdbgen::ir::DatabaseBackend::Postgres,
        },
        enums: HashMap::new(),
        models: old_models,
        plugins: HashMap::new(),
        seeds: HashMap::new(),
    };

    let mut new_models = HashMap::new();
    let mut excls = HashMap::new();
    excls.insert(
        "post_excl".to_string(),
        ExclusionConstraintDef {
            definition: "USING gist (author_id WITH =)".into(),
        },
    );
    new_models.insert(
        "Post".to_string(),
        ModelDef { includes: Vec::new(), 
            fields: fields,
            indexes: HashMap::new(),
            relations: HashMap::new(),
            unique_constraints: HashMap::new(),
            check_constraints: HashMap::new(),
            exclusion_constraints: excls,
            permissions: Default::default(),
            options: Default::default(),
        },
    );
    let new = SchemaIR { macros: HashMap::new(), 
        routes: HashMap::new(),
        schema_version: "1.0".into(),
        meta: Meta {
            auth: Default::default(),
            observability_provider: None,
            rust_case_style: "camel".into(),
            db_case_style: "snake".into(),
            default_derives: vec![],
            db_backend: rustdbgen::ir::DatabaseBackend::Postgres,
        },
        enums: HashMap::new(),
        models: new_models,
        plugins: HashMap::new(),
        seeds: HashMap::new(),
    };

    let (up, down) = generate_migration(Some(&old), &new).unwrap();
    assert!(up.contains("ADD CONSTRAINT post_excl EXCLUDE"));
    assert!(down.contains("DROP CONSTRAINT post_excl"));
}

#[test]
fn diff_migration_changes_exclusion_constraint() {
    let mut fields = HashMap::new();
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

    let mut old_excls = HashMap::new();
    old_excls.insert(
        "post_excl".to_string(),
        ExclusionConstraintDef {
            definition: "USING gist (author_id WITH =)".into(),
        },
    );
    let mut old_models = HashMap::new();
    old_models.insert(
        "Post".to_string(),
        ModelDef { includes: Vec::new(), 
            fields: fields.clone(),
            indexes: HashMap::new(),
            relations: HashMap::new(),
            unique_constraints: HashMap::new(),
            check_constraints: HashMap::new(),
            exclusion_constraints: old_excls,
            permissions: Default::default(),
            options: Default::default(),
        },
    );
    let old = SchemaIR { macros: HashMap::new(), 
        routes: HashMap::new(),
        schema_version: "1.0".into(),
        meta: Meta {
            auth: Default::default(),
            observability_provider: None,
            rust_case_style: "camel".into(),
            db_case_style: "snake".into(),
            default_derives: vec![],
            db_backend: rustdbgen::ir::DatabaseBackend::Postgres,
        },
        enums: HashMap::new(),
        models: old_models,
        plugins: HashMap::new(),
        seeds: HashMap::new(),
    };

    let mut new_excls = HashMap::new();
    new_excls.insert(
        "post_excl".to_string(),
        ExclusionConstraintDef {
            definition: "USING gist (author_id WITH <>)".into(),
        },
    );
    let mut new_models = HashMap::new();
    new_models.insert(
        "Post".to_string(),
        ModelDef { includes: Vec::new(), 
            fields: fields,
            indexes: HashMap::new(),
            relations: HashMap::new(),
            unique_constraints: HashMap::new(),
            check_constraints: HashMap::new(),
            exclusion_constraints: new_excls,
            permissions: Default::default(),
            options: Default::default(),
        },
    );
    let new = SchemaIR { macros: HashMap::new(), 
        routes: HashMap::new(),
        schema_version: "1.0".into(),
        meta: Meta {
            auth: Default::default(),
            observability_provider: None,
            rust_case_style: "camel".into(),
            db_case_style: "snake".into(),
            default_derives: vec![],
            db_backend: rustdbgen::ir::DatabaseBackend::Postgres,
        },
        enums: HashMap::new(),
        models: new_models,
        plugins: HashMap::new(),
        seeds: HashMap::new(),
    };

    let (up, down) = generate_migration(Some(&old), &new).unwrap();
    assert!(up.contains("DROP CONSTRAINT post_excl"));
    assert!(up.contains("ADD CONSTRAINT post_excl EXCLUDE"));
    assert!(down.contains("ADD CONSTRAINT post_excl EXCLUDE USING gist (author_id WITH =)"));
}
