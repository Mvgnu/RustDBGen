use crate::ir;

pub fn generate_crud_impls(ir: &ir::SchemaIR) -> String {
    let mut out = String::new();
    let tracing_enabled = matches!(ir.meta.observability_provider.as_deref(), Some("tracing"));
    let (pool_type, qb_type, placeholder_fn) = get_db_config(&ir.meta.db_backend);

    // Add imports at the top
    out.push_str("use crate::generated::*;\n\n");

    let mut models: Vec<_> = ir.models.iter().collect();
    models.sort_by(|a, b| a.0.cmp(b.0));

    for (model_name, model) in models {
        out.push_str(&generate_model_crud(
            model_name,
            model,
            ir,
            tracing_enabled,
            pool_type,
            qb_type,
            placeholder_fn,
        ));
    }

    out
}

fn generate_model_crud(
    model_name: &str,
    model: &ir::ModelDef,
    ir: &ir::SchemaIR,
    tracing_enabled: bool,
    pool_type: &str,
    qb_type: &str,
    placeholder_fn: fn(usize) -> String,
) -> String {
    let mut out = String::new();

    out.push_str(&format!("impl {} {{\n", model_name));

    // create
    if tracing_enabled {
        out.push_str("    #[tracing::instrument]\n");
    }
    
    // Generate create function signature based on ownership
    let mut create_sig = format!("    pub async fn create(pool: &{}, item: &{}New", pool_type, model_name);
    // Only add user_id parameter if model is owned by someone (not for User itself)
    if let Some(owner_model) = &model.owned_by {
        let owner_fk_name = format!("{}_id", owner_model.to_lowercase());
        create_sig.push_str(&format!(", {}: uuid::Uuid", owner_fk_name));
    }
    create_sig.push_str(&format!(") -> Result<{}, {}CreateError> {{\n", model_name, model_name));
    out.push_str(&create_sig);

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
                    "        let res = sqlx::query_as::<_, {}>(\"INSERT INTO {} DEFAULT VALUES RETURNING *\")\n            .fetch_one(pool)\n            .await;\n",
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
                    // Check if this field exists in the model
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
                out.push_str("            .fetch_one(pool)\n            .await;\n");
            }
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
                "{}CreateError::Database(e))\n            }}\n        }}\n    }}\n",
                model_name
            ));
        }
        ir::DatabaseBackend::Mysql => {
            if field_names.is_empty() {
                out.push_str(&format!(
                    "        sqlx::query(\"INSERT INTO {} DEFAULT VALUES\")\n            .execute(pool)\n            .await?;\n",
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
                    let name = escape_rust_keyword(name);
                    out.push_str(&format!("            .bind(&item.{})\n", name));
                }
                out.push_str("            .execute(pool)\n            .await?;\n");
            }
            out.push_str(&format!(
                "        let last_id: u64 = sqlx::query_scalar(\"SELECT LAST_INSERT_ID()\")\n            .fetch_one(pool)\n            .await?;\n        let res = sqlx::query_as::<_, {}>(\"SELECT * FROM {} WHERE id = ?\")\n            .bind(last_id)\n            .fetch_one(pool)\n            .await;\n",
                model_name,
                model_name.to_lowercase()
            ));
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
                "{}CreateError::Database(e))\n            }}\n        }}\n    }}\n",
                model_name
            ));
        }
    }
    out.push_str("\n");

    // find
    if tracing_enabled {
        out.push_str("    #[tracing::instrument]\n");
    }
    
    // Generate find function signature based on ownership
    if model.owned_by.is_some() {
        out.push_str(&format!(
            "    pub async fn find(pool: &{}, id: uuid::Uuid, user_id: uuid::Uuid) -> Result<{}, sqlx::Error> {{\n",
            pool_type, model_name
        ));
        let mut find_query = format!("SELECT * FROM {} WHERE id = {} AND user_id = {}", 
            model_name.to_lowercase(), placeholder_fn(1), placeholder_fn(2));
        if model.options.soft_delete {
            find_query.push_str(" AND deleted_at IS NULL");
        }
        out.push_str(&format!(
            "        sqlx::query_as::<_, {}>(\"{}\")\n            .bind(id)\n            .bind(user_id)\n            .fetch_one(pool)\n            .await\n    }}\n\n",
            model_name, find_query
        ));
    } else {
        out.push_str(&format!(
            "    pub async fn find(pool: &{}, id: uuid::Uuid) -> Result<{}, sqlx::Error> {{\n",
            pool_type, model_name
        ));
        let mut find_query = format!("SELECT * FROM {} WHERE id = {}", model_name.to_lowercase(), placeholder_fn(1));
        if model.options.soft_delete {
            find_query.push_str(" AND deleted_at IS NULL");
        }
        out.push_str(&format!(
            "        sqlx::query_as::<_, {}>(\"{}\")\n            .bind(id)\n            .fetch_one(pool)\n            .await\n    }}\n\n",
            model_name, find_query
        ));
    }

    // update
    if tracing_enabled {
        out.push_str("    #[tracing::instrument]\n");
    }
    
    // Generate update function signature based on ownership
    if model.owned_by.is_some() {
        out.push_str(&format!(
            "    pub async fn update(pool: &{}, id: uuid::Uuid, user_id: uuid::Uuid, item: &{}Update) -> Result<{}, sqlx::Error> {{\n",
            pool_type, model_name, model_name
        ));
    } else {
        out.push_str(&format!(
            "    pub async fn update(pool: &{}, id: uuid::Uuid, item: &{}Update) -> Result<{}, sqlx::Error> {{\n",
            pool_type, model_name, model_name
        ));
    }
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
                // For password updates, we expect a 'password' field in the update struct, not 'password_hash'
                // The field name in the update struct should be 'password', but we're updating 'password_hash' in the DB
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
                    // This shouldn't happen in a well-designed schema, but handle it gracefully
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
        out.push_str("            return Self::find(pool, id, user_id).await;\n");
    } else {
        out.push_str("            return Self::find(pool, id).await;\n");
    }
    out.push_str("        }\n");
    match ir.meta.db_backend {
        ir::DatabaseBackend::Postgres | ir::DatabaseBackend::Sqlite => {
            if model.owned_by.is_some() {
                out.push_str(&format!(
                    "        qb.push(\" WHERE id = \" ).push_bind(id).push(\" AND user_id = \").push_bind(user_id).push(\" RETURNING *\");\n        let query = qb.build_query_as::<{}>();\n        query.fetch_one(pool).await\n    }}\n\n",
                    model_name,
                ));
            } else {
                out.push_str(&format!(
                    "        qb.push(\" WHERE id = \" ).push_bind(id).push(\" RETURNING *\");\n        let query = qb.build_query_as::<{}>();\n        query.fetch_one(pool).await\n    }}\n\n",
                    model_name,
                ));
            }
        }
        ir::DatabaseBackend::Mysql => {
            if model.owned_by.is_some() {
                out.push_str(&format!(
                    "        qb.push(\" WHERE id = \" ).push_bind(id).push(\" AND user_id = \").push_bind(user_id);\n        let query = qb.build();\n        query.execute(pool).await?;\n        sqlx::query_as::<_, {}>(\"SELECT * FROM {} WHERE id = ? AND user_id = ?\")\n            .bind(id)\n            .bind(user_id)\n            .fetch_one(pool)\n            .await\n    }}\n\n",
                    model_name,
                    model_name.to_lowercase(),
                ));
            } else {
                out.push_str(&format!(
                    "        qb.push(\" WHERE id = \" ).push_bind(id);\n        let query = qb.build();\n        query.execute(pool).await?;\n        sqlx::query_as::<_, {}>(\"SELECT * FROM {} WHERE id = ?\")\n            .bind(id)\n            .fetch_one(pool)\n            .await\n    }}\n\n",
                    model_name,
                    model_name.to_lowercase(),
                ));
            }
        }
    }

    // delete
    if tracing_enabled {
        out.push_str("    #[tracing::instrument]\n");
    }
    
    // Generate delete function signature based on ownership
    if model.owned_by.is_some() {
        out.push_str(&format!(
            "    pub async fn delete(pool: &{}, id: uuid::Uuid, user_id: uuid::Uuid) -> Result<u64, sqlx::Error> {{\n",
            pool_type
        ));
        
        if model.options.soft_delete {
            // Generate an UPDATE statement for soft delete with ownership
            out.push_str(&format!(
                "        let res = sqlx::query(\"UPDATE {} SET deleted_at = now() WHERE id = {} AND user_id = {}\")\n            .bind(id)\n            .bind(user_id)\n            .execute(pool)\n            .await?;\n        Ok(res.rows_affected())\n    }}\n\n",
                model_name.to_lowercase(), placeholder_fn(1), placeholder_fn(2)
            ));
        } else {
            // Generate the original hard DELETE with ownership
            out.push_str(&format!(
                "        let res = sqlx::query(\"DELETE FROM {} WHERE id = {} AND user_id = {}\")\n            .bind(id)\n            .bind(user_id)\n            .execute(pool)\n            .await?;\n        Ok(res.rows_affected())\n    }}\n\n",
                model_name.to_lowercase(), placeholder_fn(1), placeholder_fn(2)
            ));
        }
    } else {
        out.push_str(&format!(
            "    pub async fn delete(pool: &{}, id: uuid::Uuid) -> Result<u64, sqlx::Error> {{\n",
            pool_type
        ));
        
        if model.options.soft_delete {
            // Generate an UPDATE statement for soft delete
            out.push_str(&format!(
                "        let res = sqlx::query(\"UPDATE {} SET deleted_at = now() WHERE id = {}\")\n            .bind(id)\n            .execute(pool)\n            .await?;\n        Ok(res.rows_affected())\n    }}\n\n",
                model_name.to_lowercase(), placeholder_fn(1)
            ));
        } else {
            // Generate the original hard DELETE
            out.push_str(&format!(
                "        let res = sqlx::query(\"DELETE FROM {} WHERE id = {}\")\n            .bind(id)\n            .execute(pool)\n            .await?;\n        Ok(res.rows_affected())\n    }}\n\n",
                model_name.to_lowercase(), placeholder_fn(1)
            ));
        }
    }

    // list
    if tracing_enabled {
        out.push_str("    #[tracing::instrument]\n");
    }
    
    // Generate list function signature based on ownership
    if model.owned_by.is_some() {
        out.push_str(&format!(
            "    pub async fn list(pool: &{}, user_id: uuid::Uuid, pagination: Option<Pagination>) -> Result<Vec<{}>, sqlx::Error> {{\n",
            pool_type, model_name
        ));
        out.push_str(&format!(
            "        let mut qb = sqlx::QueryBuilder::<{}>::new(\"SELECT * FROM {}\");\n",
            qb_type,
            model_name.to_lowercase()
        ));
        if model.options.soft_delete {
            out.push_str("        qb.push(\" WHERE deleted_at IS NULL AND user_id = \").push_bind(user_id);\n");
        } else {
            out.push_str("        qb.push(\" WHERE user_id = \").push_bind(user_id);\n");
        }
        out.push_str("        if let Some(p) = pagination {\n");
        out.push_str("            qb.push(\" LIMIT \" ).push_bind(p.limit);\n");
        out.push_str("            qb.push(\" OFFSET \" ).push_bind(p.offset);\n");
        out.push_str("        }\n");
        out.push_str(&format!(
            "        qb.build_query_as::<{}>().fetch_all(pool).await\n    }}\n}}\n\n",
            model_name
        ));
    } else {
        out.push_str(&format!(
            "    pub async fn list(pool: &{}, pagination: Option<Pagination>) -> Result<Vec<{}>, sqlx::Error> {{\n",
            pool_type, model_name
        ));
        out.push_str(&format!(
            "        let mut qb = sqlx::QueryBuilder::<{}>::new(\"SELECT * FROM {}\");\n",
            qb_type,
            model_name.to_lowercase()
        ));
        if model.options.soft_delete {
            out.push_str("        qb.push(\" WHERE deleted_at IS NULL\");\n");
        }
        out.push_str("        if let Some(p) = pagination {\n");
        out.push_str("            qb.push(\" LIMIT \" ).push_bind(p.limit);\n");
        out.push_str("            qb.push(\" OFFSET \" ).push_bind(p.offset);\n");
        out.push_str("        }\n");
        out.push_str(&format!(
            "        qb.build_query_as::<{}>().fetch_all(pool).await\n    }}\n}}\n\n",
            model_name
        ));
    }

    out
}

fn get_db_config(backend: &ir::DatabaseBackend) -> (&str, &str, fn(usize) -> String) {
    match backend {
        ir::DatabaseBackend::Postgres => ("sqlx::PgPool", "sqlx::Postgres", |i| format!("${}", i)),
        ir::DatabaseBackend::Mysql => ("sqlx::MySqlPool", "sqlx::MySql", |_| "?".into()),
        ir::DatabaseBackend::Sqlite => ("sqlx::SqlitePool", "sqlx::Sqlite", |_| "?".into()),
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