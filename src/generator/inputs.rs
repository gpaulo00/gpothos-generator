use crate::parser::{FieldType, Model};
use crate::generator::get_prisma_name;
use anyhow::Result;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Collect all enum types used by scalar fields in a model
fn collect_enum_types(model: &Model) -> HashSet<String> {
    let mut used_enums: HashSet<String> = HashSet::new();
    for field in &model.fields {
        if field.relation.is_some() {
            continue;
        }
        if let FieldType::Enum(enum_name) = &field.field_type {
            used_enums.insert(enum_name.clone());
        }
    }
    used_enums
}

/// Generate all Pothos input types for a model
pub fn generate_inputs(model: &Model, output_dir: &Path) -> Result<()> {
    let inputs_dir = output_dir.join("inputs");
    fs::create_dir_all(&inputs_dir)?;

    generate_create_input(model, &inputs_dir)?;
    generate_update_input(model, &inputs_dir)?;
    generate_where_input(model, &inputs_dir)?;
    generate_where_unique_input(model, &inputs_dir)?;
    generate_order_by_input(model, &inputs_dir)?;
    
    // Generate relation-specific input types
    generate_where_unique_input_for_relations(model, &inputs_dir)?;
    generate_relation_create_input(model, &inputs_dir)?;

    Ok(())
}

fn generate_create_input(model: &Model, dir: &Path) -> Result<()> {
    let names = get_prisma_name(&model.name);
    let mut content = String::new();
    let used_enums = collect_enum_types(model);

    content.push_str("import { builder } from \"../builder\";\n");
    if !used_enums.is_empty() {
        let enum_imports: Vec<String> = used_enums.into_iter().collect();
        content.push_str(&format!("import {{ {} }} from \"../enums\";\n", enum_imports.join(", ")));
    }
    
    // Import relation input types for nested relations
    let mut relation_imports: Vec<String> = Vec::new();
    for field in &model.fields {
        if let Some(_relation) = &field.relation {
            if let FieldType::Model(related_model) = &field.field_type {
                let relation_input_name = if field.is_list {
                    format!("{}{}ListRelationInput", model.name, related_model)
                } else {
                    format!("{}{}RelationInput", model.name, related_model)
                };
                if !relation_imports.contains(&relation_input_name) {
                    relation_imports.push(relation_input_name);
                }
            }
        }
    }
    
    if !relation_imports.is_empty() {
        content.push_str(&format!("import {{ {} }} from \"./relations\";\n", relation_imports.join(", ")));
    }
    
    content.push('\n');

    let input_name = names.create_input;

    content.push_str(&format!(
        "export const {} = builder.inputType(\"{}\", {{\n",
        input_name, input_name
    ));
    content.push_str("  fields: (t) => ({\n");

    // Collect all foreign keys to skip them
    let mut foreign_keys: Vec<String> = Vec::new();
    for field in &model.fields {
        if let Some(relation) = &field.relation {
            if !relation.fields.is_empty() {
                foreign_keys.extend(relation.fields.clone());
            }
        }
    }

    for field in &model.fields {
        // Skip auto-generated fields and foreign keys
        if field.is_updated_at || field.name == "created_at" || field.name == "updated_at" || foreign_keys.contains(&field.name) {
            continue;
        }

        // Handle relation fields
        if let Some(relation) = &field.relation {
            // Include only relations WITH fields (the "owning" side with foreign key)
            // These are the ones where we can connect/create related records
            // Skip reverse relations (without fields) to avoid duplication
            if relation.fields.is_empty() {
                continue;
            }
            
            if let FieldType::Model(related_model) = &field.field_type {
                let relation_input_type = if field.is_list {
                    format!("{}{}ListRelationInput", model.name, related_model)
                } else {
                    format!("{}{}RelationInput", model.name, related_model)
                };
                let required_option = if field.is_required { ", required: true" } else { "" };
                content.push_str(&format!(
                    "    {}: t.field({{ type: {}{} }}),\n",
                    field.name, relation_input_type, required_option
                ));
            }
            continue;
        }

        let required = if field.is_id || !field.is_required || field.default_value.is_some() {
            ""
        } else {
            "required: true"
        };

        let field_code = generate_input_field(&field.field_type, &field.name, field.is_list, required);
        content.push_str(&format!("    {},\n", field_code));
    }

    content.push_str("  }),\n");
    content.push_str("});\n");

    fs::write(dir.join(format!("{}.ts", input_name)), content)?;

    Ok(())
}

