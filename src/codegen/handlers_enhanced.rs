use crate::ir;
use std::collections::HashMap;

pub fn generate_enhanced_crud_impls(ir: &ir::SchemaIR) -> String {
    let mut out = String::new();
    let tracing_enabled = matches!(ir.meta.observability_provider.as_deref(), Some("tracing"));
    let (executor_trait, qb_type, placeholder_fn) = get_db_config(&ir.meta.db_backend);

    // Add imports at the top
    out.push_str("use crate::generated::*;\n");
    out.push_str("use crate::generated::executor::*;\n");
    out.push_str("use std::collections::HashMap;\n\n");

    let mut models: Vec<_> = ir.models.iter().collect();
    models.sort_by(|a, b| a.0.cmp(b.0));

    for (model_name, model) in models {
        out.push_str(&generate_enhanced_model_crud(
            model_name,
            model,
            ir,
            tracing_enabled,
            executor_trait,
            qb_type,
            placeholder_fn,
        ));
    }

    out
}

fn generate_enhanced_model_crud(
    model_name: &str,
    model: &ir::ModelDef,
    ir: &ir::SchemaIR,
    tracing_enabled: bool,
    executor_trait: &str,
    qb_type: &str,
    placeholder_fn: fn(usize) -> String,
) -> String {
    let mut out = String::new();

    out.push_str(&format!("impl {} {{\n", model_name));

    // Generate enhanced CRUD methods
    out.push_str(&generate_enhanced_create(model_name, model, ir, tracing_enabled, executor_trait, placeholder_fn));
    out.push_str(&generate_enhanced_find(model_name, model, ir, tracing_enabled, executor_trait, placeholder_fn));
    out.push_str(&generate_enhanced_update(model_name, model, ir, tracing_enabled, executor_trait, qb_type, placeholder_fn));
    out.push_str(&generate_enhanced_delete(model_name, model, ir, tracing_enabled, executor_trait, placeholder_fn));
    out.push_str(&generate_enhanced_list(model_name, model, ir, tracing_enabled, executor_trait, qb_type));
    
    // Generate relational helpers
    out.push_str(&generate_relational_helpers(model_name, model, ir, tracing_enabled, executor_trait));
    
    // Generate eager loading helpers
    out.push_str(&generate_eager_loading_helpers(model_name, model, ir, tracing_enabled, executor_trait));

    out.push_str("}\n\n");

    out
}

