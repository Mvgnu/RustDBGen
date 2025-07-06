use crate::ir;

pub fn generate_app_main(_ir: &crate::ir::SchemaIR) -> String {
    let mut out = String::new();
    out.push_str("pub mod generated;\n\n");
    out.push_str("use generated::main::*;\n");
    out.push_str("use generated::router::*;\n\n");
    
    out.push_str("#[tokio::main]\nasync fn main() {\n");
    out.push_str("    if let Err(e) = generated::main::main().await {\n");
    out.push_str("        eprintln!(\"Error: {}\", e);\n");
    out.push_str("        std::process::exit(1);\n");
    out.push_str("    }\n");
    out.push_str("}\n");
    out
} 