fn generate_update_input(model: &Model, dir: &Path) -> Result<()> {
    let names = get_prisma_name(&model.name);
    let mut content = String::new();
    let used_enums = collect_enum_types(model);

    content.push_str("import { builder } from \"../builder\";\n");
    if !used_enums.is_empty() {
        let enum_imports: Vec<String> = used_enums.into_iter().collect();
        content.push_str(&format!("import {{ {} }} from \"../enums\";\n", enum_imports.join(", ")));
    }
    
    // Import relation input types for nested relations
    let mut relation_imports: Vec<String> = Vec::new();
    for field in &model.fields {
        if let Some(_relation) = &field.relation {
            if let FieldType::Model(related_model) = &field.field_type {
                let relation_input_name = if field.is_list {
                    format!("{}{}ListRelationInput", model.name, related_model)
                } else {
                    format!("{}{}RelationInput", model.name, related_model)
                };
                if !relation_imports.contains(&relation_input_name) {
                    relation_imports.push(relation_input_name);
                }
            }
        }
    }
    
    if !relation_imports.is_empty() {
        content.push_str(&format!("import {{ {} }} from \"./relations\";\n", relation_imports.join(", ")));
    }
    
    content.push('\n');

    let input_name = names.update_input;

    content.push_str(&format!(
        "export const {} = builder.inputType(\"{}\", {{\n",
        input_name, input_name
    ));
    content.push_str("  fields: (t) => ({\n");

    // Collect all foreign keys to skip them
    let mut foreign_keys: Vec<String> = Vec::new();
    for field in &model.fields {
        if let Some(relation) = &field.relation {
            if !relation.fields.is_empty() {
                foreign_keys.extend(relation.fields.clone());
            }
        }
    }

    for field in &model.fields {
        if field.is_id || foreign_keys.contains(&field.name) {
            continue;
        }

        // Handle relation fields
        if let Some(relation) = &field.relation {
            // Include only relations WITH fields (the "owning" side with foreign key)
            // These are the ones where we can connect/create related records
            // Skip reverse relations (without fields) to avoid duplication
            if relation.fields.is_empty() {
                continue;
            }
            
            if let FieldType::Model(related_model) = &field.field_type {
                let relation_input_type = if field.is_list {
                    format!("{}{}ListRelationInput", model.name, related_model)
                } else {
                    format!("{}{}RelationInput", model.name, related_model)
                };
                content.push_str(&format!(
                    "    {}: t.field({{ type: {} }}),\n",
                    field.name, relation_input_type
                ));
            }
            continue;
        }

        let field_code = generate_input_field(&field.field_type, &field.name, field.is_list, "");
        content.push_str(&format!("    {},\n", field_code));
    }

    content.push_str("  }),\n");
    content.push_str("});\n");

    fs::write(dir.join(format!("{}.ts", input_name)), content)?;

    Ok(())
}

fn generate_where_input(model: &Model, dir: &Path) -> Result<()> {
    let names = get_prisma_name(&model.name);
    let mut content = String::new();

    content.push_str("import { builder } from \"../builder\";\n");
    content.push_str("import { StringFilter, IntFilter, FloatFilter, BoolFilter, DateTimeFilter } from \"./filters\";\n\n");

    let input_name = names.where_input;

    content.push_str(&format!(
        "export const {} = builder.inputType(\"{}\", {{\n",
        input_name, input_name
    ));
    content.push_str("  fields: (t) => ({\n");

    // AND, OR, NOT
    content.push_str(&format!(
        "    AND: t.field({{ type: ['{}'] as any }}),\n",
        input_name
    ));
    content.push_str(&format!(
        "    OR: t.field({{ type: ['{}'] as any }}),\n",
        input_name
    ));
    content.push_str(&format!(
        "    NOT: t.field({{ type: ['{}'] as any }}),\n",
        input_name
    ));

    // Scalar fields with filters
    for field in &model.fields {
        if field.relation.is_some() {
            continue;
        }

        let filter_type = get_filter_type(&field.field_type);
        content.push_str(&format!(
            "    {}: t.field({{ type: {} }}),\n",
            field.name, filter_type
        ));
    }

    content.push_str("  }),\n");
    content.push_str("});\n");

    fs::write(dir.join(format!("{}.ts", input_name)), content)?;

    Ok(())
}