fn generate_enhanced_create(
    model_name: &str,
    model: &ir::ModelDef,
    ir: &ir::SchemaIR,
    tracing_enabled: bool,
    executor_trait: &str,
    placeholder_fn: fn(usize) -> String,
) -> String {
    let mut out = String::new();

    if tracing_enabled {
        out.push_str("    #[tracing::instrument]\n");
    }

    // Generate create function signature with executor trait
    let mut create_sig = format!("    pub async fn create<'c, E>(executor: E, item: &{}New", model_name);
    
    // Only add user_id parameter if model is owned by someone (not for User itself)
    if let Some(owner_model) = &model.owned_by {
        let owner_fk_name = format!("{}_id", owner_model.to_lowercase());
        create_sig.push_str(&format!(", {}: uuid::Uuid", owner_fk_name));
    }
    
    create_sig.push_str(&format!(") -> Result<{}, {}CreateError>\n", model_name, model_name));
    create_sig.push_str("    where\n");
    create_sig.push_str(&format!("        E: {},\n", executor_trait));
    create_sig.push_str("    {\n");
    out.push_str(&create_sig);

    // Get field constraints and relations
    let mut uc_names: Vec<_> = model.unique_constraints.keys().collect();
    uc_names.sort();
    let mut rel_names: Vec<_> = model.relations.iter().filter(|(_, rel)| rel.on != "id").map(|(k, _)| k.clone()).collect();
    rel_names.sort();

    let mut field_names: Vec<String> = model
        .fields
        .iter()
        .filter(|(name, f)| f.default.is_none() && *name != "deleted_at")
        .map(|(n, _)| n.to_string())
        .collect();
    
    field_names.sort();

    match ir.meta.db_backend {
        ir::DatabaseBackend::Postgres | ir::DatabaseBackend::Sqlite => {
            if field_names.is_empty() {
                out.push_str(&format!(
                    "        let res = sqlx::query_as::<_, {}>(\"INSERT INTO {} DEFAULT VALUES RETURNING *\")\n            .fetch_one(executor)\n            .await;\n",
                    model_name,
                    model_name.to_lowercase()
                ));
            } else {
                let cols = field_names.join(", ");
                let binds: Vec<String> = (1..=field_names.len()).map(placeholder_fn).collect();
                let placeholders = binds.join(", ");
                out.push_str(&format!(
                    "        let res = sqlx::query_as::<_, {}>(\"INSERT INTO {} ({}) VALUES ({}) RETURNING *\")\n",
                    model_name,
                    model_name.to_lowercase(),
                    cols,
                    placeholders
                ));
                for name in &field_names {
                    if let Some(field_def) = model.fields.get(name) {
                        // Handle owner foreign key field
                        if let Some(owner_model) = &model.owned_by {
                            let owner_fk = format!("{}_id", owner_model.to_lowercase());
                            if name == &owner_fk {
                                out.push_str(&format!("            .bind(&{})\n", owner_fk));
                                continue;
                            }
                        }
                        
                        // Handle password fields - hash the password
                        if field_def.tags.contains(&"password".to_string()) {
                            out.push_str("            .bind(&{\n");
                            out.push_str("                use argon2::{Argon2, PasswordHasher};\n");
                            out.push_str("                use password_hash::{rand_core::OsRng, SaltString};\n");
                            out.push_str("                let salt = SaltString::generate(&mut OsRng);\n");
                            out.push_str("                let argon2 = Argon2::default();\n");
                            out.push_str("                argon2.hash_password(item.password.as_bytes(), &salt)\n");
                            out.push_str("                    .map_err(|_| sqlx::Error::Protocol(\"Password hashing failed\".into()))?\n");
                            out.push_str("                    .to_string()\n");
                            out.push_str("            })\n");
                            continue;
                        }
                        
                        let name_escaped = escape_rust_keyword(name);
                        out.push_str(&format!("            .bind(&item.{})\n", name_escaped));
                    }
                }
                out.push_str("            .fetch_one(executor)\n            .await;\n");
            }
            
            // Error handling
            out.push_str("        match res {\n            Ok(v) => Ok(v),\n            Err(e) => {\n                if let sqlx::Error::Database(db_err) = &e {\n                    if let Some(c) = db_err.constraint() {\n");
            for uc_name in uc_names.iter() {
                let var = pascal_case(uc_name);
                out.push_str(&format!(
                    "                        if c == \"{}\" {{ return Err({}CreateError::{}); }}\n",
                    uc_name, model_name, var
                ));
            }
            for rel_name in rel_names.iter() {
                let var = format!("{}Fk", pascal_case(rel_name));
                out.push_str(&format!(
                    "                        if c == \"{}\" {{ return Err({}CreateError::{}); }}\n",
                    rel_name, model_name, var
                ));
            }
            out.push_str("                    }\n                }\n                Err(");
            out.push_str(&format!(
                "{}CreateError::Database(e))\n            }}\n        }}\n    }}\n\n",
                model_name
            ));
        }
        ir::DatabaseBackend::Mysql => {
            // MySQL implementation (similar to original but with executor)
            if field_names.is_empty() {
                out.push_str(&format!(
                    "        sqlx::query(\"INSERT INTO {} DEFAULT VALUES\")\n            .execute(executor)\n            .await?;\n",
                    model_name.to_lowercase()
                ));
            } else {
                let cols = field_names.join(", ");
                let binds: Vec<String> = (1..=field_names.len()).map(placeholder_fn).collect();
                let placeholders = binds.join(", ");
                out.push_str(&format!(
                    "        sqlx::query(\"INSERT INTO {} ({}) VALUES ({})\")\n",
                    model_name.to_lowercase(),
                    cols,
                    placeholders
                ));
                for name in &field_names {
                    if let Some(field_def) = model.fields.get(name) {
                        // Handle owner foreign key field
                        if let Some(owner_model) = &model.owned_by {
                            let owner_fk = format!("{}_id", owner_model.to_lowercase());
                            if name == &owner_fk {
                                out.push_str(&format!("            .bind(&{})\n", owner_fk));
                                continue;
                            }
                        }
                        
                        // Handle password fields - hash the password
                        if field_def.tags.contains(&"password".to_string()) {
                            out.push_str("            .bind(&{\n");
                            out.push_str("                use argon2::{Argon2, PasswordHasher};\n");
                            out.push_str("                use password_hash::{rand_core::OsRng, SaltString};\n");
                            out.push_str("                let salt = SaltString::generate(&mut OsRng);\n");
                            out.push_str("                let argon2 = Argon2::default();\n");
                            out.push_str("                argon2.hash_password(item.password.as_bytes(), &salt)\n");
                            out.push_str("                    .map_err(|_| sqlx::Error::Protocol(\"Password hashing failed\".into()))?\n");
                            out.push_str("                    .to_string()\n");
                            out.push_str("            })\n");
                            continue;
                        }
                        
                        let name_escaped = escape_rust_keyword(name);
                        out.push_str(&format!("            .bind(&item.{})\n", name_escaped));
                    }
                }
                out.push_str("            .execute(executor)\n            .await?;\n");
            }
            out.push_str(&format!(
                "        let last_id: u64 = sqlx::query_scalar(\"SELECT LAST_INSERT_ID()\")\n            .fetch_one(executor)\n            .await?;\n        let res = sqlx::query_as::<_, {}>(\"SELECT * FROM {} WHERE id = ?\")\n            .bind(last_id)\n            .fetch_one(executor)\n            .await;\n",
                model_name,
                model_name.to_lowercase()
            ));
            
            // Error handling for MySQL
            out.push_str("        match res {\n            Ok(v) => Ok(v),\n            Err(e) => {\n                if let sqlx::Error::Database(db_err) = &e {\n                    if let Some(c) = db_err.constraint() {\n");
            for uc_name in uc_names.iter() {
                let var = pascal_case(uc_name);
                out.push_str(&format!(
                    "                        if c == \"{}\" {{ return Err({}CreateError::{}); }}\n",
                    uc_name, model_name, var
                ));
            }
            for rel_name in rel_names.iter() {
                let var = format!("{}Fk", pascal_case(rel_name));
                out.push_str(&format!(
                    "                        if c == \"{}\" {{ return Err({}CreateError::{}); }}\n",
                    rel_name, model_name, var
                ));
            }
            out.push_str("                    }\n                }\n                Err(");
            out.push_str(&format!(
                "{}CreateError::Database(e))\n            }}\n        }}\n",
                model_name
            ));
        }
    }

    out
}

