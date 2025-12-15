pub mod enums;
pub mod filters;
pub mod helpers;
pub mod inputs;
pub mod models;
pub mod relations;
pub mod resolvers;

pub use helpers::get_prisma_name;

use crate::parser::ParsedSchema;
use anyhow::Result;
use std::fs;
use std::path::Path;

/// Generate all Pothos code from parsed schema
pub fn generate(schema: &ParsedSchema, output_dir: &Path, manual_resolvers: &crate::scanner::ManualResolvers, verbose: bool) -> Result<()> {
    // Create output directories
    create_directories(output_dir)?;

    // Generate builder (contains scalars, context, etc.)
    if verbose {
        println!("Generating builder...");
    }
    helpers::generate_helpers(output_dir)?;

    // Generate enums
    if verbose {
        println!("Generating enums...");
    }
    enums::generate_base_enums(output_dir)?;
    enums::generate_schema_enums(schema, output_dir)?;

    // Generate filters
    if verbose {
        println!("Generating filters...");
    }
    filters::generate_filters(output_dir)?;

    // Generate per-model files
    for model in &schema.models {
        if verbose {
            println!("Generating for model: {}", model.name);
        }

        models::generate_model(model, output_dir)?;
        inputs::generate_inputs(model, output_dir)?;
        resolvers::generate_resolvers(model, schema, output_dir, manual_resolvers, verbose)?;
    }

    // Generate relation inputs (must be after all models are processed)
    if verbose {
        println!("Generating relation inputs...");
    }
    relations::generate_all_relation_inputs(schema, output_dir)?;

    // Generate index file
    generate_index(schema, output_dir, manual_resolvers)?;

    Ok(())
}

/// Run as a Prisma generator (reads DMMF from stdin)
pub fn run_as_prisma_generator() -> Result<()> {
    use std::io::{self, BufRead, Write};

    // Prisma generator protocol uses JSON-RPC over stdio
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }

        let request: serde_json::Value = serde_json::from_str(&line)?;

        match request.get("method").and_then(|m| m.as_str()) {
            Some("getManifest") => {
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": request.get("id"),
                    "result": {
                        "manifest": {
                            "prettyName": "Prisma Pothos Generator",
                            "defaultOutput": "../src/generated",
                            "requiresGenerators": ["prisma-client-js"]
                        }
                    }
                });
                writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
                stdout.flush()?;
            }
            Some("generate") => {
                // Extract DMMF from params
                if let Some(params) = request.get("params") {
                    if let Some(dmmf) = params.get("dmmf") {
                        let output_path = params
                            .get("generator")
                            .and_then(|g| g.get("output"))
                            .and_then(|o| o.get("value"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("./src/generated");


                        // Parse DMMF and generate
                        let schema = parse_dmmf(dmmf)?;
                        // In prisma generator mode, we don't scan for manual resolvers
                        let manual_resolvers = crate::scanner::ManualResolvers::new();
                        generate(&schema, Path::new(output_path), &manual_resolvers, false)?;
                    }
                }

                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": request.get("id"),
                    "result": null
                });
                writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
                stdout.flush()?;
            }
            _ => {}
        }
    }

    Ok(())
}

