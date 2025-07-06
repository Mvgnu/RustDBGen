pub mod models;
pub mod routes;
pub mod permissions;
pub mod pagination;
pub mod handlers;
pub mod handlers_enhanced;
pub mod router;
pub mod main_server;
pub mod main_server_enhanced;
pub mod cargo_toml;
pub mod app_main;
pub mod auth;
pub mod executor;
pub mod config;

use crate::ir;
use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn generate_code_multi(ir: &ir::SchemaIR, output_dir: &Path) -> Result<()> {
    // Create output directory structure
    let src_dir = output_dir.join("src");
    let generated_dir = src_dir.join("generated");
    fs::create_dir_all(&generated_dir)?;
    
    let models_dir = generated_dir.join("models");
    fs::create_dir_all(&models_dir)?;
    
    let handlers_dir = generated_dir.join("handlers");
    fs::create_dir_all(&handlers_dir)?;
    
    let routes_dir = generated_dir.join("routes");
    fs::create_dir_all(&routes_dir)?;
    
    // Generate models
    let models_code = models::generate_models(ir)?;
    fs::write(models_dir.join("mod.rs"), models_code)?;
    
    // Generate enhanced handlers
    let handlers_code = handlers_enhanced::generate_enhanced_crud_impls(ir);
    fs::write(handlers_dir.join("mod.rs"), handlers_code)?;
    
    // Generate routes
    let routes_code = routes::generate_routes(ir);
    fs::write(routes_dir.join("mod.rs"), routes_code)?;
    
    // Generate permissions
    let permissions_code = permissions::generate_permissions(ir);
    fs::write(generated_dir.join("permissions.rs"), permissions_code)?;
    
    // Generate pagination
    let pagination_code = pagination::generate_pagination();
    fs::write(generated_dir.join("pagination.rs"), pagination_code)?;
    
    // Generate router
    let router_code = router::generate_router(ir);
    fs::write(generated_dir.join("router.rs"), router_code)?;
    
    // Generate auth module
    let auth_code = auth::generate_auth_module(ir);
    fs::write(generated_dir.join("auth.rs"), auth_code)?;
    
    // Generate enhanced main server
    let main_server_code = main_server_enhanced::generate_enhanced_main_server(ir);
    fs::write(generated_dir.join("main.rs"), main_server_code)?;
    
    // Generate executor trait
    let executor_code = executor::generate_executor_trait(ir);
    fs::write(generated_dir.join("executor.rs"), executor_code)?;
    
    // Generate config struct
    let config_code = config::generate_config_struct(ir);
    fs::write(generated_dir.join("config.rs"), config_code)?;
    
    // Generate mod.rs for generated module
    let mod_rs = generate_mod_rs(ir);
    fs::write(generated_dir.join("mod.rs"), mod_rs)?;
    
    // Generate Cargo.toml
    let cargo_toml = cargo_toml::generate_cargo_toml(ir);
    fs::write(output_dir.join("Cargo.toml"), cargo_toml)?;
    
    // Generate src/main.rs
    fs::create_dir_all(&src_dir)?;
    let app_main = app_main::generate_app_main(ir);
    fs::write(src_dir.join("main.rs"), app_main)?;
    
    Ok(())
}

fn generate_mod_rs(_ir: &ir::SchemaIR) -> String {
    let mut out = String::new();
    
    out.push_str("pub mod models;\n");
    out.push_str("pub mod handlers;\n");
    out.push_str("pub mod routes;\n");
    out.push_str("pub mod permissions;\n");
    out.push_str("pub mod pagination;\n");
    out.push_str("pub mod router;\n");
    out.push_str("pub mod auth;\n");
    out.push_str("pub mod main;\n");
    out.push_str("pub mod executor;\n");
    out.push_str("pub mod config;\n\n");
    
    out.push_str("pub use models::*;\n");
    out.push_str("pub use handlers::*;\n");
    out.push_str("pub use routes::*;\n");
    out.push_str("pub use permissions::*;\n");
    out.push_str("pub use pagination::*;\n");
    out.push_str("pub use router::*;\n");
    out.push_str("pub use auth::*;\n");
    out.push_str("pub use main::*;\n");
    out.push_str("pub use executor::*;\n");
    out.push_str("pub use config::*;\n");
    
    out
} 