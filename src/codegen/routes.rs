use crate::ir;

pub fn generate_routes(ir: &ir::SchemaIR) -> String {
    let mut out = String::new();
    
    out.push_str("use crate::generated::{Route, Permissions};\n\n");
    out.push_str("pub mod routes {\n");
    out.push_str("    use super::*;\n\n");
    let mut routes_vec: Vec<_> = ir.routes.iter().collect();
    routes_vec.sort_by(|a, b| a.0.cmp(b.0));
    
    for (name, route) in routes_vec {
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
            "    pub const {}: Route = Route {{ methods: &[{methods}], path: \"{path}\", auth_required: {auth}, permissions: Permissions {{ read: &[{read}], update: &[{update}], delete: &[{delete}] }} }};\n",
            name.to_uppercase(),
            methods = methods,
            path = route.path,
            auth = route.auth_required,
            read = read_roles,
            update = update_roles,
            delete = delete_roles,
        ));
    }
    out.push_str("}\n\n");
    
    out
} 