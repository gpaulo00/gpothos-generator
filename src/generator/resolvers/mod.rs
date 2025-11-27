mod create_one;
mod find_many;
mod find_unique;
mod aggregate;
mod update_one;
mod relations;

use crate::parser::{Model, ParsedSchema};
use anyhow::Result;
use std::fs;
use std::path::Path;

/// Generate all resolvers for a model
pub fn generate_resolvers(
    model: &Model, 
    schema: &ParsedSchema, 
    output_dir: &Path,
    manual_resolvers: &crate::scanner::ManualResolvers,
) -> Result<()> {
    use crate::generator::get_prisma_name;
    
    // Create directories
    let resolvers_dir = output_dir.join("resolvers");
    fs::create_dir_all(&resolvers_dir)?;

    let names = get_prisma_name(&model.name);

    // Generate CRUD resolvers (in single directory for Pothos)
    // Check if each resolver exists manually before generating
    
    if !manual_resolvers.contains_query(&names.create) {
        create_one::generate(model, &resolvers_dir, &resolvers_dir)?;
    } else {
        println!("  ⏭️  Skipping createOne{} (manual resolver found: {})", model.name, names.create);
    }
    
    if !manual_resolvers.contains_query(&names.find_many) {
        find_many::generate(model, &resolvers_dir, &resolvers_dir)?;
    } else {
        println!("  ⏭️  Skipping findMany{} (manual resolver found: {})", model.name, names.find_many);
    }
    
    if !manual_resolvers.contains_query(&names.find) {
        find_unique::generate(model, &resolvers_dir, &resolvers_dir)?;
    } else {
        println!("  ⏭️  Skipping findUnique{} (manual resolver found: {})", model.name, names.find);
    }
    
    // Aggregate uses a different naming pattern
    let aggregate_name = format!("aggregate{}", model.name);
    if !manual_resolvers.contains_query(&aggregate_name) {
        aggregate::generate(model, &resolvers_dir, &resolvers_dir)?;
    } else {
        println!("  ⏭️  Skipping aggregate{} (manual resolver found: {})", model.name, aggregate_name);
    }
    
    if !manual_resolvers.contains_query(&names.update) {
        update_one::generate(model, &resolvers_dir, &resolvers_dir)?;
    } else {
        println!("  ⏭️  Skipping updateOne{} (manual resolver found: {})", model.name, names.update);
    }

    // Generate relations resolver (always generate, as it's model-specific)
    relations::generate(model, schema, &resolvers_dir)?;

    Ok(())
}
