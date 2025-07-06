use crate::ir;

pub fn generate_models(ir: &ir::SchemaIR) -> anyhow::Result<String> {
    let mut out = String::new();

    let mut needs_chrono = false;
    let mut needs_uuid = false;
    let mut needs_decimal = false;

    for model in ir.models.values() {
        for field in model.fields.values() {
            if field.rust_type.contains("DateTime") {
                needs_chrono = true;
            }
            if field.rust_type.contains("Uuid") {
                needs_uuid = true;
            }
            if field.rust_type.contains("Decimal") {
                needs_decimal = true;
            }
        }
    }

    out.push_str("use serde::{Serialize, Deserialize};\n");
    if needs_chrono {
        out.push_str("use chrono::{DateTime, Utc};\n");
    }
    if needs_uuid {
        out.push_str("use uuid::Uuid;\n");
    }
    if needs_decimal {
        out.push_str("use rust_decimal::Decimal;\n");
    }
    out.push_str("use thiserror::Error;\n\n");

    out.push_str(&generate_enums(ir));
    out.push_str(&generate_model_structs(ir));
    out.push_str(&generate_error_enums(ir));

    Ok(out)
}

pub fn generate_enums(ir: &ir::SchemaIR) -> String {
    let mut out = String::new();
    let mut enums: Vec<_> = ir.enums.iter().collect();
    enums.sort_by(|a, b| a.0.cmp(b.0));

    for (name, en) in enums {
        out.push_str("#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type)]\n");
        out.push_str(&format!(
            "#[sqlx(type_name = \"{}\", rename_all = \"lowercase\")]\n",
            name.to_lowercase()
        ));
        out.push_str(&format!("pub enum {} {{\n", name));
        for variant in &en.variants {
            let pascal_variant = pascal_case(variant);
            out.push_str(&format!("    {},\n", pascal_variant));
        }
        out.push_str("}\n\n");
    }

    out
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

pub fn generate_model_structs(ir: &ir::SchemaIR) -> String {
    let mut out = String::new();
    let mut models: Vec<_> = ir.models.iter().collect();
    models.sort_by(|a, b| a.0.cmp(b.0));

    for (model_name, model) in models {
        // Main struct
        let mut derives = ir.meta.default_derives.clone();
        derives.push("sqlx::FromRow".into());
        let derives_list = derives.join(", ");
        out.push_str(&format!(
            "#[derive({})]\npub struct {} {{\n",
            derives_list, model_name
        ));
        let mut fields_vec: Vec<_> = model.fields.iter().collect();
        fields_vec.sort_by(|a, b| a.0.cmp(b.0));
        for (field_name, field) in &fields_vec {
            let field_name_escaped = escape_rust_keyword(field_name);
            let mut rust_type = field.rust_type.clone();
            if rust_type == "Boolean" {
                rust_type = "bool".to_string();
            }
            if field.nullable && !rust_type.starts_with("Option<") {
                rust_type = format!("Option<{}>", rust_type);
            }
            out.push_str(&format!("    pub {}: {},\n", field_name_escaped, rust_type));
        }
        out.push_str("}\n\n");

        // New struct (fields without defaults and not soft-delete)
        out.push_str(&format!(
            "#[derive(Debug, serde::Deserialize)]\npub struct {}New {{\n",
            model_name
        ));
        for (field_name, field) in &fields_vec {
            if field.default.is_none() && *field_name != "deleted_at" {
                // Skip owner foreign key field if this model is owned
                if let Some(owner_model) = &model.owned_by {
                    let owner_fk = format!("{}_id", owner_model.to_lowercase());
                    if **field_name == owner_fk {
                        continue;
                    }
                }
                
                let field_name_escaped = escape_rust_keyword(field_name);
                let mut rust_type = field.rust_type.clone();
                
                // Handle password fields - use plaintext password instead of hash
                if field.tags.contains(&"password".to_string()) {
                    out.push_str("    pub password: String,\n");
                    continue;
                }
                
                if rust_type == "Boolean" {
                    rust_type = "bool".to_string();
                }
                if field.nullable && !rust_type.starts_with("Option<") {
                    rust_type = format!("Option<{}>", rust_type);
                }
                out.push_str(&format!("    pub {}: {},\n", field_name_escaped, rust_type));
            }
        }
        out.push_str("}\n\n");

        // Update struct (Option fields)
        out.push_str(&format!(
            "#[derive(Debug, serde::Deserialize, Default)]\npub struct {}Update {{\n",
            model_name
        ));
        for (field_name, field) in &fields_vec {
            if field.default.is_none() {
                let field_name_escaped = escape_rust_keyword(field_name);
                let mut rust_type = field.rust_type.clone();
                
                // Handle password fields - use plaintext password instead of hash
                if field.tags.contains(&"password".to_string()) {
                    out.push_str("    pub password: Option<String>,\n");
                    continue;
                }
                
                if rust_type == "Boolean" {
                    rust_type = "bool".to_string();
                }
                out.push_str(&format!(
                    "    pub {}: Option<{}>,\n",
                    field_name_escaped, rust_type
                ));
            }
        }
        out.push_str("}\n\n");
    }

    out
}

pub fn generate_error_enums(ir: &ir::SchemaIR) -> String {
    let mut out = String::new();
    let mut models: Vec<_> = ir.models.iter().collect();
    models.sort_by(|a, b| a.0.cmp(b.0));

    for (model_name, model) in models {
        out.push_str(&format!(
            "#[derive(Debug, Error)]\npub enum {}CreateError {{\n",
            model_name
        ));
        let mut ucs: Vec<_> = model.unique_constraints.keys().collect();
        ucs.sort();
        for uc_name in ucs {
            let var = pascal_case(uc_name);
            out.push_str(&format!(
                "    #[error(\"unique constraint `{}` violated\")]\n    {},\n",
                uc_name, var
            ));
        }
        // Only generate FK errors for forward relations (where the FK is on this table)
        let mut rels: Vec<_> = model
            .relations
            .iter()
            .filter(|(_, rel)| rel.on != "id") // Heuristic: PK is 'id'
            .collect();
        rels.sort_by(|a, b| a.0.cmp(b.0));
        for (rel_name, _) in rels {
            let var = format!("{}Fk", pascal_case(rel_name));
            out.push_str(&format!(
                "    #[error(\"foreign key `{}` violation\")]\n    {},\n",
                rel_name, var
            ));
        }
        out.push_str("    #[error(transparent)]\n    Database(#[from] sqlx::Error),\n}\n\n");
    }

    out
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