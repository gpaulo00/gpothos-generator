use crate::parser::{FieldType, Model};
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

    Ok(())
}

fn generate_create_input(model: &Model, dir: &Path) -> Result<()> {
    let mut content = String::new();
    let used_enums = collect_enum_types(model);

    content.push_str("import { builder } from \"../builder\";\n");
    if !used_enums.is_empty() {
        let enum_imports: Vec<String> = used_enums.into_iter().collect();
        content.push_str(&format!("import {{ {} }} from \"../enums\";\n", enum_imports.join(", ")));
    }
    content.push('\n');

    let input_name = format!("{}CreateInput", model.name);

    content.push_str(&format!(
        "export const {} = builder.inputType(\"{}\", {{\n",
        input_name, input_name
    ));
    content.push_str("  fields: (t) => ({\n");

    for field in &model.fields {
        if field.relation.is_some() {
            continue;
        }

        // Skip auto-generated fields
        if field.is_updated_at || field.name == "created_at" || field.name == "updated_at" {
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
    let mut content = String::new();
    let used_enums = collect_enum_types(model);

    content.push_str("import { builder } from \"../builder\";\n");
    if !used_enums.is_empty() {
        let enum_imports: Vec<String> = used_enums.into_iter().collect();
        content.push_str(&format!("import {{ {} }} from \"../enums\";\n", enum_imports.join(", ")));
    }
    content.push('\n');

    let input_name = format!("{}UpdateInput", model.name);

    content.push_str(&format!(
        "export const {} = builder.inputType(\"{}\", {{\n",
        input_name, input_name
    ));
    content.push_str("  fields: (t) => ({\n");

    for field in &model.fields {
        if field.relation.is_some() || field.is_id {
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
    let mut content = String::new();

    content.push_str("import { builder } from \"../builder\";\n");
    content.push_str("import { StringFilter, IntFilter, FloatFilter, BoolFilter, DateTimeFilter } from \"./filters\";\n\n");

    let input_name = format!("{}WhereInput", model.name);

    content.push_str(&format!(
        "export const {} = builder.inputType(\"{}\", {{\n",
        input_name, input_name
    ));
    content.push_str("  fields: (t) => ({\n");

    // AND, OR, NOT
    content.push_str(&format!(
        "    AND: t.field({{ type: [{}] }}),\n",
        input_name
    ));
    content.push_str(&format!(
        "    OR: t.field({{ type: [{}] }}),\n",
        input_name
    ));
    content.push_str(&format!(
        "    NOT: t.field({{ type: [{}] }}),\n",
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
    let mut content = String::new();

    content.push_str("import { builder } from \"../builder\";\n\n");

    let input_name = format!("{}WhereUniqueInput", model.name);

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
    let mut content = String::new();

    content.push_str("import { builder } from \"../builder\";\n");
    content.push_str("import { SortOrder } from \"../enums\";\n\n");

    let input_name = format!("{}OrderByInput", model.name);

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
