use std::collections::HashSet;

pub struct Permissions { pub read: &'static [&'static str], pub update: &'static [&'static str], pub delete: &'static [&'static str] }
pub struct Route { pub methods: &'static [&'static str], pub path: &'static str, pub auth_required: bool, pub permissions: Permissions }
pub const ANONYMOUS_ROLE: &str = "guest";
pub const PUBLIC_ROLE: &str = "public";
pub const ROLE_CLAIM: &str = "role";
pub fn has_permission(role: &str, allowed: &[&str]) -> bool { allowed.is_empty() || allowed.iter().any(|r| *r == PUBLIC_ROLE || *r == role) }
pub fn route_has_permission(route: &Route, method: &str, role: &str) -> bool {
    let m = method.to_uppercase();
    let allowed = if m == "GET" {
        route.permissions.read
    } else if m == "DELETE" {
        route.permissions.delete
    } else if m == "POST" || m == "PUT" || m == "PATCH" {
        route.permissions.update
    } else {
        &[]
    };
    has_permission(role, allowed)
}
pub fn role_from_jwt(token: &str, secret: &str) -> Option<String> {
    let data = jsonwebtoken::decode::<serde_json::Value>(token, &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()), &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256)).ok()?;
    data.claims.get(ROLE_CLAIM)?.as_str().map(|s| s.to_string())
}
