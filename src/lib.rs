pub mod ir;
pub mod codegen;
use regex::Regex;
use serde::Deserialize;
use sqlx::Row;
use std::collections::HashMap;
use std::fs;

#[derive(Deserialize)]
struct RawSchema {
    #[serde(default)]
    include: Vec<String>,
    #[serde(default)]
    schema_version: Option<String>,
    #[serde(default)]
    meta: Option<ir::Meta>,
    #[serde(default)]
    enums: HashMap<String, ir::EnumDef>,
    #[serde(default)]
    models: HashMap<String, ir::ModelDef>,
    #[serde(default)]
    routes: HashMap<String, ir::RouteDef>,
    #[serde(default)]
    plugins: HashMap<String, ir::PluginDef>,
    #[serde(default)]
    macros: HashMap<String, ir::MacroDef>,
    #[serde(default)]
    #[serde(rename = "seed")]
    seeds: HashMap<String, ir::SeedDef>,
}

/// Load a schema from the given path, processing any `include` directives.
use anyhow::{Context, Result, anyhow};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

fn load_schema_inner(path: &str, visited: &mut HashSet<PathBuf>) -> Result<ir::SchemaIR> {
    let canonical = fs::canonicalize(path)
        .with_context(|| format!("failed to canonicalize schema path {}", path))?;
    if !visited.insert(canonical.clone()) {
        anyhow::bail!("cyclic include detected for {}", canonical.display());
    }

    let text = fs::read_to_string(&canonical)
        .with_context(|| format!("failed to read schema file {}", canonical.display()))?;
    let raw: RawSchema = toml::from_str(&text)?;
    let mut ir = ir::SchemaIR {
        schema_version: raw.schema_version.unwrap_or_else(|| "1.0".into()),
        meta: raw.meta.unwrap_or_default(),
        enums: raw.enums,
        models: raw.models,
        routes: raw.routes,
        plugins: raw.plugins,
        macros: raw.macros,
        seeds: raw.seeds,
    };
    let base = canonical.parent().unwrap_or(Path::new(""));
    for inc in raw.include {
        let child_path = base.join(&inc);
        let child_str = child_path
            .to_str()
            .ok_or_else(|| anyhow!("non-UTF8 path: {}", child_path.display()))?;
        let child_ir = load_schema_inner(child_str, visited)?;
        for (name, en) in child_ir.enums {
            if ir.enums.contains_key(&name) {
                anyhow::bail!("duplicate enum definition for {}", name);
            }
            ir.enums.insert(name, en);
        }
        for (name, model) in child_ir.models {
            if ir.models.contains_key(&name) {
                anyhow::bail!("duplicate model definition for {}", name);
            }
            ir.models.insert(name, model);
        }
        for (name, route) in child_ir.routes {
            if ir.routes.contains_key(&name) {
                anyhow::bail!("duplicate route definition for {}", name);
            }
            ir.routes.insert(name, route);
        }
        for (name, plugin) in child_ir.plugins {
            if ir.plugins.contains_key(&name) {
                anyhow::bail!("duplicate plugin definition for {}", name);
            }
            ir.plugins.insert(name, plugin);
        }
        for (name, mac) in child_ir.macros {
            if ir.macros.contains_key(&name) {
                anyhow::bail!("duplicate macro definition for {}", name);
            }
            ir.macros.insert(name, mac);
        }
        for (name, seed) in child_ir.seeds {
            if ir.seeds.contains_key(&name) {
                anyhow::bail!("duplicate seed definition for {}", name);
            }
            ir.seeds.insert(name, seed);
        }
    }
    visited.remove(&canonical);
    
    // Convert unique indexes to unique constraints for error handling
    for model in ir.models.values_mut() {
        let mut unique_indexes = Vec::new();
        for (idx_name, idx) in &model.indexes {
            if idx.unique {
                unique_indexes.push((idx_name.clone(), idx.clone()));
            }
        }
        for (idx_name, idx) in unique_indexes {
            model.unique_constraints.insert(idx_name, ir::UniqueConstraintDef {
                fields: idx.fields,
            });
        }
    }
    
    Ok(ir)
}

pub fn load_schema(path: &str) -> Result<ir::SchemaIR> {
    let mut visited: HashSet<PathBuf> = HashSet::new();
    load_schema_inner(path, &mut visited)
}

fn drop_index_stmt(backend: ir::DatabaseBackend, table: &str, name: &str) -> String {
    match backend {
        ir::DatabaseBackend::Mysql => format!("DROP INDEX {} ON {}\n", name, table),
        _ => format!("DROP INDEX {}\n", name),
    }
}

fn drop_unique_stmt(backend: ir::DatabaseBackend, table: &str, name: &str) -> String {
    match backend {
        ir::DatabaseBackend::Mysql => format!("ALTER TABLE {} DROP INDEX {}\n", table, name),
        _ => format!("ALTER TABLE {} DROP CONSTRAINT {}\n", table, name),
    }
}

fn drop_check_stmt(backend: ir::DatabaseBackend, table: &str, name: &str) -> String {
    match backend {
        ir::DatabaseBackend::Mysql => format!("ALTER TABLE {} DROP CHECK {}\n", table, name),
        _ => format!("ALTER TABLE {} DROP CONSTRAINT {}\n", table, name),
    }
}

fn drop_fk_stmt(backend: ir::DatabaseBackend, table: &str, name: &str) -> String {
    match backend {
        ir::DatabaseBackend::Mysql => format!("ALTER TABLE {} DROP FOREIGN KEY {}\n", table, name),
        _ => format!("ALTER TABLE {} DROP CONSTRAINT {}\n", table, name),
    }
}