fn generate_where_unique_input(model: &Model, dir: &Path) -> Result<()> {
    let names = get_prisma_name(&model.name);
    let mut content = String::new();

    content.push_str("import { builder } from \"../builder\";\n\n");

    let input_name = names.where_unique_input;

    content.push_str(&format!(
        "export const {} = builder.inputType(\"{}\", {{\n",
        input_name, input_name
    ));
    content.push_str("  fields: (t) => ({\n");

    // ID and unique fields
    for field in &model.fields {
        if field.is_id || field.is_unique {
            let field_code = generate_input_field(&field.field_type, &field.name, false, "required: true");
            content.push_str(&format!("    {},\n", field_code));
        }
    }

    content.push_str("  }),\n");
    content.push_str("});\n");

    fs::write(dir.join(format!("{}.ts", input_name)), content)?;

    Ok(())
}

fn generate_order_by_input(model: &Model, dir: &Path) -> Result<()> {
    let names = get_prisma_name(&model.name);
    let mut content = String::new();

    content.push_str("import { builder } from \"../builder\";\n");
    content.push_str("import { SortOrder } from \"../enums\";\n\n");

    let input_name = names.order_by_input;

    content.push_str(&format!(
        "export const {} = builder.inputType(\"{}\", {{\n",
        input_name, input_name
    ));
    content.push_str("  fields: (t) => ({\n");

    for field in &model.fields {
        if field.relation.is_some() {
            continue;
        }

        content.push_str(&format!(
            "    {}: t.field({{ type: SortOrder }}),\n",
            field.name
        ));
    }

    content.push_str("  }),\n");
    content.push_str("});\n");

    fs::write(dir.join(format!("{}.ts", input_name)), content)?;

    Ok(())
}

/// Generate a single input field code
fn generate_input_field(field_type: &FieldType, name: &str, is_list: bool, required: &str) -> String {
    let required_opt = if required.is_empty() { "".to_string() } else { format!(", {}", required) };

    match field_type {
        FieldType::String => {
            if is_list {
                format!("{}: t.stringList({{{}}})", name, required)
            } else {
                format!("{}: t.string({{{}}})", name, required)
            }
        }
        FieldType::Int => {
            if is_list {
                format!("{}: t.intList({{{}}})", name, required)
            } else {
                format!("{}: t.int({{{}}})", name, required)
            }
        }
        FieldType::Float => {
            if is_list {
                format!("{}: t.field({{ type: [\"Float\"]{}}})", name, required_opt)
            } else {
                format!("{}: t.float({{{}}})", name, required)
            }
        }
        FieldType::Boolean => {
            if is_list {
                format!("{}: t.booleanList({{{}}})", name, required)
            } else {
                format!("{}: t.boolean({{{}}})", name, required)
            }
        }
        FieldType::DateTime => {
            if is_list {
                format!("{}: t.field({{ type: [\"DateTime\"]{}}})", name, required_opt)
            } else {
                format!("{}: t.field({{ type: \"DateTime\"{}}})", name, required_opt)
            }
        }
        FieldType::Json => {
            if is_list {
                format!("{}: t.field({{ type: [\"JSON\"]{}}})", name, required_opt)
            } else {
                format!("{}: t.field({{ type: \"JSON\"{}}})", name, required_opt)
            }
        }
        FieldType::Decimal => {
            // Decimal input as float
            if is_list {
                format!("{}: t.field({{ type: [\"Float\"]{}}})", name, required_opt)
            } else {
                format!("{}: t.float({{{}}})", name, required)
            }
        }
        FieldType::BigInt => {
            // BigInt input as string
            if is_list {
                format!("{}: t.stringList({{{}}})", name, required)
            } else {
                format!("{}: t.string({{{}}})", name, required)
            }
        }
        FieldType::Bytes => {
            // Bytes input as string (base64)
            if is_list {
                format!("{}: t.stringList({{{}}})", name, required)
            } else {
                format!("{}: t.string({{{}}})", name, required)
            }
        }
        FieldType::Enum(enum_name) => {
            if is_list {
                format!("{}: t.field({{ type: [{}]{}}})", name, enum_name, required_opt)
            } else {
                format!("{}: t.field({{ type: {}{}}})", name, enum_name, required_opt)
            }
        }
        FieldType::Model(_) => {
            // This shouldn't happen for input fields
            format!("{}: t.string({{{}}})", name, required)
        }
    }
}

fn get_filter_type(field_type: &FieldType) -> String {
    match field_type {
        FieldType::String => "StringFilter".to_string(),
        FieldType::Int => "IntFilter".to_string(),
        FieldType::Float => "FloatFilter".to_string(),
        FieldType::Boolean => "BoolFilter".to_string(),
        FieldType::DateTime => "DateTimeFilter".to_string(),
        FieldType::Json => "\"JSON\"".to_string(),
        FieldType::Decimal => "FloatFilter".to_string(),
        FieldType::BigInt => "StringFilter".to_string(),
        FieldType::Bytes => "StringFilter".to_string(),
        FieldType::Enum(_) => "StringFilter".to_string(),
        FieldType::Model(_) => "StringFilter".to_string(),
    }
}