fn generate_enhanced_find(
    model_name: &str,
    model: &ir::ModelDef,
    ir: &ir::SchemaIR,
    tracing_enabled: bool,
    executor_trait: &str,
    placeholder_fn: fn(usize) -> String,
) -> String {
    let mut out = String::new();

    if tracing_enabled {
        out.push_str("    #[tracing::instrument]\n");
    }
    
    // Generate find function signature with executor trait
    if model.owned_by.is_some() {
        out.push_str(&format!(
            "    pub async fn find<'c, E>(executor: E, id: uuid::Uuid, user_id: uuid::Uuid) -> Result<{}, sqlx::Error>\n",
            model_name
        ));
    } else {
        out.push_str(&format!(
            "    pub async fn find<'c, E>(executor: E, id: uuid::Uuid) -> Result<{}, sqlx::Error>\n",
            model_name
        ));
    }
    
    out.push_str("    where\n");
    out.push_str(&format!("        E: {},\n", executor_trait));
    out.push_str("    {\n");
    
    // Generate query based on ownership
    if model.owned_by.is_some() {
        let mut find_query = format!("SELECT * FROM {} WHERE id = {} AND user_id = {}", 
            model_name.to_lowercase(), placeholder_fn(1), placeholder_fn(2));
        if model.options.soft_delete {
            find_query.push_str(" AND deleted_at IS NULL");
        }
        out.push_str(&format!(
            "        sqlx::query_as::<_, {}>(\"{}\")\n            .bind(id)\n            .bind(user_id)\n            .fetch_one(executor)\n            .await\n    }}\n\n",
            model_name, find_query
        ));
    } else {
        let mut find_query = format!("SELECT * FROM {} WHERE id = {}", model_name.to_lowercase(), placeholder_fn(1));
        if model.options.soft_delete {
            find_query.push_str(" AND deleted_at IS NULL");
        }
        out.push_str(&format!(
            "        sqlx::query_as::<_, {}>(\"{}\")\n            .bind(id)\n            .fetch_one(executor)\n            .await\n    }}\n\n",
            model_name, find_query
        ));
    }

    out
}

