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
pub fn generate_resolvers(model: &Model, schema: &ParsedSchema, output_dir: &Path) -> Result<()> {
    // Create directories
    let resolvers_dir = output_dir.join("resolvers");
    fs::create_dir_all(&resolvers_dir)?;

    // Generate CRUD resolvers (in single directory for Pothos)
    create_one::generate(model, &resolvers_dir, &resolvers_dir)?;
    find_many::generate(model, &resolvers_dir, &resolvers_dir)?;
    find_unique::generate(model, &resolvers_dir, &resolvers_dir)?;
    aggregate::generate(model, &resolvers_dir, &resolvers_dir)?;
    update_one::generate(model, &resolvers_dir, &resolvers_dir)?;

    // Generate relations resolver
    relations::generate(model, schema, &resolvers_dir)?;

    Ok(())
}
