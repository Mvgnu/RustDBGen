use rustdbgen::{load_schema, apply_macros, apply_model_options, apply_type_aliases};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ir = load_schema("schema.model.toml")?;
    apply_macros(&mut ir);
    apply_model_options(&mut ir);
    
    println!("Loaded schema with {} models", ir.models.len());
    
    for (name, model) in &ir.models {
        println!("\nModel: {}", name);
        println!("  owned_by: {:?}", model.owned_by);
        println!("  fields: {:?}", model.fields.keys().collect::<Vec<_>>());
    }
    
    Ok(())
}