fn generate_enhanced_update(
    model_name: &str,
    model: &ir::ModelDef,
    ir: &ir::SchemaIR,
    tracing_enabled: bool,
    executor_trait: &str,
    qb_type: &str,
    placeholder_fn: fn(usize) -> String,
) -> String {
    let mut out = String::new();

    if tracing_enabled {
        out.push_str("    #[tracing::instrument]\n");
    }
    
    // Generate update function signature with executor trait
    if model.owned_by.is_some() {
        out.push_str(&format!(
            "    pub async fn update<'c, E>(executor: E, id: uuid::Uuid, user_id: uuid::Uuid, item: &{}Update) -> Result<{}, sqlx::Error>\n",
            model_name, model_name
        ));
    } else {
        out.push_str(&format!(
            "    pub async fn update<'c, E>(executor: E, id: uuid::Uuid, item: &{}Update) -> Result<{}, sqlx::Error>\n",
            model_name, model_name
        ));
    }
    
    out.push_str("    where\n");
    out.push_str(&format!("        E: {},\n", executor_trait));
    out.push_str("    {\n");
    
    out.push_str(&format!(
        "        let mut qb = sqlx::QueryBuilder::<{}>::new(\"UPDATE {} SET \" );\n",
        qb_type,
        model_name.to_lowercase()
    ));
    out.push_str("        let mut has_updates = false;\n");
    let mut fields_vec: Vec<_> = model.fields.iter().collect();
    fields_vec.sort_by(|a, b| a.0.cmp(b.0));
    out.push_str("        let mut separated = qb.separated(\", \");\n");
    
    for (field_name, field) in fields_vec {
        if field.default.is_none() {
            let field_name_escaped = escape_rust_keyword(field_name);
            
            // Handle password fields specially - hash the password
            if field.tags.contains(&"password".to_string()) {
                if field_name == "password_hash" {
                    out.push_str("        if let Some(password) = &item.password {\n");
                    out.push_str("            use argon2::{Argon2, PasswordHasher};\n");
                    out.push_str("            use password_hash::{rand_core::OsRng, SaltString};\n");
                    out.push_str("            let salt = SaltString::generate(&mut OsRng);\n");
                    out.push_str("            let argon2 = Argon2::default();\n");
                    out.push_str("            let hashed = argon2.hash_password(password.as_bytes(), &salt)\n");
                    out.push_str("                .map_err(|_| sqlx::Error::Protocol(\"Password hashing failed\".into()))?\n");
                    out.push_str("                .to_string();\n");
                    out.push_str(&format!(
                        "            separated.push(\"{} = \").push_bind(hashed); has_updates = true;\n",
                        field_name
                    ));
                    out.push_str("        }\n");
                } else {
                    out.push_str(&format!(
                        "        if let Some(value) = &item.{} {{ separated.push(\"{} = \").push_bind(value); has_updates = true; }}\n",
                        field_name_escaped, field_name
                    ));
                }
            } else {
                out.push_str(&format!(
                    "        if let Some(value) = &item.{} {{ separated.push(\"{} = \").push_bind(value); has_updates = true; }}\n",
                    field_name_escaped, field_name
                ));
            }
        }
    }
    
    out.push_str("        if !has_updates {\n");
    if model.owned_by.is_some() {
        out.push_str("            // Can't call Self::find with a generic executor easily, so we query directly\n");
        out.push_str(&format!("            return sqlx::query_as(\"SELECT * FROM {} WHERE id = {} AND user_id = {}", 
            model_name.to_lowercase(), placeholder_fn(1), placeholder_fn(2)));
        if model.options.soft_delete {
            out.push_str(" AND deleted_at IS NULL");
        }
        out.push_str("\")\n                .bind(id).bind(user_id).fetch_one(executor).await;\n");
    } else {
        out.push_str(&format!("            return sqlx::query_as(\"SELECT * FROM {} WHERE id = {}", 
            model_name.to_lowercase(), placeholder_fn(1)));
        if model.options.soft_delete {
            out.push_str(" AND deleted_at IS NULL");
        }
        out.push_str("\")\n                .bind(id).fetch_one(executor).await;\n");
    }
    out.push_str("        }\n");
    
    match ir.meta.db_backend {
        ir::DatabaseBackend::Postgres | ir::DatabaseBackend::Sqlite => {
            if model.owned_by.is_some() {
                out.push_str(&format!(
                    "        qb.push(\" WHERE id = \" ).push_bind(id).push(\" AND user_id = \").push_bind(user_id).push(\" RETURNING *\");\n        let query = qb.build_query_as::<{}>();\n        query.fetch_one(executor).await\n    }}\n\n",
                    model_name,
                ));
            } else {
                out.push_str(&format!(
                    "        qb.push(\" WHERE id = \" ).push_bind(id).push(\" RETURNING *\");\n        let query = qb.build_query_as::<{}>();\n        query.fetch_one(executor).await\n    }}\n\n",
                    model_name,
                ));
            }
        }
        ir::DatabaseBackend::Mysql => {
            // MySQL implementation (similar pattern)
            if model.owned_by.is_some() {
                out.push_str(&format!(
                    "        qb.push(\" WHERE id = \" ).push_bind(id).push(\" AND user_id = \").push_bind(user_id);\n        let query = qb.build();\n        query.execute(executor).await?;\n        sqlx::query_as::<_, {}>(\"SELECT * FROM {} WHERE id = ? AND user_id = ?\")\n            .bind(id)\n            .bind(user_id)\n            .fetch_one(executor)\n            .await\n    }}\n\n",
                    model_name,
                    model_name.to_lowercase(),
                ));
            } else {
                out.push_str(&format!(
                    "        qb.push(\" WHERE id = \" ).push_bind(id);\n        let query = qb.build();\n        query.execute(executor).await?;\n        sqlx::query_as::<_, {}>(\"SELECT * FROM {} WHERE id = ?\")\n            .bind(id)\n            .fetch_one(executor)\n            .await\n    }}\n\n",
                    model_name,
                    model_name.to_lowercase(),
                ));
            }
        }
    }

    out
}