/// Generate WhereUnique input for relations (used in connect operations)
/// This is similar to the regular WhereUniqueInput but specifically for relation operations
fn generate_where_unique_input_for_relations(model: &Model, dir: &Path) -> Result<()> {
    let _names = get_prisma_name(&model.name);
    let mut content = String::new();

    content.push_str("import { builder } from \"../builder\";\n\n");

    let input_name = format!("{}WhereUniqueRelationInput", model.name);

    content.push_str(&format!(
        "export const {} = builder.inputType(\"{}\", {{\n",
        input_name, input_name
    ));
    content.push_str("  fields: (t) => ({\n");

    // ID and unique fields only
    for field in &model.fields {
        if field.is_id || field.is_unique {
            if field.relation.is_some() {
                continue; // Skip relation fields in WhereUnique
            }
            let field_code = generate_input_field(&field.field_type, &field.name, false, "");
            content.push_str(&format!("    {},\n", field_code));
        }
    }

    content.push_str("  }),\n");
    content.push_str("});\n");

    fs::write(dir.join(format!("{}.ts", input_name)), content)?;

    Ok(())
}

/// Generate RelationCreate input (like CreateInput but without reverse relations to avoid circular deps)
fn generate_relation_create_input(model: &Model, dir: &Path) -> Result<()> {
    let _names = get_prisma_name(&model.name);
    let mut content = String::new();
    let used_enums = collect_enum_types(model);

    content.push_str("import { builder } from \"../builder\";\n");
    if !used_enums.is_empty() {
        let enum_imports: Vec<String> = used_enums.into_iter().collect();
        content.push_str(&format!("import {{ {} }} from \"../enums\";\n", enum_imports.join(", ")));
    }
    
    // Import relation input types for nested relations
    let mut relation_imports: Vec<String> = Vec::new();
    for field in &model.fields {
        if let Some(_relation) = &field.relation {
            if let FieldType::Model(related_model) = &field.field_type {
                let relation_input_name = if field.is_list {
                    format!("{}{}ListRelationInput", model.name, related_model)
                } else {
                    format!("{}{}RelationInput", model.name, related_model)
                };
                if !relation_imports.contains(&relation_input_name) {
                    relation_imports.push(relation_input_name);
                }
            }
        }
    }
    
    if !relation_imports.is_empty() {
        content.push_str(&format!("import {{ {} }} from \"./relations\";\n", relation_imports.join(", ")));
    }
    
    content.push('\n');

    let input_name = format!("{}RelationCreateInput", model.name);

    content.push_str(&format!(
        "export const {} = builder.inputType(\"{}\", {{\n",
        input_name, input_name
    ));
    content.push_str("  fields: (t) => ({\n");

    // Collect all foreign keys to skip them
    let mut foreign_keys: Vec<String> = Vec::new();
    for field in &model.fields {
        if let Some(relation) = &field.relation {
            if !relation.fields.is_empty() {
                foreign_keys.extend(relation.fields.clone());
            }
        }
    }

    for field in &model.fields {
        // Skip auto-generated fields and foreign keys
        if field.is_updated_at || field.name == "created_at" || field.name == "updated_at" || foreign_keys.contains(&field.name) {
            continue;
        }

        // For relation fields, add the nested relation input
        if let Some(relation) = &field.relation {
            // Include only relations WITH fields (the "owning" side with foreign key)
            // These are the ones where we can connect/create related records
            // Skip reverse relations (without fields) to avoid duplication
            if relation.fields.is_empty() {
                continue;
            }
            
            if let FieldType::Model(related_model) = &field.field_type {
                let relation_input_type = if field.is_list {
                    format!("{}{}ListRelationInput", model.name, related_model)
                } else {
                    format!("{}{}RelationInput", model.name, related_model)
                };
                let required_option = if field.is_required { ", required: true" } else { "" };
                content.push_str(&format!(
                    "    {}: t.field({{ type: {}{} }}),\n",
                    field.name, relation_input_type, required_option
                ));
            }
            continue;
        }

        let required = if field.is_id || !field.is_required || field.default_value.is_some() {
            ""
        } else {
            "required: true"
        };

        let field_code = generate_input_field(&field.field_type, &field.name, field.is_list, required);
        content.push_str(&format!("    {},\n", field_code));
    }

    content.push_str("  }),\n");
    content.push_str("});\n");

    fs::write(dir.join(format!("{}.ts", input_name)), content)?;

    Ok(())
}
