use crate::parser::{FieldType, Model};
use anyhow::Result;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Generate Pothos PrismaObject type
pub fn generate_model(model: &Model, output_dir: &Path) -> Result<()> {
    let models_dir = output_dir.join("models");
    fs::create_dir_all(&models_dir)?;

    // Collect all enum types used in this model
    let mut used_enums: HashSet<String> = HashSet::new();
    for field in &model.fields {
        if let FieldType::Enum(enum_name) = &field.field_type {
            used_enums.insert(enum_name.clone());
        }
    }

    let mut content = String::new();

    // Imports
    content.push_str("import { builder } from \"../builder\";\n");

    // Import used enums
    if !used_enums.is_empty() {
        let enum_imports: Vec<String> = used_enums.into_iter().collect();
        content.push_str(&format!("import {{ {} }} from \"../enums\";\n", enum_imports.join(", ")));
    }
    content.push('\n');

    // PrismaObject definition
    content.push_str(&format!(
        "export const {} = builder.prismaObject(\"{}\", {{\n",
        model.name, model.name
    ));
    content.push_str("  fields: (t) => ({\n");

    // Scalar fields
    for field in &model.fields {
        if field.relation.is_some() {
            continue; // Relations handled separately
        }

        let field_code = generate_field_code(field);
        content.push_str(&format!("    {},\n", field_code));
    }

    // Relation fields
    for field in &model.fields {
        if field.relation.is_none() {
            continue;
        }

        let nullable = if field.is_required { "" } else { "nullable: true" };

        if field.is_list {
            content.push_str(&format!(
                "    {}: t.relation(\"{}\", {{\n",
                field.name, field.name
            ));
            content.push_str("      query: () => ({}),\n");
            content.push_str("    }),\n");
        } else {
            content.push_str(&format!(
                "    {}: t.relation(\"{}\", {{ {} }}),\n",
                field.name, field.name, nullable
            ));
        }
    }

    content.push_str("  }),\n");
    content.push_str("});\n");

    fs::write(models_dir.join(format!("{}.ts", model.name)), content)?;

    Ok(())
}

fn generate_field_code(field: &crate::parser::Field) -> String {
    let nullable_opt = if field.is_required { "" } else { "nullable: true" };

    match &field.field_type {
        // Simple expose methods for basic types
        FieldType::String => {
            if field.is_list {
                format!("{}: t.exposeStringList(\"{}\", {{ {} }})",
                    field.name, field.name, nullable_opt)
            } else if field.is_id {
                format!("{}: t.exposeID(\"{}\"{})",
                    field.name, field.name,
                    if !field.is_required { ", { nullable: true }" } else { "" })
            } else {
                format!("{}: t.exposeString(\"{}\"{})",
                    field.name, field.name,
                    if !field.is_required { ", { nullable: true }" } else { "" })
            }
        }
        FieldType::Int => {
            if field.is_list {
                format!("{}: t.exposeIntList(\"{}\", {{ {} }})",
                    field.name, field.name, nullable_opt)
            } else {
                format!("{}: t.exposeInt(\"{}\"{})",
                    field.name, field.name,
                    if !field.is_required { ", { nullable: true }" } else { "" })
            }
        }
        FieldType::Float => {
            if field.is_list {
                format!("{}: t.exposeFloatList(\"{}\", {{ {} }})",
                    field.name, field.name, nullable_opt)
            } else {
                format!("{}: t.exposeFloat(\"{}\"{})",
                    field.name, field.name,
                    if !field.is_required { ", { nullable: true }" } else { "" })
            }
        }
        FieldType::Boolean => {
            if field.is_list {
                format!("{}: t.exposeBooleanList(\"{}\", {{ {} }})",
                    field.name, field.name, nullable_opt)
            } else {
                format!("{}: t.exposeBoolean(\"{}\"{})",
                    field.name, field.name,
                    if !field.is_required { ", { nullable: true }" } else { "" })
            }
        }
        // Types that need explicit type specification
        FieldType::DateTime => {
            if field.is_list {
                format!("{}: t.field({{ type: [\"DateTime\"], resolve: (parent) => parent.{}, {} }})",
                    field.name, field.name, nullable_opt)
            } else {
                format!("{}: t.field({{ type: \"DateTime\", resolve: (parent) => parent.{}{} }})",
                    field.name, field.name,
                    if !field.is_required { ", nullable: true" } else { "" })
            }
        }
        FieldType::Json => {
            if field.is_list {
                format!("{}: t.field({{ type: [\"JSON\"], resolve: (parent) => parent.{}, {} }})",
                    field.name, field.name, nullable_opt)
            } else {
                format!("{}: t.field({{ type: \"JSON\", resolve: (parent) => parent.{}{} }})",
                    field.name, field.name,
                    if !field.is_required { ", nullable: true" } else { "" })
            }
        }
        FieldType::Decimal => {
            // Decimal typically serializes to string
            if field.is_list {
                format!("{}: t.field({{ type: [\"String\"], resolve: (parent) => parent.{}?.map(d => d.toString()), {} }})",
                    field.name, field.name, nullable_opt)
            } else {
                format!("{}: t.field({{ type: \"String\", resolve: (parent) => parent.{}?.toString(){} }})",
                    field.name, field.name,
                    if !field.is_required { ", nullable: true" } else { "" })
            }
        }
        FieldType::BigInt => {
            // BigInt serializes to string
            if field.is_list {
                format!("{}: t.field({{ type: [\"String\"], resolve: (parent) => parent.{}?.map(b => b.toString()), {} }})",
                    field.name, field.name, nullable_opt)
            } else {
                format!("{}: t.field({{ type: \"String\", resolve: (parent) => parent.{}?.toString(){} }})",
                    field.name, field.name,
                    if !field.is_required { ", nullable: true" } else { "" })
            }
        }
        FieldType::Bytes => {
            // Bytes serializes to base64 string
            if field.is_list {
                format!("{}: t.field({{ type: [\"String\"], resolve: (parent) => parent.{}?.map(b => b.toString('base64')), {} }})",
                    field.name, field.name, nullable_opt)
            } else {
                format!("{}: t.field({{ type: \"String\", resolve: (parent) => parent.{}?.toString('base64'){} }})",
                    field.name, field.name,
                    if !field.is_required { ", nullable: true" } else { "" })
            }
        }
        FieldType::Enum(enum_name) => {
            if field.is_list {
                format!("{}: t.field({{ type: [{}], resolve: (parent) => parent.{}, {} }})",
                    field.name, enum_name, field.name, nullable_opt)
            } else {
                format!("{}: t.field({{ type: {}, resolve: (parent) => parent.{}{} }})",
                    field.name, enum_name, field.name,
                    if !field.is_required { ", nullable: true" } else { "" })
            }
        }
        FieldType::Model(_) => {
            // This shouldn't happen for non-relation fields, but handle it
            format!("{}: t.exposeString(\"{}\"{})",
                field.name, field.name,
                if !field.is_required { ", { nullable: true }" } else { "" })
        }
    }
}