fn generate_enhanced_delete(
    model_name: &str,
    model: &ir::ModelDef,
    ir: &ir::SchemaIR,
    tracing_enabled: bool,
    executor_trait: &str,
    placeholder_fn: fn(usize) -> String,
) -> String {
    let mut out = String::new();

    if tracing_enabled {
        out.push_str("    #[tracing::instrument]\n");
    }
    
    // Generate delete function signature with executor trait
    if model.owned_by.is_some() {
        out.push_str(&format!(
            "    pub async fn delete<'c, E>(executor: E, id: uuid::Uuid, user_id: uuid::Uuid) -> Result<u64, sqlx::Error>\n"
        ));
    } else {
        out.push_str(&format!(
            "    pub async fn delete<'c, E>(executor: E, id: uuid::Uuid) -> Result<u64, sqlx::Error>\n"
        ));
    }
    
    out.push_str("    where\n");
    out.push_str(&format!("        E: {},\n", executor_trait));
    out.push_str("    {\n");
    
    if model.owned_by.is_some() {
        if model.options.soft_delete {
            out.push_str(&format!(
                "        let res = sqlx::query(\"UPDATE {} SET deleted_at = now() WHERE id = {} AND user_id = {}\")\n            .bind(id)\n            .bind(user_id)\n            .execute(executor)\n            .await?;\n        Ok(res.rows_affected())\n    }}\n\n",
                model_name.to_lowercase(), placeholder_fn(1), placeholder_fn(2)
            ));
        } else {
            out.push_str(&format!(
                "        let res = sqlx::query(\"DELETE FROM {} WHERE id = {} AND user_id = {}\")\n            .bind(id)\n            .bind(user_id)\n            .execute(executor)\n            .await?;\n        Ok(res.rows_affected())\n    }}\n\n",
                model_name.to_lowercase(), placeholder_fn(1), placeholder_fn(2)
            ));
        }
    } else {
        if model.options.soft_delete {
            out.push_str(&format!(
                "        let res = sqlx::query(\"UPDATE {} SET deleted_at = now() WHERE id = {}\")\n            .bind(id)\n            .execute(executor)\n            .await?;\n        Ok(res.rows_affected())\n    }}\n\n",
                model_name.to_lowercase(), placeholder_fn(1)
            ));
        } else {
            out.push_str(&format!(
                "        let res = sqlx::query(\"DELETE FROM {} WHERE id = {}\")\n            .bind(id)\n            .execute(executor)\n            .await?;\n        Ok(res.rows_affected())\n    }}\n\n",
                model_name.to_lowercase(), placeholder_fn(1)
            ));
        }
    }

    out
}

