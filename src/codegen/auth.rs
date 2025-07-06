use crate::ir;

pub fn generate_auth_module(ir: &ir::SchemaIR) -> String {
    let mut out = String::new();
    
    out.push_str("use axum::{\n");
    out.push_str("    extract::{MatchedPath, State},\n");
    out.push_str("    http::{Request, StatusCode},\n");
    out.push_str("    middleware::Next,\n");
    out.push_str("    response::Response,\n");
    out.push_str("};\n");
    out.push_str("use axum_extra::{\n");
    out.push_str("    headers::{authorization::Bearer, Authorization},\n");
    out.push_str("    TypedHeader,\n");
    out.push_str("};\n");
    out.push_str("use jsonwebtoken::{decode, DecodingKey, Validation};\n");
    out.push_str("use serde::{Deserialize, Serialize};\n");
    out.push_str("use std::sync::Arc;\n");
    out.push_str("use crate::generated::main::AppState;\n");
    out.push_str("use crate::generated::routes::routes;\n");
    out.push_str("use crate::generated::permissions::{route_has_permission, Route};\n\n");
    
    // Generate Claims struct
    out.push_str("#[derive(Debug, Serialize, Deserialize, Clone)]\n");
    out.push_str("pub struct Claims {\n");
    out.push_str("    pub sub: uuid::Uuid,\n");
    out.push_str(&format!("    pub {}: String,\n", ir.meta.auth.role_claim));
    out.push_str("    pub exp: usize,\n");
    out.push_str("}\n\n");
    
    // Generate auth middleware
    out.push_str("pub async fn auth_middleware(\n");
    out.push_str("    matched_path: MatchedPath,\n");
    out.push_str("    State(state): State<Arc<AppState>>,\n");
    out.push_str("    auth_header: Option<TypedHeader<Authorization<Bearer>>>,\n");
    out.push_str("    mut request: Request<axum::body::Body>,\n");
    out.push_str("    next: Next,\n");
    out.push_str(") -> Result<Response, StatusCode> {\n");
    
    // Extract the path
    out.push_str("    let path = matched_path.as_str();\n\n");
    
    // Generate route mapping for permission checks
    out.push_str("    // Map the matched path to route definition\n");
    out.push_str("    let route: Option<&Route> = match path {\n");
    
    // Generate route mappings for all defined routes
    let mut routes: Vec<_> = ir.routes.iter().collect();
    routes.sort_by(|a, b| a.0.cmp(b.0));
    
    for (route_name, route_def) in &routes {
        let route_path = &route_def.path;
        let route_path_with_id = format!("{}/:id", route_path);
        out.push_str(&format!("        \"{}\" | \"{}\" => Some(&routes::{}),\n", 
            route_path, route_path_with_id, route_name.to_uppercase()));
    }
    
    out.push_str("        _ => None,\n");
    out.push_str("    };\n\n");
    
    // Check if authentication is required for this route
    out.push_str("    let auth_is_required = route.map_or(true, |r| r.auth_required);\n\n");
    
    out.push_str("    if !auth_is_required {\n");
    out.push_str("        // This is a public route, let it through without any checks\n");
    out.push_str("        return Ok(next.run(request).await);\n");
    out.push_str("    }\n\n");
    
    // Handle JWT authentication for protected routes
    match ir.meta.auth.provider {
        ir::AuthProvider::None => {
            out.push_str("    // No JWT provider configured - create anonymous claims for protected routes\n");
            out.push_str("    let claims = Claims {\n");
            out.push_str("        sub: uuid::Uuid::nil(),\n");
            out.push_str(&format!("        {}: \"{}\".to_string(),\n", ir.meta.auth.role_claim, ir.meta.auth.anonymous_role));
            out.push_str("        exp: usize::MAX,\n");
            out.push_str("    };\n");
        }
        ir::AuthProvider::Jwt => {
            out.push_str("    // Extract and validate JWT token\n");
            out.push_str("    let token = auth_header\n");
            out.push_str("        .ok_or(StatusCode::UNAUTHORIZED)?\n");
            out.push_str("        .token()\n");
            out.push_str("        .to_string();\n\n");
            
            out.push_str("    let claims = decode::<Claims>(\n");
            out.push_str("        &token,\n");
            out.push_str("        &DecodingKey::from_secret(state.jwt_secret.as_ref()),\n");
            out.push_str("        &Validation::default(),\n");
            out.push_str("    )\n");
            out.push_str("    .map_err(|_| StatusCode::UNAUTHORIZED)?\n");
            out.push_str("    .claims;\n");
        }
    }
    
    // Check role-based permissions
    out.push_str("\n    // Check role-based permissions\n");
    out.push_str("    if let Some(r) = route {\n");
    out.push_str(&format!("        if !route_has_permission(r, request.method().as_str(), &claims.{}) {{\n", ir.meta.auth.role_claim));
    out.push_str("            return Err(StatusCode::FORBIDDEN);\n");
    out.push_str("        }\n");
    out.push_str("    }\n\n");
    
    out.push_str("    request.extensions_mut().insert(claims);\n");
    out.push_str("    Ok(next.run(request).await)\n");
    
    out.push_str("}\n\n");
    
    // Generate helper function to extract claims from request
    out.push_str("use axum::extract::FromRequestParts;\n");
    out.push_str("use axum::http::request::Parts;\n\n");
    
    out.push_str("#[axum::async_trait]\n");
    out.push_str("impl<S> FromRequestParts<S> for Claims\n");
    out.push_str("where\n");
    out.push_str("    S: Send + Sync,\n");
    out.push_str("{\n");
    out.push_str("    type Rejection = StatusCode;\n\n");
    
    out.push_str("    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {\n");
    out.push_str("        parts.extensions\n");
    out.push_str("            .get::<Claims>()\n");
    out.push_str("            .cloned()\n");
    out.push_str("            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)\n");
    out.push_str("    }\n");
    out.push_str("}\n");
    
    out
}