/// Generate SQL for an initial migration based on the provided schema.
pub fn generate_initial_migration(ir: &ir::SchemaIR) -> Result<(String, String)> {
    let mut up_body = String::new();
    let mut down_body = String::new();

    let mut enums: Vec<_> = ir.enums.iter().collect();
    enums.sort_by(|a, b| a.0.cmp(b.0));
    for (enum_name, en) in enums {
        up_body.push_str(&format!(
            "CREATE TYPE {} AS ENUM ({});\n",
            enum_name.to_lowercase(),
            en.variants
                .iter()
                .map(|v| format!("'{}'", v.to_lowercase()))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        down_body.push_str(&format!("DROP TYPE {};\n", enum_name.to_lowercase()));
    }

    let mut models: Vec<_> = ir.models.iter().collect();
    models.sort_by(|a, b| a.0.cmp(b.0));
    for (model_name, model) in models {
        up_body.push_str(&format!("CREATE TABLE {} (\n", model_name.to_lowercase()));
        let mut first = true;
        for (field_name, field) in &model.fields {
            if !first {
                up_body.push_str(",\n");
            }
            first = false;
            let db_type = field
                .db_type
                .as_deref()
                .ok_or_else(|| anyhow!("missing db_type for {}.{}", model_name, field_name))?;
            up_body.push_str(&format!("    {} {}", field_name, db_type));
            if !field.nullable {
                up_body.push_str(" NOT NULL");
            }
            if let Some(def) = &field.default {
                up_body.push_str(&format!(" DEFAULT {}", def));
            }
        }
        up_body.push_str("\n);\n\n");

        down_body.push_str(&format!("DROP TABLE {};", model_name.to_lowercase()));
        down_body.push_str("\n\n");
    }

    // Indexes
    let mut models: Vec<_> = ir.models.iter().collect();
    models.sort_by(|a, b| a.0.cmp(b.0));
    for (model_name, model) in models {
        let mut indexes: Vec<_> = model.indexes.iter().collect();
        indexes.sort_by(|a, b| a.0.cmp(b.0));
        for (idx_name, idx) in indexes {
            up_body.push_str(&format!(
                "CREATE {}INDEX {} ON {} ({});\n",
                if idx.unique { "UNIQUE " } else { "" },
                idx_name,
                model_name.to_lowercase(),
                idx.fields.join(", ")
            ));
            down_body.push_str(&drop_index_stmt(
                ir.meta.db_backend.clone(),
                &model_name.to_lowercase(),
                idx_name,
            ));
        }
    }

    // Unique constraints
    let mut models: Vec<_> = ir.models.iter().collect();
    models.sort_by(|a, b| a.0.cmp(b.0));
    for (model_name, model) in models {
        let mut ucs: Vec<_> = model.unique_constraints.iter().collect();
        ucs.sort_by(|a, b| a.0.cmp(b.0));
        for (uc_name, uc) in ucs {
            up_body.push_str(&format!(
                "ALTER TABLE {} ADD CONSTRAINT {} UNIQUE ({});\n",
                model_name.to_lowercase(),
                uc_name,
                uc.fields.join(", ")
            ));
            down_body.push_str(&drop_unique_stmt(
                ir.meta.db_backend.clone(),
                &model_name.to_lowercase(),
                uc_name,
            ));
        }
    }

    // Check constraints
    let mut models: Vec<_> = ir.models.iter().collect();
    models.sort_by(|a, b| a.0.cmp(b.0));
    for (model_name, model) in models {
        let mut cks: Vec<_> = model.check_constraints.iter().collect();
        cks.sort_by(|a, b| a.0.cmp(b.0));
        for (ck_name, ck) in cks {
            up_body.push_str(&format!(
                "ALTER TABLE {} ADD CONSTRAINT {} CHECK ({});\n",
                model_name.to_lowercase(),
                ck_name,
                ck.expression
            ));
            down_body.push_str(&drop_check_stmt(
                ir.meta.db_backend.clone(),
                &model_name.to_lowercase(),
                ck_name,
            ));
        }
    }

    // Exclusion constraints
    let mut models: Vec<_> = ir.models.iter().collect();
    models.sort_by(|a, b| a.0.cmp(b.0));
    for (model_name, model) in models {
        let mut exs: Vec<_> = model.exclusion_constraints.iter().collect();
        exs.sort_by(|a, b| a.0.cmp(b.0));
        for (ex_name, ex) in exs {
            up_body.push_str(&format!(
                "ALTER TABLE {} ADD CONSTRAINT {} EXCLUDE {};\n",
                model_name.to_lowercase(),
                ex_name,
                ex.definition
            ));
            down_body.push_str(&format!(
                "ALTER TABLE {} DROP CONSTRAINT {};\n",
                model_name.to_lowercase(),
                ex_name
            ));
        }
    }

    // Foreign keys
    let mut models: Vec<_> = ir.models.iter().collect();
    models.sort_by(|a, b| a.0.cmp(b.0));
    for (model_name, model) in models {
        let mut rels: Vec<_> = model.relations.iter().collect();
        rels.sort_by(|a, b| a.0.cmp(b.0));
        for (rel_name, rel) in rels {
            up_body.push_str(&format!(
                "ALTER TABLE {} ADD CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {}.{}\n",
                model_name.to_lowercase(),
                rel_name,
                rel.on,
                rel.references.model.to_lowercase(),
                rel.references.field
            ));
            down_body.push_str(&drop_fk_stmt(
                ir.meta.db_backend.clone(),
                &model_name.to_lowercase(),
                rel_name,
            ));
        }
    }

    let mut up = String::new();
    up.push_str("BEGIN;\n");
    up.push_str(&up_body);
    up.push_str("COMMIT;\n");

    let mut down = String::new();
    down.push_str("BEGIN;\n");
    down.push_str(&down_body);
    down.push_str("COMMIT;\n");

    Ok((up, down))
}

/// Generate SQL migration by diffing `old` schema against `new`.
/// If `old` is `None`, this is equivalent to `generate_initial_migration`.
pub fn generate_migration(
    old: Option<&ir::SchemaIR>,
    new: &ir::SchemaIR,
) -> Result<(String, String)> {
    if let Some(old_ir) = old {
        let mut up = String::new();
        let mut down = String::new();

        // Enum differences
        let mut enums: Vec<_> = new.enums.iter().collect();
        enums.sort_by(|a, b| a.0.cmp(b.0));
        for (name, en) in enums {
            match old_ir.enums.get(name) {
                None => {
                    up.push_str(&format!(
                        "CREATE TYPE {} AS ENUM ({});\n",
                        name.to_lowercase(),
                        en.variants
                            .iter()
                            .map(|v| format!("'{}'", v.to_lowercase()))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                    down.push_str(&format!("DROP TYPE {};\n", name.to_lowercase()));
                }
                Some(old_en) => {
                    if old_en.variants != en.variants {
                        up.push_str(&format!("DROP TYPE {};\n", name.to_lowercase()));
                        up.push_str(&format!(
                            "CREATE TYPE {} AS ENUM ({});\n",
                            name.to_lowercase(),
                            en.variants
                                .iter()
                                .map(|v| format!("'{}'", v.to_lowercase()))
                                .collect::<Vec<_>>()
                                .join(", ")
                        ));
                        down.push_str(&format!("DROP TYPE {};\n", name.to_lowercase()));
                        down.push_str(&format!(
                            "CREATE TYPE {} AS ENUM ({});\n",
                            name.to_lowercase(),
                            old_en
                                .variants
                                .iter()
                                .map(|v| format!("'{}'", v.to_lowercase()))
                                .collect::<Vec<_>>()
                                .join(", ")
                        ));
                    }
                }
            }
        }

        let mut old_enums: Vec<_> = old_ir.enums.iter().collect();
        old_enums.sort_by(|a, b| a.0.cmp(b.0));
        for (name, old_en) in old_enums {
            if !new.enums.contains_key(name) {
                up.push_str(&format!("DROP TYPE {};\n", name.to_lowercase()));
                down.push_str(&format!(
                    "CREATE TYPE {} AS ENUM ({});\n",
                    name.to_lowercase(),
                    old_en
                        .variants
                        .iter()
                        .map(|v| format!("'{}'", v.to_lowercase()))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }

        // New models
        let mut new_models: Vec<_> = new.models.iter().collect();
        new_models.sort_by(|a, b| a.0.cmp(b.0));
        for (model_name, model) in new_models {
            if !old_ir.models.contains_key(model_name) {
                up.push_str(&format!("CREATE TABLE {} (\n", model_name.to_lowercase()));
                let mut first = true;
                for (field_name, field) in &model.fields {
                    if !first {
                        up.push_str(",\n");
                    }
                    first = false;
                    let db_type = field.db_type.as_deref().ok_or_else(|| {
                        anyhow!("missing db_type for {}.{}", model_name, field_name)
                    })?;
                    up.push_str(&format!("    {} {}", field_name, db_type));
                    if !field.nullable {
                        up.push_str(" NOT NULL");
                    }
                    if let Some(def) = &field.default {
                        up.push_str(&format!(" DEFAULT {}", def));
                    }
                }
                up.push_str("\n);\n\n");

                down.push_str(&format!("DROP TABLE {};\n\n", model_name.to_lowercase()));
            }
        }

        // Removed models
        let mut old_models: Vec<_> = old_ir.models.iter().collect();
        old_models.sort_by(|a, b| a.0.cmp(b.0));
        for (model_name, model) in old_models {
            if !new.models.contains_key(model_name) {
                up.push_str(&format!("DROP TABLE {};\n\n", model_name.to_lowercase()));
                down.push_str(&format!("CREATE TABLE {} (\n", model_name.to_lowercase()));
                let mut first = true;
                for (field_name, field) in &model.fields {
                    if !first {
                        down.push_str(",\n");
                    }
                    first = false;
                    let db_type = field.db_type.as_deref().ok_or_else(|| {
                        anyhow!("missing db_type for {}.{}", model_name, field_name)
                    })?;
                    down.push_str(&format!("    {} {}", field_name, db_type));
                    if !field.nullable {
                        down.push_str(" NOT NULL");
                    }
                    if let Some(def) = &field.default {
                        down.push_str(&format!(" DEFAULT {}", def));
                    }
                }
                down.push_str("\n);\n\n");
            }
        }

        // Existing models - field diffs
        let mut new_models: Vec<_> = new.models.iter().collect();
        new_models.sort_by(|a, b| a.0.cmp(b.0));
        for (model_name, new_model) in new_models {
            if let Some(old_model) = old_ir.models.get(model_name) {
                use std::collections::HashSet;
                let mut handled_new = HashSet::new();
                let mut handled_old = HashSet::new();

                // Renamed fields
                let mut new_fields: Vec<_> = new_model.fields.iter().collect();
                new_fields.sort_by(|a, b| a.0.cmp(b.0));
                for (new_name, new_field) in new_fields {
                    if let Some(old_name) = &new_field.rename_from {
                        if let Some(old_field) = old_model.fields.get(old_name) {
                            up.push_str(&format!(
                                "ALTER TABLE {} RENAME COLUMN {} TO {};\n",
                                model_name.to_lowercase(),
                                old_name,
                                new_name
                            ));
                            down.push_str(&format!(
                                "ALTER TABLE {} RENAME COLUMN {} TO {};\n",
                                model_name.to_lowercase(),
                                new_name,
                                old_name
                            ));

                            if old_field.db_type != new_field.db_type {
                                up.push_str(&format!(
                                    "ALTER TABLE {} ALTER COLUMN {} TYPE {};\n",
                                    model_name.to_lowercase(),
                                    new_name,
                                    new_field.db_type.as_deref().ok_or_else(|| anyhow!(
                                        "missing db_type for {}.{}",
                                        model_name,
                                        new_name
                                    ))?
                                ));
                                down.push_str(&format!(
                                    "ALTER TABLE {} ALTER COLUMN {} TYPE {};\n",
                                    model_name.to_lowercase(),
                                    new_name,
                                    old_field.db_type.as_deref().ok_or_else(|| anyhow!(
                                        "missing db_type for {}.{}",
                                        model_name,
                                        new_name
                                    ))?
                                ));
                            }
                            if old_field.default != new_field.default {
                                match &new_field.default {
                                    Some(def) => up.push_str(&format!(
                                        "ALTER TABLE {} ALTER COLUMN {} SET DEFAULT {};\n",
                                        model_name.to_lowercase(),
                                        new_name,
                                        def
                                    )),
                                    None => up.push_str(&format!(
                                        "ALTER TABLE {} ALTER COLUMN {} DROP DEFAULT;\n",
                                        model_name.to_lowercase(),
                                        new_name
                                    )),
                                }
                                match &old_field.default {
                                    Some(def) => down.push_str(&format!(
                                        "ALTER TABLE {} ALTER COLUMN {} SET DEFAULT {};\n",
                                        model_name.to_lowercase(),
                                        new_name,
                                        def
                                    )),
                                    None => down.push_str(&format!(
                                        "ALTER TABLE {} ALTER COLUMN {} DROP DEFAULT;\n",
                                        model_name.to_lowercase(),
                                        new_name
                                    )),
                                }
                            }
                            if old_field.nullable != new_field.nullable {
                                if new_field.nullable {
                                    up.push_str(&format!(
                                        "ALTER TABLE {} ALTER COLUMN {} DROP NOT NULL;\n",
                                        model_name.to_lowercase(),
                                        new_name
                                    ));
                                } else {
                                    up.push_str(&format!(
                                        "ALTER TABLE {} ALTER COLUMN {} SET NOT NULL;\n",
                                        model_name.to_lowercase(),
                                        new_name
                                    ));
                                }
                                if old_field.nullable {
                                    down.push_str(&format!(
                                        "ALTER TABLE {} ALTER COLUMN {} DROP NOT NULL;\n",
                                        model_name.to_lowercase(),
                                        new_name
                                    ));
                                } else {
                                    down.push_str(&format!(
                                        "ALTER TABLE {} ALTER COLUMN {} SET NOT NULL;\n",
                                        model_name.to_lowercase(),
                                        new_name
                                    ));
                                }
                            }

                            handled_new.insert(new_name.clone());
                            handled_old.insert(old_name.clone());
                        }
                    }
                }

                // Added fields
                let mut new_fields: Vec<_> = new_model.fields.iter().collect();
                new_fields.sort_by(|a, b| a.0.cmp(b.0));
                for (field_name, field) in new_fields {
                    if handled_new.contains(field_name) {
                        continue;
                    }
                    if !old_model.fields.contains_key(field_name) {
                        let db_type = field.db_type.as_deref().ok_or_else(|| {
                            anyhow!("missing db_type for {}.{}", model_name, field_name)
                        })?;
                        up.push_str(&format!(
                            "ALTER TABLE {} ADD COLUMN {} {}",
                            model_name.to_lowercase(),
                            field_name,
                            db_type
                        ));
                        if !field.nullable {
                            up.push_str(" NOT NULL");
                        }
                        if let Some(def) = &field.default {
                            up.push_str(&format!(" DEFAULT {}", def));
                        }
                        up.push_str(";\n");
                        down.push_str(&format!(
                            "ALTER TABLE {} DROP COLUMN {};\n",
                            model_name.to_lowercase(),
                            field_name
                        ));
                    }
                }

                // Removed fields
                let mut old_fields: Vec<_> = old_model.fields.iter().collect();
                old_fields.sort_by(|a, b| a.0.cmp(b.0));
                for (field_name, field) in old_fields {
                    if handled_old.contains(field_name) {
                        continue;
                    }
                    if !new_model.fields.contains_key(field_name) {
                        up.push_str(&format!(
                            "ALTER TABLE {} DROP COLUMN {};\n",
                            model_name.to_lowercase(),
                            field_name
                        ));
                        let db_type = field.db_type.as_deref().ok_or_else(|| {
                            anyhow!("missing db_type for {}.{}", model_name, field_name)
                        })?;
                        down.push_str(&format!(
                            "ALTER TABLE {} ADD COLUMN {} {}",
                            model_name.to_lowercase(),
                            field_name,
                            db_type
                        ));
                        if !field.nullable {
                            down.push_str(" NOT NULL");
                        }
                        if let Some(def) = &field.default {
                            down.push_str(&format!(" DEFAULT {}", def));
                        }
                        down.push_str(";\n");
                    }
                }

                // Modified fields
                let mut new_fields: Vec<_> = new_model.fields.iter().collect();
                new_fields.sort_by(|a, b| a.0.cmp(b.0));
                for (field_name, new_field) in new_fields {
                    if handled_new.contains(field_name) {
                        continue;
                    }
                    if let Some(old_field) = old_model.fields.get(field_name) {
                        if old_field.db_type != new_field.db_type {
                            up.push_str(&format!(
                                "ALTER TABLE {} ALTER COLUMN {} TYPE {};\n",
                                model_name.to_lowercase(),
                                field_name,
                                new_field.db_type.as_deref().ok_or_else(|| anyhow!(
                                    "missing db_type for {}.{}",
                                    model_name,
                                    field_name
                                ))?
                            ));
                            down.push_str(&format!(
                                "ALTER TABLE {} ALTER COLUMN {} TYPE {};\n",
                                model_name.to_lowercase(),
                                field_name,
                                old_field.db_type.as_deref().ok_or_else(|| anyhow!(
                                    "missing db_type for {}.{}",
                                    model_name,
                                    field_name
                                ))?
                            ));
                        }
                        if old_field.default != new_field.default {
                            match &new_field.default {
                                Some(def) => up.push_str(&format!(
                                    "ALTER TABLE {} ALTER COLUMN {} SET DEFAULT {};\n",
                                    model_name.to_lowercase(),
                                    field_name,
                                    def
                                )),
                                None => up.push_str(&format!(
                                    "ALTER TABLE {} ALTER COLUMN {} DROP DEFAULT;\n",
                                    model_name.to_lowercase(),
                                    field_name
                                )),
                            }
                            match &old_field.default {
                                Some(def) => down.push_str(&format!(
                                    "ALTER TABLE {} ALTER COLUMN {} SET DEFAULT {};\n",
                                    model_name.to_lowercase(),
                                    field_name,
                                    def
                                )),
                                None => down.push_str(&format!(
                                    "ALTER TABLE {} ALTER COLUMN {} DROP DEFAULT;\n",
                                    model_name.to_lowercase(),
                                    field_name
                                )),
                            }
                        }
                        if old_field.nullable != new_field.nullable {
                            if new_field.nullable {
                                up.push_str(&format!(
                                    "ALTER TABLE {} ALTER COLUMN {} DROP NOT NULL;\n",
                                    model_name.to_lowercase(),
                                    field_name
                                ));
                            } else {
                                up.push_str(&format!(
                                    "ALTER TABLE {} ALTER COLUMN {} SET NOT NULL;\n",
                                    model_name.to_lowercase(),
                                    field_name
                                ));
                            }
                            if old_field.nullable {
                                down.push_str(&format!(
                                    "ALTER TABLE {} ALTER COLUMN {} DROP NOT NULL;\n",
                                    model_name.to_lowercase(),
                                    field_name
                                ));
                            } else {
                                down.push_str(&format!(
                                    "ALTER TABLE {} ALTER COLUMN {} SET NOT NULL;\n",
                                    model_name.to_lowercase(),
                                    field_name
                                ));
                            }
                        }
                    }
                }

                // Index changes
                let mut new_indexes: Vec<_> = new_model.indexes.iter().collect();
                new_indexes.sort_by(|a, b| a.0.cmp(b.0));
                for (idx_name, new_idx) in new_indexes {
                    match old_model.indexes.get(idx_name) {
                        None => {
                            up.push_str(&format!(
                                "CREATE {}INDEX {} ON {} ({});\n",
                                if new_idx.unique { "UNIQUE " } else { "" },
                                idx_name,
                                model_name.to_lowercase(),
                                new_idx.fields.join(", ")
                            ));
                            down.push_str(&drop_index_stmt(
                                new.meta.db_backend.clone(),
                                &model_name.to_lowercase(),
                                idx_name,
                            ));
                        }
                        Some(old_idx) => {
                            if old_idx.fields != new_idx.fields || old_idx.unique != new_idx.unique
                            {
                                up.push_str(&drop_index_stmt(
                                    new.meta.db_backend.clone(),
                                    &model_name.to_lowercase(),
                                    idx_name,
                                ));
                                up.push_str(&format!(
                                    "CREATE {}INDEX {} ON {} ({});\n",
                                    if new_idx.unique { "UNIQUE " } else { "" },
                                    idx_name,
                                    model_name.to_lowercase(),
                                    new_idx.fields.join(", ")
                                ));
                                down.push_str(&drop_index_stmt(
                                    new.meta.db_backend.clone(),
                                    &model_name.to_lowercase(),
                                    idx_name,
                                ));
                                down.push_str(&format!(
                                    "CREATE {}INDEX {} ON {} ({});\n",
                                    if old_idx.unique { "UNIQUE " } else { "" },
                                    idx_name,
                                    model_name.to_lowercase(),
                                    old_idx.fields.join(", ")
                                ));
                            }
                        }
                    }
                }

                let mut old_indexes: Vec<_> = old_model.indexes.iter().collect();
                old_indexes.sort_by(|a, b| a.0.cmp(b.0));
                for (idx_name, old_idx) in old_indexes {
                    if !new_model.indexes.contains_key(idx_name) {
                        up.push_str(&drop_index_stmt(
                            new.meta.db_backend.clone(),
                            &model_name.to_lowercase(),
                            idx_name,
                        ));
                        down.push_str(&format!(
                            "CREATE {}INDEX {} ON {} ({});\n",
                            if old_idx.unique { "UNIQUE " } else { "" },
                            idx_name,
                            model_name.to_lowercase(),
                            old_idx.fields.join(", ")
                        ));
                    }
                }

                // Unique constraint changes
                let mut new_ucs: Vec<_> = new_model.unique_constraints.iter().collect();
                new_ucs.sort_by(|a, b| a.0.cmp(b.0));
                for (uc_name, new_uc) in new_ucs {
                    match old_model.unique_constraints.get(uc_name) {
                        None => {
                            up.push_str(&format!(
                                "ALTER TABLE {} ADD CONSTRAINT {} UNIQUE ({});\n",
                                model_name.to_lowercase(),
                                uc_name,
                                new_uc.fields.join(", ")
                            ));
                            down.push_str(&drop_unique_stmt(
                                new.meta.db_backend.clone(),
                                &model_name.to_lowercase(),
                                uc_name,
                            ));
                        }
                        Some(old_uc) => {
                            if old_uc.fields != new_uc.fields {
                                up.push_str(&drop_unique_stmt(
                                    new.meta.db_backend.clone(),
                                    &model_name.to_lowercase(),
                                    uc_name,
                                ));
                                up.push_str(&format!(
                                    "ALTER TABLE {} ADD CONSTRAINT {} UNIQUE ({});\n",
                                    model_name.to_lowercase(),
                                    uc_name,
                                    new_uc.fields.join(", ")
                                ));
                                down.push_str(&drop_unique_stmt(
                                    new.meta.db_backend.clone(),
                                    &model_name.to_lowercase(),
                                    uc_name,
                                ));
                                down.push_str(&format!(
                                    "ALTER TABLE {} ADD CONSTRAINT {} UNIQUE ({});\n",
                                    model_name.to_lowercase(),
                                    uc_name,
                                    old_uc.fields.join(", ")
                                ));
                            }
                        }
                    }
                }

                // Check constraint changes
                let mut new_cks: Vec<_> = new_model.check_constraints.iter().collect();
                new_cks.sort_by(|a, b| a.0.cmp(b.0));
                for (ck_name, new_ck) in new_cks {
                    match old_model.check_constraints.get(ck_name) {
                        None => {
                            up.push_str(&format!(
                                "ALTER TABLE {} ADD CONSTRAINT {} CHECK ({});\n",
                                model_name.to_lowercase(),
                                ck_name,
                                new_ck.expression
                            ));
                            down.push_str(&drop_check_stmt(
                                new.meta.db_backend.clone(),
                                &model_name.to_lowercase(),
                                ck_name,
                            ));
                        }
                        Some(old_ck) => {
                            if old_ck.expression != new_ck.expression {
                                up.push_str(&drop_check_stmt(
                                    new.meta.db_backend.clone(),
                                    &model_name.to_lowercase(),
                                    ck_name,
                                ));
                                up.push_str(&format!(
                                    "ALTER TABLE {} ADD CONSTRAINT {} CHECK ({});\n",
                                    model_name.to_lowercase(),
                                    ck_name,
                                    new_ck.expression
                                ));
                                down.push_str(&drop_check_stmt(
                                    new.meta.db_backend.clone(),
                                    &model_name.to_lowercase(),
                                    ck_name,
                                ));
                                down.push_str(&format!(
                                    "ALTER TABLE {} ADD CONSTRAINT {} CHECK ({});\n",
                                    model_name.to_lowercase(),
                                    ck_name,
                                    old_ck.expression
                                ));
                            }
                        }
                    }
                }

                // Exclusion constraint changes
                let mut new_exs: Vec<_> = new_model.exclusion_constraints.iter().collect();
                new_exs.sort_by(|a, b| a.0.cmp(b.0));
                for (ex_name, new_ex) in new_exs {
                    match old_model.exclusion_constraints.get(ex_name) {
                        None => {
                            up.push_str(&format!(
                                "ALTER TABLE {} ADD CONSTRAINT {} EXCLUDE {};\n",
                                model_name.to_lowercase(),
                                ex_name,
                                new_ex.definition
                            ));
                            down.push_str(&format!(
                                "ALTER TABLE {} DROP CONSTRAINT {};\n",
                                model_name.to_lowercase(),
                                ex_name
                            ));
                        }
                        Some(old_ex) => {
                            if old_ex.definition != new_ex.definition {
                                up.push_str(&format!(
                                    "ALTER TABLE {} DROP CONSTRAINT {};\n",
                                    model_name.to_lowercase(),
                                    ex_name
                                ));
                                up.push_str(&format!(
                                    "ALTER TABLE {} ADD CONSTRAINT {} EXCLUDE {};\n",
                                    model_name.to_lowercase(),
                                    ex_name,
                                    new_ex.definition
                                ));
                                down.push_str(&format!(
                                    "ALTER TABLE {} DROP CONSTRAINT {};\n",
                                    model_name.to_lowercase(),
                                    ex_name
                                ));
                                down.push_str(&format!(
                                    "ALTER TABLE {} ADD CONSTRAINT {} EXCLUDE {};\n",
                                    model_name.to_lowercase(),
                                    ex_name,
                                    old_ex.definition
                                ));
                            }
                        }
                    }
                }

                let mut old_ucs: Vec<_> = old_model.unique_constraints.iter().collect();
                old_ucs.sort_by(|a, b| a.0.cmp(b.0));
                for (uc_name, old_uc) in old_ucs {
                    if !new_model.unique_constraints.contains_key(uc_name) {
                        up.push_str(&drop_unique_stmt(
                            new.meta.db_backend.clone(),
                            &model_name.to_lowercase(),
                            uc_name,
                        ));
                        down.push_str(&format!(
                            "ALTER TABLE {} ADD CONSTRAINT {} UNIQUE ({});\n",
                            model_name.to_lowercase(),
                            uc_name,
                            old_uc.fields.join(", ")
                        ));
                    }
                }

                let mut old_cks: Vec<_> = old_model.check_constraints.iter().collect();
                old_cks.sort_by(|a, b| a.0.cmp(b.0));
                for (ck_name, old_ck) in old_cks {
                    if !new_model.check_constraints.contains_key(ck_name) {
                        up.push_str(&drop_check_stmt(
                            new.meta.db_backend.clone(),
                            &model_name.to_lowercase(),
                            ck_name,
                        ));
                        down.push_str(&format!(
                            "ALTER TABLE {} ADD CONSTRAINT {} CHECK ({});\n",
                            model_name.to_lowercase(),
                            ck_name,
                            old_ck.expression
                        ));
                    }
                }

                let mut old_exs: Vec<_> = old_model.exclusion_constraints.iter().collect();
                old_exs.sort_by(|a, b| a.0.cmp(b.0));
                for (ex_name, old_ex) in old_exs {
                    if !new_model.exclusion_constraints.contains_key(ex_name) {
                        up.push_str(&format!(
                            "ALTER TABLE {} DROP CONSTRAINT {};\n",
                            model_name.to_lowercase(),
                            ex_name
                        ));
                        down.push_str(&format!(
                            "ALTER TABLE {} ADD CONSTRAINT {} EXCLUDE {};\n",
                            model_name.to_lowercase(),
                            ex_name,
                            old_ex.definition
                        ));
                    }
                }

                // Relation changes
                let mut new_rels: Vec<_> = new_model.relations.iter().collect();
                new_rels.sort_by(|a, b| a.0.cmp(b.0));
                for (rel_name, new_rel) in new_rels {
                    match old_model.relations.get(rel_name) {
                        None => {
                            // Added relation
                            let target = format!(
                                "{}.{}",
                                new_rel.references.model.to_lowercase(),
                                new_rel.references.field
                            );
                            up.push_str(&format!(
                                "ALTER TABLE {} ADD CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {}\n",
                                model_name.to_lowercase(),
                                rel_name,
                                new_rel.on,
                                target
                            ));
                            down.push_str(&drop_fk_stmt(
                                new.meta.db_backend.clone(),
                                &model_name.to_lowercase(),
                                rel_name,
                            ));
                        }
                        Some(old_rel) => {
                            if old_rel.on != new_rel.on
                                || old_rel.references.model != new_rel.references.model
                                || old_rel.references.field != new_rel.references.field
                            {
                                // Modified relation
                                up.push_str(&drop_fk_stmt(
                                    new.meta.db_backend.clone(),
                                    &model_name.to_lowercase(),
                                    rel_name,
                                ));
                                up.push_str(&format!(
                                    "ALTER TABLE {} ADD CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {}.{}\n",
                                    model_name.to_lowercase(),
                                    rel_name,
                                    new_rel.on,
                                    new_rel.references.model.to_lowercase(),
                                    new_rel.references.field
                                ));
                                down.push_str(&drop_fk_stmt(
                                    new.meta.db_backend.clone(),
                                    &model_name.to_lowercase(),
                                    rel_name,
                                ));
                                down.push_str(&format!(
                                    "ALTER TABLE {} ADD CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {}.{}\n",
                                    model_name.to_lowercase(),
                                    rel_name,
                                    old_rel.on,
                                    old_rel.references.model.to_lowercase(),
                                    old_rel.references.field
                                ));
                            }
                        }
                    }
                }

                // Removed relations
                let mut old_rels: Vec<_> = old_model.relations.iter().collect();
                old_rels.sort_by(|a, b| a.0.cmp(b.0));
                for (rel_name, rel) in old_rels {
                    if !new_model.relations.contains_key(rel_name) {
                        up.push_str(&drop_fk_stmt(
                            new.meta.db_backend.clone(),
                            &model_name.to_lowercase(),
                            rel_name,
                        ));
                        down.push_str(&format!(
                            "ALTER TABLE {} ADD CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {}.{}\n",
                            model_name.to_lowercase(),
                            rel_name,
                            rel.on,
                            rel.references.model.to_lowercase(),
                            rel.references.field
                        ));
                    }
                }
            }
        }

        if up.trim().is_empty() {
            Ok((String::new(), String::new()))
        } else {
            let mut up_tx = String::new();
            up_tx.push_str("BEGIN;\n");
            up_tx.push_str(&up);
            up_tx.push_str("COMMIT;\n");

            let mut down_tx = String::new();
            down_tx.push_str("BEGIN;\n");
            down_tx.push_str(&down);
            down_tx.push_str("COMMIT;\n");

            Ok((up_tx, down_tx))
        }
    } else {
        generate_initial_migration(new)
    }
}


pub fn generate_code_multi(ir: &ir::SchemaIR, output_dir: &std::path::Path) -> anyhow::Result<()> {
    codegen::generate_code_multi(ir, output_dir)
}


/// Introspect the connected Postgres database and build a `SchemaIR` representing
/// the discovered tables, columns, indexes and constraints.
pub async fn introspect_schema(
    url: &str,
    db_type_map: &std::collections::HashMap<String, String>,
) -> Result<ir::SchemaIR> {
    match infer_backend_from_url(url) {
        Some(ir::DatabaseBackend::Postgres) => {
            let pool = sqlx::PgPool::connect(url).await?;
            introspect_schema_postgres(&pool, db_type_map).await
        }
        Some(ir::DatabaseBackend::Sqlite) => {
            let pool = sqlx::SqlitePool::connect(url).await?;
            introspect_schema_sqlite(&pool, db_type_map).await
        }
        Some(ir::DatabaseBackend::Mysql) => {
            let pool = sqlx::MySqlPool::connect(url).await?;
            introspect_schema_mysql(&pool, db_type_map).await
        }
        _ => Err(anyhow!("unsupported database url")),
    }
}

async fn introspect_schema_postgres(
    pool: &sqlx::PgPool,
    db_type_map: &std::collections::HashMap<String, String>,
) -> Result<ir::SchemaIR> {
    let table_rows = sqlx::query(
        "SELECT table_name FROM information_schema.tables \
         WHERE table_schema = 'public' AND table_type = 'BASE TABLE'",
    )
    .fetch_all(pool)
    .await?;

    let enum_rows = sqlx::query(
        "SELECT t.typname, e.enumlabel FROM pg_type t \n             JOIN pg_enum e ON t.oid = e.enumtypid \n             JOIN pg_namespace n ON n.oid = t.typnamespace \n             WHERE n.nspname = 'public' ORDER BY t.typname, e.enumsortorder",
    )
    .fetch_all(pool)
    .await?;
    let mut enums: HashMap<String, ir::EnumDef> = HashMap::new();
    for r in enum_rows {
        let name: String = r.get("typname");
        let label: String = r.get("enumlabel");
        enums
            .entry(name)
            .or_insert_with(|| ir::EnumDef { variants: vec![] })
            .variants
            .push(label);
    }

    let mut models = HashMap::new();

    for row in table_rows {
        let table_name: String = row.get("table_name");
        let column_rows = sqlx::query(
            "SELECT column_name, data_type, is_nullable, column_default \
             FROM information_schema.columns \
             WHERE table_schema='public' AND table_name=$1 \
             ORDER BY ordinal_position",
        )
        .bind(&table_name)
        .fetch_all(pool)
        .await?;

        let mut fields = HashMap::new();
        for c in column_rows {
            let col_name: String = c.get("column_name");
            let data_type: String = c.get("data_type");
            let nullable: String = c.get("is_nullable");
            let default: Option<String> = c.try_get("column_default").ok();
            let rust_type = db_type_map
                .get(&data_type)
                .cloned()
                .unwrap_or_else(|| "String".to_string());
            fields.insert(
                col_name,
                ir::FieldDef {
                    rust_type: rust_type.to_string(),
                    db_type: Some(data_type),
                    default,
                    nullable: nullable == "YES",
                    rename_from: None,
                    tags: Vec::new(),
                    zod: None,
                    storage: None,
                },
            );
        }

        // indexes using pg_index and pg_attribute for reliability
        let index_rows = sqlx::query(
            "SELECT i.relname AS indexname, ix.indisunique, a.attname, ord.ordinality \
             FROM pg_class t \
             JOIN pg_index ix ON t.oid = ix.indrelid \
             JOIN pg_class i ON i.oid = ix.indexrelid \
             JOIN LATERAL unnest(ix.indkey) WITH ORDINALITY AS ord(attnum, ordinality) ON true \
             JOIN pg_attribute a ON a.attrelid = t.oid AND a.attnum = ord.attnum \
             WHERE t.relname = $1 AND NOT ix.indisprimary \
             ORDER BY indexname, ordinality",
        )
        .bind(&table_name)
        .fetch_all(pool)
        .await?;
        let mut indexes: HashMap<String, ir::IndexDef> = HashMap::new();
        for r in index_rows {
            let name: String = r.get("indexname");
            let col: String = r.get("attname");
            let unique: bool = r.get("indisunique");
            indexes
                .entry(name)
                .or_insert_with(|| ir::IndexDef {
                    fields: Vec::new(),
                    unique,
                })
                .fields
                .push(col);
        }

        // unique constraints
        let uc_rows = sqlx::query(
            "SELECT tc.constraint_name, kcu.column_name FROM information_schema.table_constraints tc \
             JOIN information_schema.key_column_usage kcu ON tc.constraint_name = kcu.constraint_name AND tc.table_name = kcu.table_name \
             WHERE tc.table_schema='public' AND tc.table_name=$1 AND tc.constraint_type='UNIQUE' \
             ORDER BY kcu.ordinal_position",
        )
        .bind(&table_name)
        .fetch_all(pool)
        .await?;
        let mut unique_constraints: HashMap<String, ir::UniqueConstraintDef> = HashMap::new();
        for r in uc_rows {
            let name: String = r.get("constraint_name");
            let col: String = r.get("column_name");
            unique_constraints
                .entry(name)
                .or_insert_with(|| ir::UniqueConstraintDef { fields: vec![] })
                .fields
                .push(col);
        }

        // check constraints
        let ck_rows = sqlx::query(
            "SELECT tc.constraint_name, cc.check_clause FROM information_schema.table_constraints tc \
             JOIN information_schema.check_constraints cc ON tc.constraint_name = cc.constraint_name \
             WHERE tc.table_schema='public' AND tc.table_name=$1 AND tc.constraint_type='CHECK'",
        )
        .bind(&table_name)
        .fetch_all(pool)
        .await?;
        let mut check_constraints = HashMap::new();
        for r in ck_rows {
            let name: String = r.get("constraint_name");
            let expr: String = r.get("check_clause");
            check_constraints.insert(name, ir::CheckConstraintDef { expression: expr });
        }

        // exclusion constraints
        let ex_rows = sqlx::query(
            "SELECT conname, pg_get_constraintdef(oid) AS def FROM pg_constraint \
             WHERE contype = 'x' AND conrelid = $1::regclass",
        )
        .bind(&table_name)
        .fetch_all(pool)
        .await?;
        let mut exclusion_constraints = HashMap::new();
        for r in ex_rows {
            let name: String = r.get("conname");
            let def: String = r.get("def");
            let definition = def
                .split_once("EXCLUDE")
                .map(|(_, rest)| rest)
                .unwrap_or("")
                .trim()
                .to_string();
            exclusion_constraints.insert(name, ir::ExclusionConstraintDef { definition });
        }

        // foreign keys
        let fk_rows = sqlx::query(
            "SELECT tc.constraint_name, kcu.column_name, ccu.table_name AS foreign_table, ccu.column_name AS foreign_column \
             FROM information_schema.table_constraints tc \
             JOIN information_schema.key_column_usage kcu ON tc.constraint_name = kcu.constraint_name AND tc.table_name = kcu.table_name \
             JOIN information_schema.constraint_column_usage ccu ON ccu.constraint_name = tc.constraint_name \
             WHERE tc.table_schema='public' AND tc.table_name=$1 AND tc.constraint_type='FOREIGN KEY'",
        )
        .bind(&table_name)
        .fetch_all(pool)
        .await?;
        let mut relations = HashMap::new();
        for r in fk_rows {
            let name: String = r.get("constraint_name");
            let col: String = r.get("column_name");
            let foreign_table: String = r.get("foreign_table");
            let foreign_column: String = r.get("foreign_column");
            relations.insert(
                name,
                ir::RelationDef {
                    on: col,
                    references: ir::FieldRef {
                        model: foreign_table,
                        field: foreign_column,
                    },
                },
            );
        }

        models.insert(
            table_name.to_string(),
            ir::ModelDef {
                includes: Vec::new(),
                fields,
                indexes,
                relations,
                unique_constraints,
                check_constraints,
                exclusion_constraints,
                permissions: ir::Permissions::default(),
                options: ir::ModelOptions::default(),
                owned_by: None,
            },
        );
    }

    Ok(ir::SchemaIR {
        schema_version: "1.0".into(),
        meta: ir::Meta {
            rust_case_style: "camel".into(),
            db_case_style: "snake".into(),
            default_derives: vec![],
            observability_provider: None,
            db_backend: ir::DatabaseBackend::Postgres,
            auth: ir::AuthConfig::default(),
        },
        enums,
        models,
        routes: HashMap::new(),
        plugins: HashMap::new(),
        macros: HashMap::new(),
        seeds: HashMap::new(),
    })
}

async fn introspect_schema_sqlite(
    pool: &sqlx::SqlitePool,
    db_type_map: &std::collections::HashMap<String, String>,
) -> Result<ir::SchemaIR> {
    let table_rows = sqlx::query(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
    )
    .fetch_all(pool)
    .await?;

    let mut models = HashMap::new();
    let re_check = Regex::new(r#"(?i)CONSTRAINT\s+`?(\w+)`?\s+CHECK\s*\(([^)]+)\)"#).unwrap();

    for row in table_rows {
        let table_name: String = row.get("name");

        let column_rows = sqlx::query(&format!("PRAGMA table_info(`{}`)", table_name))
            .fetch_all(pool)
            .await?;

        let mut fields = HashMap::new();
        for c in column_rows {
            let col_name: String = c.get("name");
            let data_type: String = c.get("type");
            let notnull: i64 = c.get("notnull");
            let default: Option<String> = c.try_get("dflt_value").ok();
            let rust_type = db_type_map
                .get(&data_type.to_lowercase())
                .cloned()
                .unwrap_or_else(|| "String".to_string());
            fields.insert(
                col_name,
                ir::FieldDef {
                    rust_type,
                    db_type: Some(data_type),
                    default,
                    nullable: notnull == 0,
                    rename_from: None,
                    tags: Vec::new(),
                    zod: None,
                    storage: None,
                },
            );
        }

        let index_rows = sqlx::query(&format!("PRAGMA index_list(`{}`)", table_name))
            .fetch_all(pool)
            .await?;
        let mut indexes = HashMap::new();
        let mut unique_constraints = HashMap::new();
        for r in index_rows {
            let idx_name: String = r.get("name");
            let unique: i64 = r.get("unique");
            let origin: Option<String> = r.try_get("origin").ok();
            let info = sqlx::query(&format!("PRAGMA index_info(`{}`)", idx_name))
                .fetch_all(pool)
                .await?;
            let mut fields_vec = Vec::new();
            for i in info {
                let col: String = i.get("name");
                fields_vec.push(col);
            }
            if unique != 0 && origin.as_deref() == Some("u") {
                unique_constraints.insert(
                    idx_name.clone(),
                    ir::UniqueConstraintDef {
                        fields: fields_vec.clone(),
                    },
                );
            }
            indexes.insert(
                idx_name.clone(),
                ir::IndexDef {
                    fields: fields_vec,
                    unique: unique != 0,
                },
            );
        }

        let fk_rows = sqlx::query(&format!("PRAGMA foreign_key_list(`{}`)", table_name))
            .fetch_all(pool)
            .await?;
        let mut relations = HashMap::new();
        for fk in fk_rows {
            let id: i64 = fk.get("id");
            let seq: i64 = fk.get("seq");
            let from: String = fk.get("from");
            let to_table: String = fk.get("table");
            let to_col: String = fk.get("to");
            relations.insert(
                format!("fk_{}_{}_{}", table_name, id, seq),
                ir::RelationDef {
                    on: from,
                    references: ir::FieldRef {
                        model: to_table,
                        field: to_col,
                    },
                },
            );
        }

        let create_row =
            sqlx::query("SELECT sql FROM sqlite_master WHERE type='table' AND name = ?")
                .bind(&table_name)
                .fetch_one(pool)
                .await?;
        let create_sql: String = create_row.get("sql");
        let mut check_constraints = HashMap::new();
        for cap in re_check.captures_iter(&create_sql) {
            check_constraints.insert(
                cap[1].to_string(),
                ir::CheckConstraintDef {
                    expression: cap[2].to_string(),
                },
            );
        }

        // unique_constraints collected earlier from index parsing

        models.insert(
            table_name.clone(),
            ir::ModelDef {
                includes: Vec::new(),
                fields,
                indexes,
                relations,
                unique_constraints,
                check_constraints,
                exclusion_constraints: HashMap::new(),
                permissions: ir::Permissions::default(),
                options: ir::ModelOptions::default(),
                owned_by: None,
            },
        );
    }

    Ok(ir::SchemaIR {
        schema_version: "1.0".into(),
        meta: ir::Meta {
            rust_case_style: "camel".into(),
            db_case_style: "snake".into(),
            default_derives: vec![],
            observability_provider: None,
            db_backend: ir::DatabaseBackend::Sqlite,
            auth: ir::AuthConfig::default(),
        },
        enums: HashMap::new(),
        models,
        routes: HashMap::new(),
        plugins: HashMap::new(),
        macros: HashMap::new(),
        seeds: HashMap::new(),
    })
}

async fn introspect_schema_mysql(
    pool: &sqlx::MySqlPool,
    db_type_map: &std::collections::HashMap<String, String>,
) -> Result<ir::SchemaIR> {
    let table_rows = sqlx::query("SHOW TABLES").fetch_all(pool).await?;

    let mut models = HashMap::new();

    for row in table_rows {
        let table_name: String = row.get(0);
        let column_rows = sqlx::query(&format!("SHOW COLUMNS FROM `{}`", table_name))
            .fetch_all(pool)
            .await?;

        let mut fields = HashMap::new();
        for c in column_rows {
            let col_name: String = c.get("Field");
            let data_type: String = c.get("Type");
            let nullable: String = c.get("Null");
            let default: Option<String> = c.try_get("Default").ok();
            let rust_type = db_type_map
                .get(&data_type.to_lowercase())
                .cloned()
                .unwrap_or_else(|| "String".to_string());
            fields.insert(
                col_name,
                ir::FieldDef {
                    rust_type,
                    db_type: Some(data_type),
                    default,
                    nullable: nullable == "YES",
                    rename_from: None,
                    tags: Vec::new(),
                    zod: None,
                    storage: None,
                },
            );
        }

        // indexes
        let index_rows = sqlx::query(&format!("SHOW INDEX FROM `{}`", table_name))
            .fetch_all(pool)
            .await?;
        let mut indexes: HashMap<String, ir::IndexDef> = HashMap::new();
        for r in index_rows {
            let idx_name: String = r.get("Key_name");
            let col_name: String = r.get("Column_name");
            let non_unique: i64 = r.get("Non_unique");
            let entry = indexes.entry(idx_name).or_insert_with(|| ir::IndexDef {
                fields: Vec::new(),
                unique: non_unique == 0,
            });
            entry.fields.push(col_name);
        }

        // unique constraints
        let uc_rows = sqlx::query(
            "SELECT tc.CONSTRAINT_NAME, kcu.COLUMN_NAME \
             FROM information_schema.TABLE_CONSTRAINTS tc \
             JOIN information_schema.KEY_COLUMN_USAGE kcu \
               ON tc.CONSTRAINT_NAME = kcu.CONSTRAINT_NAME AND tc.TABLE_NAME = kcu.TABLE_NAME \
             WHERE tc.TABLE_SCHEMA = DATABASE() AND tc.TABLE_NAME = ? AND tc.CONSTRAINT_TYPE = 'UNIQUE' \
             ORDER BY kcu.ORDINAL_POSITION",
        )
        .bind(&table_name)
        .fetch_all(pool)
        .await?;
        let mut unique_constraints = HashMap::new();
        for r in uc_rows {
            let name: String = r.get("CONSTRAINT_NAME");
            let col: String = r.get("COLUMN_NAME");
            unique_constraints
                .entry(name)
                .or_insert_with(|| ir::UniqueConstraintDef { fields: vec![] })
                .fields
                .push(col);
        }

        // check constraints (MySQL 8+)
        let ck_rows = sqlx::query(
            "SELECT tc.CONSTRAINT_NAME, cc.CHECK_CLAUSE \
             FROM information_schema.TABLE_CONSTRAINTS tc \
             JOIN information_schema.CHECK_CONSTRAINTS cc \
               ON tc.CONSTRAINT_NAME = cc.CONSTRAINT_NAME \
             WHERE tc.TABLE_SCHEMA = DATABASE() AND tc.TABLE_NAME = ? AND tc.CONSTRAINT_TYPE = 'CHECK'",
        )
        .bind(&table_name)
        .fetch_all(pool)
        .await?;
        let mut check_constraints = HashMap::new();
        for r in ck_rows {
            let name: String = r.get("CONSTRAINT_NAME");
            let clause: String = r.get("CHECK_CLAUSE");
            check_constraints.insert(name, ir::CheckConstraintDef { expression: clause });
        }

        // foreign keys
        let fk_rows = sqlx::query(
            "SELECT CONSTRAINT_NAME, COLUMN_NAME, REFERENCED_TABLE_NAME, REFERENCED_COLUMN_NAME \
             FROM information_schema.KEY_COLUMN_USAGE \
             WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ? AND REFERENCED_TABLE_NAME IS NOT NULL",
        )
        .bind(&table_name)
        .fetch_all(pool)
        .await?;
        let mut relations = HashMap::new();
        for fk in fk_rows {
            let cname: String = fk.get("CONSTRAINT_NAME");
            let column: String = fk.get("COLUMN_NAME");
            let to_table: String = fk.get("REFERENCED_TABLE_NAME");
            let to_col: String = fk.get("REFERENCED_COLUMN_NAME");
            relations.insert(
                cname.clone(),
                ir::RelationDef {
                    on: column,
                    references: ir::FieldRef {
                        model: to_table,
                        field: to_col,
                    },
                },
            );
        }

        models.insert(
            table_name.clone(),
            ir::ModelDef {
                includes: Vec::new(),
                fields,
                indexes,
                relations,
                unique_constraints,
                check_constraints,
                exclusion_constraints: HashMap::new(),
                permissions: ir::Permissions::default(),
                options: ir::ModelOptions::default(),
                owned_by: None,
            },
        );
    }

    Ok(ir::SchemaIR {
        schema_version: "1.0".into(),
        meta: ir::Meta {
            rust_case_style: "camel".into(),
            db_case_style: "snake".into(),
            default_derives: vec![],
            observability_provider: None,
            db_backend: ir::DatabaseBackend::Mysql,
            auth: ir::AuthConfig::default(),
        },
        enums: HashMap::new(),
        models,
        routes: HashMap::new(),
        plugins: HashMap::new(),
        macros: HashMap::new(),
        seeds: HashMap::new(),
    })
}

pub fn lint_schema(ir: &ir::SchemaIR) -> Vec<String> {
    let mut errors = Vec::new();
    let mut role_variants: std::collections::HashSet<&str> = std::collections::HashSet::new();
    if let Some(en) = ir.enums.get("Role") {
        for v in &en.variants {
            role_variants.insert(v.as_str());
        }
    }
    role_variants.insert(&ir.meta.auth.public_role);
    role_variants.insert(&ir.meta.auth.anonymous_role);
    let mut route_paths: std::collections::HashMap<&str, &str> = std::collections::HashMap::new();

    // check for duplicate route paths
    for (route_name, route) in &ir.routes {
        if let Some(existing) = route_paths.insert(&route.path, route_name) {
            errors.push(format!(
                "Route {} and {} share the same path {}",
                existing, route_name, route.path
            ));
        }
        // validate HTTP methods
        for method in &route.methods {
            let m = method.to_uppercase();
            match m.as_str() {
                "GET" | "POST" | "PUT" | "DELETE" | "PATCH" => {}
                _ => errors.push(format!(
                    "Route {} uses unsupported HTTP method {}",
                    route_name, method
                )),
            }
        }
        if route.methods.is_empty() {
            errors.push(format!("Route {} defines no methods", route_name));
        }
        for role in route
            .permissions
            .read
            .iter()
            .chain(&route.permissions.update)
            .chain(&route.permissions.delete)
        {
            if !role_variants.contains(role.as_str()) {
                errors.push(format!(
                    "Route {} references unknown role {}",
                    route_name, role
                ));
            }
        }
    }

    for (model_name, model) in &ir.models {
        // check relations
        for (rel_name, rel) in &model.relations {
            if !model.fields.contains_key(&rel.on) {
                errors.push(format!(
                    "Relation {} on model {} references unknown field {}",
                    rel_name, model_name, rel.on
                ));
            }
            let target_model = &rel.references.model;
            let target_field = &rel.references.field;
            match ir.models.get(target_model) {
                Some(target) => {
                    if !target.fields.contains_key(target_field) {
                        errors.push(format!(
                            "Relation {} on model {} references unknown field {}.{}",
                            rel_name, model_name, target_model, target_field
                        ));
                    }
                    // check for reciprocal relation
                    let mut has_reverse = false;
                    for rev_rel in target.relations.values() {
                        if rev_rel.references.model == *model_name && rev_rel.on == *target_field {
                            has_reverse = true;
                            break;
                        }
                    }
                    if !has_reverse {
                        errors.push(format!(
                            "Relation {} on model {} is not reciprocated by model {}",
                            rel_name, model_name, target_model
                        ));
                    }
                }
                None => errors.push(format!(
                    "Relation {} on model {} references unknown model {}",
                    rel_name, model_name, target_model
                )),
            }
        }
        // check indexes
        for (idx_name, idx) in &model.indexes {
            for f in &idx.fields {
                if !model.fields.contains_key(f) {
                    errors.push(format!(
                        "Index {} on model {} references unknown field {}",
                        idx_name, model_name, f
                    ));
                }
            }
        }
        // unique constraints
        for (uc_name, uc) in &model.unique_constraints {
            for f in &uc.fields {
                if !model.fields.contains_key(f) {
                    errors.push(format!(
                        "Unique constraint {} on model {} references unknown field {}",
                        uc_name, model_name, f
                    ));
                }
            }
        }
        // check constraints
        for (cc_name, cc) in &model.check_constraints {
            if cc.expression.trim().is_empty() {
                errors.push(format!(
                    "Check constraint {} on model {} has empty expression",
                    cc_name, model_name
                ));
            } else {
                let tokens: Vec<&str> = cc
                    .expression
                    .split(|c: char| !c.is_alphanumeric() && c != '_')
                    .collect();
                let mut has_field = false;
                for token in tokens {
                    if model.fields.contains_key(token) {
                        has_field = true;
                        break;
                    }
                }
                if !has_field {
                    errors.push(format!(
                        "Check constraint {} on model {} references no known fields",
                        cc_name, model_name
                    ));
                }
            }
        }
        // verify enum references
        for (field_name, field) in &model.fields {
            let mut ty = field.rust_type.as_str();
            if let Some(inner) = ty.strip_prefix("Option<").and_then(|s| s.strip_suffix('>')) {
                ty = inner;
            }
            if ir.enums.contains_key(ty) {
                continue;
            }
            match ty {
                "String"
                | "Uuid"
                | "bool"
                | "i32"
                | "i64"
                | "f32"
                | "f64"
                | "DateTime<Utc>"
                | "chrono::DateTime<chrono::Utc>" => {}
                _ => {
                    if ir.enums.contains_key(ty) {
                        // handled above
                    } else if field
                        .db_type
                        .as_deref()
                        .map(|s| ir.enums.contains_key(s))
                        .unwrap_or(false)
                    {
                        errors.push(format!(
                            "Field {}.{} references unknown enum {}",
                            model_name, field_name, ty
                        ));
                    }
                }
            }
        }
        for role in model
            .permissions
            .read
            .iter()
            .chain(&model.permissions.update)
            .chain(&model.permissions.delete)
        {
            if !role_variants.contains(role.as_str()) {
                errors.push(format!(
                    "Model {} references unknown role {}",
                    model_name, role
                ));
            }
        }
        for ex_name in model.exclusion_constraints.keys() {
            errors.push(format!(
                "Exclusion constraint {} on model {} uses a raw definition that cannot be fully validated",
                ex_name, model_name
            ));
        }
    }

    // validate seed blocks
    for (model_name, seed) in &ir.seeds {
        let Some(model) = ir.models.get(model_name) else {
            errors.push(format!(
                "Seed block references unknown model {}",
                model_name
            ));
            continue;
        };
        for row in &seed.rows {
            for field in row.keys() {
                if !model.fields.contains_key(field) {
                    errors.push(format!(
                        "Seed data for model {} has unknown field {}",
                        model_name, field
                    ));
                }
            }
        }
    }
    errors
}

/// Apply macros to models by merging macro fields and options.
pub fn apply_macros(ir: &mut ir::SchemaIR) {
    for model in ir.models.values_mut() {
        let includes = model.includes.clone();
        for mac_name in includes {
            if let Some(mac) = ir.macros.get(&mac_name) {
                for (fname, fdef) in &mac.fields {
                    model.fields.entry(fname.clone()).or_insert(fdef.clone());
                }
                if mac.options.timestamps {
                    model.options.timestamps = true;
                }
                if mac.options.soft_delete {
                    model.options.soft_delete = true;
                }
            }
        }
    }
}

/// Expand model options like `timestamps` and `soft_delete` into concrete fields
pub fn apply_model_options(ir: &mut ir::SchemaIR) {
    for model in ir.models.values_mut() {
        if model.options.timestamps {
            model
                .fields
                .entry("created_at".into())
                .or_insert(ir::FieldDef {
                    rust_type: "DateTime<Utc>".into(),
                    db_type: Some("TIMESTAMPTZ".into()),
                    default: Some("now()".into()),
                    nullable: false,
                    rename_from: None,
                    tags: Vec::new(),
                    zod: None,
                    storage: None,
                });
            model
                .fields
                .entry("updated_at".into())
                .or_insert(ir::FieldDef {
                    rust_type: "DateTime<Utc>".into(),
                    db_type: Some("TIMESTAMPTZ".into()),
                    default: Some("now()".into()),
                    nullable: false,
                    rename_from: None,
                    tags: Vec::new(),
                    zod: None,
                    storage: None,
                });
        }
        if model.options.soft_delete {
            model
                .fields
                .entry("deleted_at".into())
                .or_insert(ir::FieldDef {
                    rust_type: "DateTime<Utc>".into(),
                    db_type: Some("TIMESTAMPTZ".into()),
                    default: None,
                    nullable: true,
                    rename_from: None,
                    tags: Vec::new(),
                    zod: None,
                    storage: None,
                });
        }
    }
}

/// Replace any field types that match aliases with their expanded forms.
pub fn apply_type_aliases(
    ir: &mut ir::SchemaIR,
    aliases: &std::collections::HashMap<String, ir::TypeAlias>,
) {
    for model in ir.models.values_mut() {
        for field in model.fields.values_mut() {
            if let Some(alias) = aliases.get(&field.rust_type) {
                field.rust_type = alias.rust_type.clone();
                if field.db_type.is_none() {
                    field.db_type = Some(alias.db_type.clone());
                }
            }
        }
    }
}

/// Generate TypeScript interfaces and route definitions from the schema.
pub fn generate_typescript(ir: &ir::SchemaIR) -> String {
    fn map_type(name: &str) -> String {
        if let Some(inner) = name
            .strip_prefix("Option<")
            .and_then(|s| s.strip_suffix('>'))
        {
            return format!("{} | null", map_type(inner));
        }
        match name {
            "String" | "Uuid" => "string".into(),
            "i32" | "i64" | "u32" | "u64" | "usize" | "f32" | "f64" => "number".into(),
            "bool" => "boolean".into(),
            other => other.to_string(),
        }
    }

    fn map_zod(name: &str) -> String {
        if let Some(inner) = name
            .strip_prefix("Option<")
            .and_then(|s| s.strip_suffix('>'))
        {
            return format!("{}.optional()", map_zod(inner));
        }
        match name {
            "String" | "Uuid" => "z.string()".into(),
            "i32" | "i64" | "u32" | "u64" | "usize" | "f32" | "f64" => "z.number()".into(),
            "bool" => "z.boolean()".into(),
            _ => "z.any()".into(),
        }
    }

    let mut out = String::new();
    out.push_str("// -----------------------------------------------------------\n");
    out.push_str("// ⚠️ WARNING: THIS FILE IS AUTOGENERATED. DO NOT EDIT. ⚠️\n");
    out.push_str("// -----------------------------------------------------------\n\n");
    out.push_str("import { z } from 'zod';\n\n");

    // Enums
    for (name, en) in &ir.enums {
        out.push_str(&format!("export enum {} {{\n", name));
        for variant in &en.variants {
            out.push_str(&format!("    {} = \"{}\",\n", variant, variant));
        }
        out.push_str("}\n\n");
    }

    // Interfaces and Zod schemas
    for (model_name, model) in &ir.models {
        out.push_str(&format!("export interface {} {{\n", model_name));
        for (field_name, field) in &model.fields {
            let ts_type = map_type(&field.rust_type);
            let ts_type = if field.nullable {
                format!("{} | null", ts_type)
            } else {
                ts_type
            };
            out.push_str(&format!("    {}: {};\n", field_name, ts_type));
        }
        out.push_str("}\n\n");

        out.push_str(&format!(
            "export const {}Schema = z.object({{\n",
            model_name
        ));
        for (field_name, field) in &model.fields {
            let mut expr = if let Some(custom) = &field.zod {
                custom.clone()
            } else {
                map_zod(&field.rust_type)
            };
            if field.nullable {
                expr.push_str(".nullable()");
            }
            out.push_str(&format!("    {}: {},\n", field_name, expr));
        }
        out.push_str("});\n\n");
    }

    // Routes
    if !ir.routes.is_empty() {
        out.push_str("export interface Permissions { read: readonly string[]; update: readonly string[]; delete: readonly string[] }\n");
        out.push_str("export interface Route { methods: readonly string[]; path: string; authRequired: boolean; permissions: Permissions }\n");
        out.push_str(&format!("export function hasPermission(role: string, allowed: readonly string[]): boolean {{ return allowed.length === 0 || allowed.includes('{}') || allowed.includes(role); }}\n", ir.meta.auth.public_role));
        out.push_str("export function routeHasPermission(route: Route, method: string, role: string): boolean {\n");
        out.push_str("    const m = method.toUpperCase();\n");
        out.push_str("    const allowed = m === 'GET' ? route.permissions.read : m === 'DELETE' ? route.permissions.delete : ['POST','PUT','PATCH'].includes(m) ? route.permissions.update : [];\n");
        out.push_str("    return hasPermission(role, allowed);\n}\n");
        out.push_str("export const routes = {\n");
        for (name, route) in &ir.routes {
            let methods = route
                .methods
                .iter()
                .map(|m| format!("\"{}\"", m.to_uppercase()))
                .collect::<Vec<_>>()
                .join(", ");
            let read_roles = route
                .permissions
                .read
                .iter()
                .map(|r| format!("\"{}\"", r))
                .collect::<Vec<_>>()
                .join(", ");
            let update_roles = route
                .permissions
                .update
                .iter()
                .map(|r| format!("\"{}\"", r))
                .collect::<Vec<_>>()
                .join(", ");
            let delete_roles = route
                .permissions
                .delete
                .iter()
                .map(|r| format!("\"{}\"", r))
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!(
                "    {}: {{ methods: [{}], path: \"{}\", authRequired: {}, permissions: {{ read: [{}], update: [{}], delete: [{}] }} }},\n",
                name.to_lowercase(),
                methods,
                route.path,
                route.auth_required,
                read_roles,
                update_roles,
                delete_roles,
            ));
        }
        out.push_str("} as const;\n");
    }

    out
}

/// Generate GraphQL type definitions based on the schema IR.
pub fn generate_graphql_schema(ir: &ir::SchemaIR) -> String {
    fn map_type(name: &str) -> String {
        if let Some(inner) = name
            .strip_prefix("Option<")
            .and_then(|s| s.strip_suffix('>'))
        {
            return map_type(inner);
        }
        match name {
            "String" => "String".into(),
            "Uuid" => "ID".into(),
            "i32" | "i64" | "u32" | "u64" | "usize" => "Int".into(),
            "f32" | "f64" => "Float".into(),
            "bool" => "Boolean".into(),
            other => other.to_string(),
        }
    }

    let mut out = String::new();

    // Enums
    for (name, en) in &ir.enums {
        out.push_str(&format!("enum {} {{\n", name));
        for variant in &en.variants {
            out.push_str(&format!("    {}\n", variant));
        }
        out.push_str("}\n\n");
    }

    // Models
    for (model_name, model) in &ir.models {
        out.push_str(&format!("type {} {{\n", model_name));
        for (field_name, field) in &model.fields {
            let mut gql_type = map_type(&field.rust_type);
            if !field.nullable {
                gql_type.push('!');
            }
            out.push_str(&format!("    {}: {}\n", field_name, gql_type));
        }
        // Relations
        for (rel_name, rel) in &model.relations {
            let target_model = &rel.references.model;
            out.push_str(&format!("    {}: {}!\n", rel_name, target_model));
        }
        out.push_str("}\n\n");
    }

    // Input types and root operations
    for (model_name, model) in &ir.models {
        // Create input
        out.push_str(&format!("input Create{}Input {{\n", model_name));
        for (field_name, field) in &model.fields {
            if field.default.is_none() {
                let mut gql_type = map_type(&field.rust_type);
                if !field.nullable {
                    gql_type.push('!');
                }
                out.push_str(&format!("    {}: {}\n", field_name, gql_type));
            }
        }
        out.push_str("}\n\n");

        // Update input
        out.push_str(&format!("input Update{}Input {{\n", model_name));
        for (field_name, field) in &model.fields {
            if field.default.is_none() {
                let gql_type = map_type(&field.rust_type);
                out.push_str(&format!("    {}: {}\n", field_name, gql_type));
            }
        }
        out.push_str("}\n\n");
    }

    // Query type
    out.push_str("type Query {\n");
    for model_name in ir.models.keys() {
        let lower = model_name.to_lowercase();
        out.push_str(&format!("    {}(id: ID!): {}\n", lower, model_name));
        out.push_str(&format!("    list{}s: [{}!]!\n", model_name, model_name));
    }
    out.push_str("}\n\n");

    // Mutation type
    out.push_str("type Mutation {\n");
    for model_name in ir.models.keys() {
        out.push_str(&format!(
            "    create{}(input: Create{}Input!): {}!\n",
            model_name, model_name, model_name
        ));
        out.push_str(&format!(
            "    update{}(id: ID!, input: Update{}Input!): {}!\n",
            model_name, model_name, model_name
        ));
        out.push_str(&format!("    delete{}(id: ID!): Boolean!\n", model_name));
    }
    out.push_str("}\n");

    out
}

/// Infer the database backend from a connection URL.
pub fn infer_backend_from_url(url: &str) -> Option<ir::DatabaseBackend> {
    if url.starts_with("postgres") {
        Some(ir::DatabaseBackend::Postgres)
    } else if url.starts_with("mysql") {
        Some(ir::DatabaseBackend::Mysql)
    } else if url.starts_with("sqlite") || url.ends_with(".db") || url.starts_with("file:") {
        Some(ir::DatabaseBackend::Sqlite)
    } else {
        None
    }
}

/// Connect to a database URL using `sqlx::AnyPool` with the required driver
/// automatically installed. This helper relies on [`infer_backend_from_url`]
/// to determine which driver to enable.
pub async fn connect_any_pool(url: &str) -> Result<sqlx::AnyPool> {
    sqlx::any::install_default_drivers();
    // Ensure we enable the right driver for the backend, otherwise the
    // connection attempt will fail with an opaque error.
    if infer_backend_from_url(url).is_none() {
        anyhow::bail!("Unsupported database URL: {}", url);
    }
    Ok(sqlx::AnyPool::connect(url).await?)
}

/// Generate a simple fetch-based TypeScript client for all routes.
pub fn generate_ts_client(ir: &ir::SchemaIR) -> String {
    let mut out = String::new();
    out.push_str("// ------------------------------------------------------------\n");
    out.push_str("// ⚠️ WARNING: THIS FILE IS AUTOGENERATED. DO NOT EDIT. ⚠️\n");
    out.push_str("// ------------------------------------------------------------\n\n");
    out.push_str("import { routes } from './types';\n\n");

    out.push_str("async function call(route: { path: string }, method: string, body?: any) {\n");
    out.push_str("    const res = await fetch(route.path, {\n");
    out.push_str("        method,\n        headers: { 'Content-Type': 'application/json' },\n");
    out.push_str("        body: body ? JSON.stringify(body) : undefined,\n    });\n");
    out.push_str("    return res.json();\n}\n\n");

    for (name, route) in &ir.routes {
        let const_name = name.to_lowercase();
        out.push_str(&format!("export const {} = {{\n", const_name));
        for method in &route.methods {
            out.push_str(&format!(
                "    {}: (body?: any) => call(routes.{}, '{}', body),\n",
                method.to_lowercase(),
                const_name,
                method.to_uppercase()
            ));
        }
        out.push_str("};\n\n");
    }

    out
}

/// Generate SQL INSERT statements for seed data defined in the schema.
pub fn generate_seed_sql(ir: &ir::SchemaIR) -> String {
    fn format_val(v: &toml::Value) -> String {
        match v {
            toml::Value::String(s) => format!("'{}'", s.replace('\'', "''")),
            toml::Value::Integer(i) => i.to_string(),
            toml::Value::Float(f) => f.to_string(),
            toml::Value::Boolean(b) => b.to_string(),
            toml::Value::Datetime(dt) => format!("'{}'", dt),
            other => format!("'{}'", other),
        }
    }

    let mut out = String::new();
    for (model, seed) in &ir.seeds {
        for row in &seed.rows {
            let cols: Vec<_> = row.keys().cloned().collect();
            let vals: Vec<_> = row.values().map(format_val).collect();
            out.push_str(&format!(
                "INSERT INTO {} ({}) VALUES ({});\n",
                model.to_lowercase(),
                cols.join(", "),
                vals.join(", ")
            ));
        }
    }
    out
}

/// Execute an external plugin, providing the schema IR as JSON on stdin and
/// returning the plugin's stdout. The plugin must exit with status 0.
pub fn run_plugin(
    exe: &str,
    args: &[String],
    env: &std::collections::HashMap<String, String>,
    cwd: &Option<String>,
    ir: &ir::SchemaIR,
) -> anyhow::Result<String> {
    use std::process::{Command, Stdio};

    let mut cmd = Command::new(exe);
    cmd.args(args).stdin(Stdio::piped()).stdout(Stdio::piped());
    for (k, v) in env {
        cmd.env(k, v);
    }
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    let mut child = cmd.spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        if let Err(e) = serde_json::to_writer(&mut stdin, ir) {
            if !(e.is_io() && e.io_error_kind() == Some(std::io::ErrorKind::BrokenPipe)) {
                return Err(e.into());
            }
        }
    }

    let output = child.wait_with_output()?;
    if !output.status.success() {
        anyhow::bail!("plugin exited with status {}", output.status);
    }

    Ok(String::from_utf8(output.stdout)?)
}

/// Apply the seed data to a database using the provided connection pool.
pub async fn apply_seed_data(pool: &sqlx::AnyPool, ir: &ir::SchemaIR) -> anyhow::Result<()> {
    let sql = generate_seed_sql(ir);
    for stmt in sql.split(';') {
        let trimmed = stmt.trim();
        if !trimmed.is_empty() {
            sqlx::query(trimmed).execute(pool).await?;
        }
    }
    Ok(())
}

/// Copy the local schema to the given registry path.
pub fn push_schema(local_path: &str, registry_path: &str) -> anyhow::Result<()> {
    if let Some(parent) = std::path::Path::new(registry_path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::copy(local_path, registry_path)?;
    Ok(())
}

/// Retrieve the schema from the registry path to the local path.
pub fn pull_schema(registry_path: &str, local_path: &str) -> anyhow::Result<()> {
    std::fs::copy(registry_path, local_path)?;
    Ok(())
}

/// Apply any pending migrations in the given directory to the database.
/// Migration files must end with `.up.sql` and will be executed in name order.
pub async fn apply_migrations(pool: &sqlx::AnyPool, dir: &str) -> anyhow::Result<()> {
    use std::fs;
    use sha2::{Digest, Sha256};
    use std::time::Instant;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS __rustdbgen_migrations (
            name TEXT PRIMARY KEY,
            hash TEXT NOT NULL,
            applied_at TEXT NOT NULL,
            execution_time_ms INTEGER,
            success BOOLEAN NOT NULL
        )",
    )
    .execute(pool)
    .await?;

    let mut entries: Vec<_> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension().map(|ext| ext == "sql").unwrap_or(false)
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.ends_with(".up.sql") || n.ends_with(".data.sql"))
                    .unwrap_or(false)
        })
        .collect();
    entries.sort();

    for suffix in [".up.sql", ".data.sql"] {
        for path in entries.iter().filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.ends_with(suffix))
                .unwrap_or(false)
        }) {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| anyhow::anyhow!("invalid migration filename"))?;
            let sql = fs::read_to_string(path)?;
            let hash = format!("{:x}", Sha256::digest(sql.as_bytes()));
            if let Some((stored_hash,)) =
                sqlx::query_as::<_, (String,)>(
                    "SELECT hash FROM __rustdbgen_migrations WHERE name = ?"
                )
                .bind(name)
                .fetch_optional(pool)
                .await?
            {
                if stored_hash != hash {
                    anyhow::bail!("migration {} has changed after being applied", name);
                }
                continue;
            }

            let start = Instant::now();
            let mut tx = pool.begin().await?;
            for stmt in sql.split(';') {
                let trimmed = stmt.trim();
                if !trimmed.is_empty() {
                    sqlx::query(trimmed).execute(&mut *tx).await?;
                }
            }
            tx.commit().await?;
            let duration = start.elapsed().as_millis() as i64;

            sqlx::query(
                "INSERT INTO __rustdbgen_migrations (name, hash, applied_at, execution_time_ms, success) VALUES (?, ?, ?, ?, 1)"
            )
            .bind(name)
            .bind(hash)
            .bind(chrono::Utc::now().to_rfc3339())
            .bind(duration)
            .execute(pool)
            .await?;
        }
    }
    Ok(())
}