fn generate_enhanced_list(
    model_name: &str,
    model: &ir::ModelDef,
    ir: &ir::SchemaIR,
    tracing_enabled: bool,
    executor_trait: &str,
    qb_type: &str,
) -> String {
    let mut out = String::new();

    if tracing_enabled {
        out.push_str("    #[tracing::instrument]\n");
    }
    
    // Generate list function signature with executor trait
    if model.owned_by.is_some() {
        out.push_str(&format!(
            "    pub async fn list<'c, E>(executor: E, user_id: uuid::Uuid, pagination: Option<Pagination>) -> Result<Vec<{}>, sqlx::Error>\n",
            model_name
        ));
    } else {
        out.push_str(&format!(
            "    pub async fn list<'c, E>(executor: E, pagination: Option<Pagination>) -> Result<Vec<{}>, sqlx::Error>\n",
            model_name
        ));
    }
    
    out.push_str("    where\n");
    out.push_str(&format!("        E: {},\n", executor_trait));
    out.push_str("    {\n");
    
    out.push_str(&format!(
        "        let mut qb = sqlx::QueryBuilder::<{}>::new(\"SELECT * FROM {}\");\n",
        qb_type,
        model_name.to_lowercase()
    ));
    
    if model.owned_by.is_some() {
        if model.options.soft_delete {
            out.push_str("        qb.push(\" WHERE deleted_at IS NULL AND user_id = \").push_bind(user_id);\n");
        } else {
            out.push_str("        qb.push(\" WHERE user_id = \").push_bind(user_id);\n");
        }
    } else {
        if model.options.soft_delete {
            out.push_str("        qb.push(\" WHERE deleted_at IS NULL\");\n");
        }
    }
    
    out.push_str("        if let Some(p) = pagination {\n");
    out.push_str("            qb.push(\" LIMIT \" ).push_bind(p.limit);\n");
    out.push_str("            qb.push(\" OFFSET \" ).push_bind(p.offset);\n");
    out.push_str("        }\n");
    out.push_str(&format!(
        "        qb.build_query_as::<{}>().fetch_all(executor).await\n    }}\n\n",
        model_name
    ));

    out
}

