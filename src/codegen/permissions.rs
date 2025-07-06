use crate::ir;

pub fn generate_permissions(ir: &ir::SchemaIR) -> String {
    let mut out = String::new();
    
    out.push_str("use std::collections::HashSet;\n\n");
    out.push_str("pub struct Permissions { pub read: &'static [&'static str], pub update: &'static [&'static str], pub delete: &'static [&'static str] }\n");
    out.push_str("pub struct Route { pub methods: &'static [&'static str], pub path: &'static str, pub auth_required: bool, pub permissions: Permissions }\n");
    out.push_str(&format!("pub const ANONYMOUS_ROLE: &str = \"{}\";\n", ir.meta.auth.anonymous_role));
    out.push_str(&format!("pub const PUBLIC_ROLE: &str = \"{}\";\n", ir.meta.auth.public_role));
    out.push_str(&format!("pub const ROLE_CLAIM: &str = \"{}\";\n", ir.meta.auth.role_claim));
    
    out.push_str("pub fn has_permission(role: &str, allowed: &[&str]) -> bool { allowed.is_empty() || allowed.iter().any(|r| *r == PUBLIC_ROLE || *r == role) }\n");
    out.push_str("pub fn route_has_permission(route: &Route, method: &str, role: &str) -> bool {\n");
    out.push_str("    let m = method.to_uppercase();\n");
    out.push_str("    let allowed = if m == \"GET\" {\n");
    out.push_str("        route.permissions.read\n");
    out.push_str("    } else if m == \"DELETE\" {\n");
    out.push_str("        route.permissions.delete\n");
    out.push_str("    } else if m == \"POST\" || m == \"PUT\" || m == \"PATCH\" {\n");
    out.push_str("        route.permissions.update\n");
    out.push_str("    } else {\n");
    out.push_str("        &[]\n");
    out.push_str("    };\n");
    out.push_str("    has_permission(role, allowed)\n");
    out.push_str("}\n");
    
    if matches!(ir.meta.auth.provider, ir::AuthProvider::Jwt) {
        out.push_str("pub fn role_from_jwt(token: &str, secret: &str) -> Option<String> {\n");
        out.push_str("    let data = jsonwebtoken::decode::<serde_json::Value>(token, &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()), &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256)).ok()?;\n");
        out.push_str("    data.claims.get(ROLE_CLAIM)?.as_str().map(|s| s.to_string())\n");
        out.push_str("}\n");
    }
    
    out
} 