/// Create a timestamped data migration file with placeholder contents.
pub fn create_data_migration(name: &str) -> anyhow::Result<String> {
    use std::fs;
    let ts = chrono::Utc::now().format("%Y%m%d%H%M%S");
    fs::create_dir_all("migrations")?;
    let path = format!("migrations/{}_{}.data.sql", ts, name);
    let tmpl = "BEGIN;\n\n-- Write data migration SQL here\n\nCOMMIT;\n";
    fs::write(&path, tmpl)?;
    Ok(path)
}

/// Serve a minimal web editor for the schema.
pub async fn serve_editor(addr: &str, schema_path: &str) -> anyhow::Result<()> {
    use axum::{routing::{get, post}, Router, extract::State, response::Html, http::StatusCode};
    use std::sync::Arc;
    use std::path::PathBuf;

    let schema = Arc::new(PathBuf::from(schema_path));

    async fn index() -> Html<&'static str> {
        Html(include_str!("../web/editor.html"))
    }

    async fn get_schema(State(p): State<Arc<PathBuf>>) -> Result<String, StatusCode> {
        tokio::fs::read_to_string(&*p).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }

    async fn save_schema(State(p): State<Arc<PathBuf>>, body: String) -> Result<StatusCode, StatusCode> {
        tokio::fs::write(&*p, body).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(StatusCode::OK)
    }

    let app = Router::new()
        .route("/", get(index))
        .route("/schema", get(get_schema).post(save_schema))
        .with_state(schema);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}