fn generate_relational_helpers(
    model_name: &str,
    model: &ir::ModelDef,
    ir: &ir::SchemaIR,
    tracing_enabled: bool,
    executor_trait: &str,
) -> String {
    let mut out = String::new();

    // Generate "belongs to" relationships (foreign keys this model has)
    for (field_name, field) in &model.fields {
        if field_name.ends_with("_id") && field.rust_type == "uuid::Uuid" {
            let target_model = field_name.strip_suffix("_id").unwrap();
            let target_model_pascal = pascal_case(target_model);
            
            // Check if this target model actually exists in the schema
            if ir.models.contains_key(&target_model_pascal) {
                out.push_str("    // --- Relational Helper: belongs to ---\n");
                if tracing_enabled {
                    out.push_str("    #[tracing::instrument]\n");
                }
                out.push_str(&format!(
                    "    pub async fn find_{}<'c, E>(&self, executor: E) -> Result<{}, sqlx::Error>\n",
                    target_model, target_model_pascal
                ));
                out.push_str("    where\n");
                out.push_str(&format!("        E: {},\n", executor_trait));
                out.push_str("    {\n");
                out.push_str(&format!(
                    "        {}::find(executor, self.{}).await\n",
                    target_model_pascal, field_name
                ));
                out.push_str("    }\n\n");
            }
        }
    }

    // Generate "has many" relationships (other models that reference this model)
    for (other_model_name, other_model) in &ir.models {
        if other_model_name == model_name {
            continue; // Skip self
        }
        
        // Check if other model has a foreign key to this model
        let expected_fk = format!("{}_id", model_name.to_lowercase());
        if other_model.fields.contains_key(&expected_fk) {
            out.push_str("    // --- Relational Helper: has many ---\n");
            if tracing_enabled {
                out.push_str("    #[tracing::instrument]\n");
            }
            
            let other_model_plural = format!("{}s", other_model_name.to_lowercase()); // Simple pluralization
            out.push_str(&format!(
                "    pub async fn find_{}<'c, E>(&self, executor: E, pagination: Option<Pagination>) -> Result<Vec<{}>, sqlx::Error>\n",
                other_model_plural, other_model_name
            ));
            out.push_str("    where\n");
            out.push_str(&format!("        E: {},\n", executor_trait));
            out.push_str("    {\n");
            
            // Generate appropriate list call based on whether the other model is owned
            if other_model.owned_by.is_some() {
                // This is tricky - we need to figure out if we can call the list method
                // For now, let's create a custom query
                let qb_type = match ir.meta.db_backend {
                    ir::DatabaseBackend::Postgres => "sqlx::Postgres",
                    ir::DatabaseBackend::Mysql => "sqlx::MySql",
                    ir::DatabaseBackend::Sqlite => "sqlx::Sqlite",
                };
                
                out.push_str(&format!(
                    "        let mut qb = sqlx::QueryBuilder::<{}>::new(\"SELECT * FROM {}\");\n",
                    qb_type, other_model_name.to_lowercase()
                ));
                
                let mut where_conditions = Vec::new();
                if other_model.options.soft_delete {
                    where_conditions.push("deleted_at IS NULL".to_string());
                }
                where_conditions.push(format!("{} = ", expected_fk));
                
                if !where_conditions.is_empty() {
                    out.push_str("        qb.push(\" WHERE \");\n");
                    for (i, condition) in where_conditions.iter().enumerate() {
                        if i > 0 {
                            out.push_str("        qb.push(\" AND \");\n");
                        }
                        if condition.ends_with(" = ") {
                            out.push_str(&format!("        qb.push(\"{}\").push_bind(self.id);\n", condition));
                        } else {
                            out.push_str(&format!("        qb.push(\"{}\");\n", condition));
                        }
                    }
                }
                
                out.push_str("        if let Some(p) = pagination {\n");
                out.push_str("            qb.push(\" LIMIT \" ).push_bind(p.limit);\n");
                out.push_str("            qb.push(\" OFFSET \" ).push_bind(p.offset);\n");
                out.push_str("        }\n");
                out.push_str(&format!(
                    "        qb.build_query_as::<{}>().fetch_all(executor).await\n",
                    other_model_name
                ));
            } else {
                // For non-owned models, use a direct query since list_by methods may not exist
                let qb_type = match ir.meta.db_backend {
                    ir::DatabaseBackend::Postgres => "sqlx::Postgres",
                    ir::DatabaseBackend::Mysql => "sqlx::MySql", 
                    ir::DatabaseBackend::Sqlite => "sqlx::Sqlite",
                };
                
                out.push_str(&format!(
                    "        let mut qb = sqlx::QueryBuilder::<{}>::new(\"SELECT * FROM {}\");\n",
                    qb_type, other_model_name.to_lowercase()
                ));
                
                let mut where_conditions = Vec::new();
                if other_model.options.soft_delete {
                    where_conditions.push("deleted_at IS NULL".to_string());
                }
                where_conditions.push(format!("{} = ", expected_fk));
                
                if !where_conditions.is_empty() {
                    out.push_str("        qb.push(\" WHERE \");\n");
                    for (i, condition) in where_conditions.iter().enumerate() {
                        if i > 0 {
                            out.push_str("        qb.push(\" AND \");\n");
                        }
                        if condition.ends_with(" = ") {
                            out.push_str(&format!("        qb.push(\"{}\").push_bind(self.id);\n", condition));
                        } else {
                            out.push_str(&format!("        qb.push(\"{}\");\n", condition));
                        }
                    }
                }
                
                out.push_str("        if let Some(p) = pagination {\n");
                out.push_str("            qb.push(\" LIMIT \" ).push_bind(p.limit);\n");
                out.push_str("            qb.push(\" OFFSET \" ).push_bind(p.offset);\n");
                out.push_str("        }\n");
                out.push_str(&format!(
                    "        qb.build_query_as::<{}>().fetch_all(executor).await\n",
                    other_model_name
                ));
            }
            
            out.push_str("    }\n\n");
        }
    }

    out
}