/// Parse Prisma DMMF (Data Model Meta Format) into our ParsedSchema
fn parse_dmmf(dmmf: &serde_json::Value) -> Result<ParsedSchema> {
    use crate::parser::{Enum, EnumValue, Field, FieldType, Model, PrimaryKey, Relation};

    let mut models = Vec::new();
    let mut enums = Vec::new();

    // Parse enums from DMMF
    if let Some(datamodel) = dmmf.get("datamodel") {
        if let Some(dmmf_enums) = datamodel.get("enums").and_then(|e| e.as_array()) {
            for e in dmmf_enums {
                let values: Vec<EnumValue> = e
                    .get("values")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| {
                                Some(EnumValue {
                                    name: v.get("name")?.as_str()?.to_string(),
                                    db_name: v.get("dbName").and_then(|d| d.as_str()).map(String::from),
                                })
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                if let Some(name) = e.get("name").and_then(|n| n.as_str()) {
                    enums.push(Enum {
                        name: name.to_string(),
                        values,
                    });
                }
            }
        }

        // Parse models from DMMF
        if let Some(dmmf_models) = datamodel.get("models").and_then(|m| m.as_array()) {
            for m in dmmf_models {
                let name = m.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
                let db_name = m.get("dbName").and_then(|d| d.as_str()).map(String::from);

                let fields: Vec<Field> = m
                    .get("fields")
                    .and_then(|f| f.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|f| {
                                let field_name = f.get("name")?.as_str()?.to_string();
                                let kind = f.get("kind")?.as_str()?;
                                let type_str = f.get("type")?.as_str()?;
                                let is_required = f.get("isRequired").and_then(|r| r.as_bool()).unwrap_or(false);
                                let is_list = f.get("isList").and_then(|l| l.as_bool()).unwrap_or(false);
                                let is_id = f.get("isId").and_then(|i| i.as_bool()).unwrap_or(false);
                                let is_unique = f.get("isUnique").and_then(|u| u.as_bool()).unwrap_or(false);
                                let is_updated_at = f.get("isUpdatedAt").and_then(|u| u.as_bool()).unwrap_or(false);

                                let field_type = match kind {
                                    "scalar" => match type_str {
                                        "String" => FieldType::String,
                                        "Int" => FieldType::Int,
                                        "Float" => FieldType::Float,
                                        "Boolean" => FieldType::Boolean,
                                        "DateTime" => FieldType::DateTime,
                                        "Json" => FieldType::Json,
                                        "Decimal" => FieldType::Decimal,
                                        "BigInt" => FieldType::BigInt,
                                        "Bytes" => FieldType::Bytes,
                                        _ => FieldType::String,
                                    },
                                    "enum" => FieldType::Enum(type_str.to_string()),
                                    "object" => FieldType::Model(type_str.to_string()),
                                    _ => FieldType::String,
                                };

                                let relation = if kind == "object" {
                                    Some(Relation {
                                        name: f.get("relationName").and_then(|r| r.as_str()).map(String::from),
                                        fields: f
                                            .get("relationFromFields")
                                            .and_then(|r| r.as_array())
                                            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                                            .unwrap_or_default(),
                                        references: f
                                            .get("relationToFields")
                                            .and_then(|r| r.as_array())
                                            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                                            .unwrap_or_default(),
                                        related_model: type_str.to_string(),
                                        on_delete: f
                                            .get("relationOnDelete")
                                            .and_then(|r| r.as_str())
                                            .map(String::from),
                                        on_update: None,
                                    })
                                } else {
                                    None
                                };

                                Some(Field {
                                    name: field_name,
                                    field_type,
                                    is_required,
                                    is_list,
                                    is_id,
                                    is_unique,
                                    is_updated_at,
                                    default_value: f.get("default").map(|d| d.to_string()),
                                    relation,
                                })
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                // Extract primary key
                let primary_key = m.get("primaryKey").and_then(|pk| {
                    if pk.is_null() {
                        // Single field PK
                        let pk_field = fields.iter().find(|f| f.is_id)?;
                        Some(PrimaryKey {
                            fields: vec![pk_field.name.clone()],
                            name: None,
                        })
                    } else {
                        // Composite PK
                        let pk_fields: Vec<String> = pk
                            .get("fields")
                            .and_then(|f| f.as_array())
                            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                            .unwrap_or_default();
                        Some(PrimaryKey {
                            fields: pk_fields,
                            name: pk.get("name").and_then(|n| n.as_str()).map(String::from),
                        })
                    }
                });

                models.push(Model {
                    name,
                    db_name,
                    fields,
                    primary_key,
                    unique_fields: Vec::new(),
                });
            }
        }
    }

    Ok(ParsedSchema { models, enums })
}

fn create_directories(output_dir: &Path) -> Result<()> {
    let dirs = [
        output_dir.to_path_buf(),
        output_dir.join("models"),
        output_dir.join("enums"),
        output_dir.join("inputs"),
        output_dir.join("resolvers"),
    ];

    for dir in &dirs {
        fs::create_dir_all(dir)?;
    }

    Ok(())
}

fn generate_index(schema: &ParsedSchema, output_dir: &Path, manual_resolvers: &crate::scanner::ManualResolvers) -> Result<()> {
    let mut content = String::new();

    content.push_str("// Auto-generated by prisma-pothos-generator\n");
    content.push_str("// DO NOT EDIT MANUALLY\n\n");

    content.push_str("// Builder (must be imported first)\n");
    content.push_str("import { builder, prisma } from './builder';\n");
    content.push_str("export { builder, prisma };\n\n");

    content.push_str("// Enums\n");
    content.push_str("export * from './enums';\n\n");

    content.push_str("// Filters\n");
    content.push_str("export * from './inputs/filters';\n\n");

    content.push_str("// Models\n");
    for model in &schema.models {
        content.push_str(&format!("export * from './models/{}';\n", model.name));
    }

    content.push_str("\n// Inputs\n");
    for model in &schema.models {
        let names = get_prisma_name(&model.name);
        content.push_str(&format!("export * from './inputs/{}';\n", names.create_input));
        content.push_str(&format!("export * from './inputs/{}';\n", names.create_many_input));
        content.push_str(&format!("export * from './inputs/{}';\n", names.update_input));
        content.push_str(&format!("export * from './inputs/{}';\n", names.where_input));
        content.push_str(&format!("export * from './inputs/{}';\n", names.where_unique_input));
        content.push_str(&format!("export * from './inputs/{}';\n", names.order_by_input));
    }

    content.push_str("\n// Resolvers\n");
    for model in &schema.models {
        let names = get_prisma_name(&model.name);
        
        // Only export resolvers that were actually generated (not skipped)
        // Note: createOne, createMany and updateOne are mutations, others are queries
        if !manual_resolvers.contains_mutation(&names.create) {
            content.push_str(&format!("export * from './resolvers/createOne{}';\n", model.name));
        }
        
        if !manual_resolvers.contains_mutation(&names.create_many) {
            content.push_str(&format!("export * from './resolvers/createMany{}';\n", model.name));
        }
        
        if !manual_resolvers.contains_query(&names.find_many) {
            content.push_str(&format!("export * from './resolvers/findMany{}';\n", model.name));
        }
        
        if !manual_resolvers.contains_query(&names.find) {
            content.push_str(&format!("export * from './resolvers/findUnique{}';\n", model.name));
        }
        
        let aggregate_name = format!("aggregate{}", model.name);
        if !manual_resolvers.contains_query(&aggregate_name) {
            content.push_str(&format!("export * from './resolvers/aggregate{}';\n", model.name));
        }
        
        if !manual_resolvers.contains_mutation(&names.update) {
            content.push_str(&format!("export * from './resolvers/updateOne{}';\n", model.name));
        }
    }

    content.push_str("\n// Build schema\n");
    content.push_str("export const schema = builder.toSchema();\n");

    fs::write(output_dir.join("index.ts"), content)?;

    Ok(())
}
