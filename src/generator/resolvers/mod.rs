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
    verbose: bool,
) -> Result<()> {
    use crate::generator::get_prisma_name;
    
    // Create directories
    let resolvers_dir = output_dir.join("resolvers");
    fs::create_dir_all(&resolvers_dir)?;

    let names = get_prisma_name(&model.name);

    // Generate CRUD resolvers (in single directory for Pothos)
    // Check if each resolver exists manually before generating
    // Note: createOne and updateOne are mutations, others are queries
    
    if !manual_resolvers.contains_mutation(&names.create) {
        create_one::generate(model, &resolvers_dir, &resolvers_dir)?;
    } else if verbose {
        println!("  ⏭️  Skipping createOne{} (manual mutation found: {})", model.name, names.create);
    }
    
    if !manual_resolvers.contains_query(&names.find_many) {
        find_many::generate(model, &resolvers_dir, &resolvers_dir)?;
    } else if verbose {
        println!("  ⏭️  Skipping findMany{} (manual query found: {})", model.name, names.find_many);
    }
    
    if !manual_resolvers.contains_query(&names.find) {
        find_unique::generate(model, &resolvers_dir, &resolvers_dir)?;
    } else if verbose {
        println!("  ⏭️  Skipping findUnique{} (manual query found: {})", model.name, names.find);
    }
    
    // Aggregate uses a different naming pattern
    let aggregate_name = format!("aggregate{}", model.name);
    if !manual_resolvers.contains_query(&aggregate_name) {
        aggregate::generate(model, &resolvers_dir, &resolvers_dir)?;
    } else if verbose {
        println!("  ⏭️  Skipping aggregate{} (manual query found: {})", model.name, aggregate_name);
    }
    
    if !manual_resolvers.contains_mutation(&names.update) {
        update_one::generate(model, &resolvers_dir, &resolvers_dir)?;
    } else if verbose {
        println!("  ⏭️  Skipping updateOne{} (manual mutation found: {})", model.name, names.update);
    }

    // Generate relations resolver (always generate, as it's model-specific)
    relations::generate(model, schema, &resolvers_dir)?;

    Ok(())
}