fn generate_eager_loading_helpers(
    model_name: &str,
    model: &ir::ModelDef,
    ir: &ir::SchemaIR,
    tracing_enabled: bool,
    executor_trait: &str,
) -> String {
    let mut out = String::new();

    // Generate find_by_ids helper
    out.push_str("    // --- Eager Loading Helper ---\n");
    if tracing_enabled {
        out.push_str("    #[tracing::instrument]\n");
    }
    out.push_str(&format!(
        "    pub async fn find_by_ids<'c, E>(executor: E, ids: &[uuid::Uuid]) -> Result<Vec<{}>, sqlx::Error>\n",
        model_name
    ));
    out.push_str("    where\n");
    out.push_str(&format!("        E: {},\n", executor_trait));
    out.push_str("    {\n");
    
    let mut query = format!("SELECT * FROM {} WHERE id = ANY($1)", model_name.to_lowercase());
    if model.options.soft_delete {
        query.push_str(" AND deleted_at IS NULL");
    }
    
    out.push_str(&format!(
        "        sqlx::query_as(\"{}\")\n            .bind(ids)\n            .fetch_all(executor)\n            .await\n    }}\n\n",
        query
    ));

    // Generate eager loading helpers for related models
    for (field_name, field) in &model.fields {
        if field_name.ends_with("_id") && field.rust_type == "uuid::Uuid" {
            let target_model = field_name.strip_suffix("_id").unwrap();
            let target_model_pascal = pascal_case(target_model);
            
            // Check if this target model actually exists in the schema
            if ir.models.contains_key(&target_model_pascal) {
                if tracing_enabled {
                    out.push_str("    #[tracing::instrument]\n");
                }
                out.push_str(&format!(
                    "    pub async fn eager_load_{}<'c, E>(executor: E, items: &[{}]) -> Result<HashMap<uuid::Uuid, {}>, sqlx::Error>\n",
                    target_model_pascal.to_lowercase(), model_name, target_model_pascal
                ));
                out.push_str("    where\n");
                out.push_str(&format!("        E: {},\n", executor_trait));
                out.push_str("    {\n");
                out.push_str(&format!(
                    "        let ids: Vec<_> = items.iter().map(|item| item.{}).collect();\n",
                    field_name
                ));
                out.push_str(&format!(
                    "        let results = {}::find_by_ids(executor, &ids).await?;\n",
                    target_model_pascal
                ));
                out.push_str("        Ok(results.into_iter().map(|item| (item.id, item)).collect())\n");
                out.push_str("    }\n\n");
            }
        }
    }

    out
}

fn get_db_config(backend: &ir::DatabaseBackend) -> (&str, &str, fn(usize) -> String) {
    match backend {
        ir::DatabaseBackend::Postgres => ("PgExecutor<'c>", "sqlx::Postgres", |i| format!("${}", i)),
        ir::DatabaseBackend::Mysql => ("MySqlExecutor<'c>", "sqlx::MySql", |_| "?".into()),
        ir::DatabaseBackend::Sqlite => ("SqliteExecutor<'c>", "sqlx::Sqlite", |_| "?".into()),
    }
}

fn pascal_case(s: &str) -> String {
    s.split('_')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let mut c = p.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_ascii_uppercase().to_string() + c.as_str(),
            }
        })
        .collect()
}

const RUST_KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where", "while", "async", "await", "dyn", "abstract", "become", "box", "do", "final", "macro", "override", "priv", "try", "typeof", "unsized", "virtual", "yield",
];

fn escape_rust_keyword(name: &str) -> String {
    if RUST_KEYWORDS.contains(&name) {
        format!("r#{}", name)
    } else {
        name.to_string()
    }
}