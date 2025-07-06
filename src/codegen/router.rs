use crate::ir;

pub fn generate_router(ir: &ir::SchemaIR) -> String {
    let mut out = String::new();
    let mut models: Vec<_> = ir.models.iter().collect();
    models.sort_by(|a, b| a.0.cmp(b.0));
    
    out.push_str("use axum::{\n");
    out.push_str("    extract::{Path, Query, State},\n");
    out.push_str("    http::StatusCode,\n");
    out.push_str("    response::Json,\n");
    out.push_str("    routing::{delete, get, post, put},\n");
    out.push_str("    Router,\n");
    out.push_str("};\n");
    out.push_str("use serde_json::{json, Value};\n");
    out.push_str("use std::sync::Arc;\n");
    out.push_str("use crate::generated::*;\n");
    out.push_str("use crate::generated::main::AppState;\n");
    out.push_str("use crate::generated::auth::Claims;\n\n");
    
    out.push_str("pub fn create_router() -> Router<Arc<AppState>> {\n");
    out.push_str("    Router::new()\n");
    
    for (model_name, _model) in &models {
        let model_lower = model_name.to_lowercase();
        let route_path = format!("/{}", model_lower);
        let route_path_with_id = format!("{}/:id", route_path);
        
        // List handler
        out.push_str(&format!(
            "        .route(\"{}\", get(list_{}))\n",
            route_path, model_lower
        ));
        
        // Create handler
        out.push_str(&format!(
            "        .route(\"{}\", post(create_{}))\n",
            route_path, model_lower
        ));
        
        // Get handler
        out.push_str(&format!(
            "        .route(\"{}\", get(get_{}))\n",
            route_path_with_id, model_lower
        ));
        
        // Update handler
        out.push_str(&format!(
            "        .route(\"{}\", put(update_{}))\n",
            route_path_with_id, model_lower
        ));
        
        // Delete handler
        out.push_str(&format!(
            "        .route(\"{}\", delete(delete_{}))\n",
            route_path_with_id, model_lower
        ));
    }
    
    out.push_str("}\n\n");
    
    // Generate handler functions
    for (model_name, model) in models {
        let model_lower = model_name.to_lowercase();
        
        // List handler
        if model.owned_by.is_some() {
            out.push_str(&format!(
                "async fn list_{}(State(state): State<Arc<AppState>>, claims: Claims, Query(pagination): Query<Option<Pagination>>) -> Result<Json<Vec<{}>>, (StatusCode, Json<Value>)> {{\n",
                model_lower, model_name
            ));
            out.push_str(&format!(
                "    let items = {}::list(&state.pool, claims.sub, pagination)\n        .await\n        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({{\"error\": e.to_string()}}))))?;\n    Ok(Json(items))\n}}\n\n",
                model_name
            ));
        } else {
            out.push_str(&format!(
                "async fn list_{}(State(state): State<Arc<AppState>>, Query(pagination): Query<Option<Pagination>>) -> Result<Json<Vec<{}>>, (StatusCode, Json<Value>)> {{\n",
                model_lower, model_name
            ));
            out.push_str(&format!(
                "    let items = {}::list(&state.pool, pagination)\n        .await\n        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({{\"error\": e.to_string()}}))))?;\n    Ok(Json(items))\n}}\n\n",
                model_name
            ));
        }
        
        // Create handler
        if model.owned_by.is_some() {
            out.push_str(&format!(
                "async fn create_{}(State(state): State<Arc<AppState>>, claims: Claims, Json(item): Json<{}New>) -> Result<Json<{}>, (StatusCode, Json<Value>)> {{\n",
                model_lower, model_name, model_name
            ));
            out.push_str(&format!(
                "    let item = {}::create(&state.pool, &item, claims.sub)\n        .await\n        .map_err(|e| {{\n            let status = match e {{\n",
                model_name
            ));
        } else {
            out.push_str(&format!(
                "async fn create_{}(State(state): State<Arc<AppState>>, Json(item): Json<{}New>) -> Result<Json<{}>, (StatusCode, Json<Value>)> {{\n",
                model_lower, model_name, model_name
            ));
            out.push_str(&format!(
                "    let item = {}::create(&state.pool, &item)\n        .await\n        .map_err(|e| {{\n            let status = match e {{\n",
                model_name
            ));
        }
        
        // Unique constraint errors
        let mut unique_names: Vec<_> = model.unique_constraints.keys().collect();
        unique_names.sort();
        for unique_name in unique_names {
            let var = pascal_case(unique_name);
            out.push_str(&format!("                {}CreateError::{} => StatusCode::CONFLICT,\n", model_name, var));
        }
        
        // Foreign key errors (only for actual foreign keys where FK is on this table)
        let mut rel_names: Vec<_> = model.relations.iter()
            .filter(|(_, rel)| rel.on != "id") // Heuristic: PK is 'id'
            .map(|(k, _)| k.clone())
            .collect();
        rel_names.sort();
        for rel_name in rel_names {
            let var = format!("{}Fk", pascal_case(&rel_name));
            out.push_str(&format!("                {}CreateError::{} => StatusCode::BAD_REQUEST,\n", model_name, var));
        }
        
        out.push_str(&format!(
            "                {}CreateError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,\n            }};\n            (status, Json(json!({{\"error\": e.to_string()}})))\n        }})?;\n    Ok(Json(item))\n}}\n\n",
            model_name
        ));
        
        // Get handler
        if model.owned_by.is_some() {
            out.push_str(&format!(
                "async fn get_{}(State(state): State<Arc<AppState>>, claims: Claims, Path(id): Path<uuid::Uuid>) -> Result<Json<{}>, (StatusCode, Json<Value>)> {{\n",
                model_lower, model_name
            ));
            out.push_str(&format!(
                "    let item = {}::find(&state.pool, id, claims.sub)\n        .await\n        .map_err(|e| {{\n            if let sqlx::Error::RowNotFound = e {{\n                (StatusCode::NOT_FOUND, Json(json!({{\"error\": \"Not found\"}})))\n            }} else {{\n                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({{\"error\": e.to_string()}})))\n            }}\n        }})?;\n    Ok(Json(item))\n}}\n\n",
                model_name
            ));
        } else {
            out.push_str(&format!(
                "async fn get_{}(State(state): State<Arc<AppState>>, Path(id): Path<uuid::Uuid>) -> Result<Json<{}>, (StatusCode, Json<Value>)> {{\n",
                model_lower, model_name
            ));
            out.push_str(&format!(
                "    let item = {}::find(&state.pool, id)\n        .await\n        .map_err(|e| {{\n            if let sqlx::Error::RowNotFound = e {{\n                (StatusCode::NOT_FOUND, Json(json!({{\"error\": \"Not found\"}})))\n            }} else {{\n                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({{\"error\": e.to_string()}})))\n            }}\n        }})?;\n    Ok(Json(item))\n}}\n\n",
                model_name
            ));
        }
        
        // Update handler
        if model.owned_by.is_some() {
            out.push_str(&format!(
                "async fn update_{}(State(state): State<Arc<AppState>>, claims: Claims, Path(id): Path<uuid::Uuid>, Json(item): Json<{}Update>) -> Result<Json<{}>, (StatusCode, Json<Value>)> {{\n",
                model_lower, model_name, model_name
            ));
            out.push_str(&format!(
                "    let item = {}::update(&state.pool, id, claims.sub, &item)\n        .await\n        .map_err(|e| {{\n            if let sqlx::Error::RowNotFound = e {{\n                (StatusCode::NOT_FOUND, Json(json!({{\"error\": \"Not found\"}})))\n            }} else {{\n                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({{\"error\": e.to_string()}})))\n            }}\n        }})?;\n    Ok(Json(item))\n}}\n\n",
                model_name
            ));
        } else {
            out.push_str(&format!(
                "async fn update_{}(State(state): State<Arc<AppState>>, Path(id): Path<uuid::Uuid>, Json(item): Json<{}Update>) -> Result<Json<{}>, (StatusCode, Json<Value>)> {{\n",
                model_lower, model_name, model_name
            ));
            out.push_str(&format!(
                "    let item = {}::update(&state.pool, id, &item)\n        .await\n        .map_err(|e| {{\n            if let sqlx::Error::RowNotFound = e {{\n                (StatusCode::NOT_FOUND, Json(json!({{\"error\": \"Not found\"}})))\n            }} else {{\n                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({{\"error\": e.to_string()}})))\n            }}\n        }})?;\n    Ok(Json(item))\n}}\n\n",
                model_name
            ));
        }
        
        // Delete handler
        if model.owned_by.is_some() {
            out.push_str(&format!(
                "async fn delete_{}(State(state): State<Arc<AppState>>, claims: Claims, Path(id): Path<uuid::Uuid>) -> Result<Json<Value>, (StatusCode, Json<Value>)> {{\n",
                model_lower
            ));
            out.push_str(&format!(
                "    let affected = {}::delete(&state.pool, id, claims.sub)\n        .await\n        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({{\"error\": e.to_string()}}))))?;\n    if affected == 0 {{\n        return Err((StatusCode::NOT_FOUND, Json(json!({{\"error\": \"Not found\"}}))));\n    }}\n    Ok(Json(json!({{\"message\": \"Deleted successfully\"}})))\n}}\n\n",
                model_name
            ));
        } else {
            out.push_str(&format!(
                "async fn delete_{}(State(state): State<Arc<AppState>>, Path(id): Path<uuid::Uuid>) -> Result<Json<Value>, (StatusCode, Json<Value>)> {{\n",
                model_lower
            ));
            out.push_str(&format!(
                "    let affected = {}::delete(&state.pool, id)\n        .await\n        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({{\"error\": e.to_string()}}))))?;\n    if affected == 0 {{\n        return Err((StatusCode::NOT_FOUND, Json(json!({{\"error\": \"Not found\"}}))));\n    }}\n    Ok(Json(json!({{\"message\": \"Deleted successfully\"}})))\n}}\n\n",
                model_name
            ));
        }
    }
    
    out
}

fn pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize = true;
    
    for c in s.chars() {
        if c == '_' {
            capitalize = true;
        } else {
            if capitalize {
                result.push(c.to_ascii_uppercase());
                capitalize = false;
            } else {
                result.push(c);
            }
        }
    }
    
    result
}

 