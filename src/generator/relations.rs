use crate::parser::ParsedSchema;
use anyhow::Result;
use std::fs;
use std::path::Path;

/// Generate all relation input types for all models
pub fn generate_all_relation_inputs(schema: &ParsedSchema, output_dir: &Path) -> Result<()> {
    let inputs_dir = output_dir.join("inputs");
    fs::create_dir_all(&inputs_dir)?;

    let mut all_relation_types = Vec::new();

    // Collect all relation input types needed
    for model in &schema.models {
        for field in &model.fields {
            if let Some(_relation) = &field.relation {
                if let crate::parser::FieldType::Model(related_model) = &field.field_type {
                    let relation_input_name = if field.is_list {
                        format!("{}{}ListRelationInput", model.name, related_model)
                    } else {
                        format!("{}{}RelationInput", model.name, related_model)
                    };
                    
                    if !all_relation_types.iter().any(|(name, _, _, _)| name == &relation_input_name) {
                        all_relation_types.push((
                            relation_input_name,
                            model.name.clone(),
                            related_model.clone(),
                            field.is_list,
                        ));
                    }
                }
            }
        }
    }

    // Generate a single relations.ts file with all relation input types
    generate_relations_file(&all_relation_types, &inputs_dir)?;

    Ok(())
}

fn generate_relations_file(
    relation_types: &[(String, String, String, bool)],
    dir: &Path,
) -> Result<()> {
    let mut content = String::new();

    content.push_str("import { builder } from \"../builder\";\n");
    
    // Import all WhereUniqueRelationInput and RelationCreateInput types
    let mut unique_models: Vec<String> = Vec::new();
    for (_, _, related_model, _) in relation_types {
        if !unique_models.contains(related_model) {
            unique_models.push(related_model.clone());
        }
    }
    
    for model in &unique_models {
        content.push_str(&format!(
            "import {{ {}WhereUniqueRelationInput }} from \"./{}WhereUniqueRelationInput\";\n",
            model, model
        ));
        content.push_str(&format!(
            "import {{ {}RelationCreateInput }} from \"./{}RelationCreateInput\";\n",
            model, model
        ));
    }
    
    content.push('\n');

    // Generate each relation input type
    for (input_name, _parent_model, related_model, is_list) in relation_types {
        content.push_str(&format!(
            "export const {} = builder.inputType(\"{}\", {{\n",
            input_name, input_name
        ));
        content.push_str("  fields: (t) => ({\n");

        if *is_list {
            // For list relations: connect, create, disconnect
            content.push_str(&format!(
                "    connect: t.field({{ type: [{}WhereUniqueRelationInput] }}),\n",
                related_model
            ));
            content.push_str(&format!(
                "    create: t.field({{ type: [{}RelationCreateInput] }}),\n",
                related_model
            ));
            content.push_str(&format!(
                "    disconnect: t.field({{ type: [{}WhereUniqueRelationInput] }}),\n",
                related_model
            ));
        } else {
            // For single relations: connect, create, disconnect
            content.push_str(&format!(
                "    connect: t.field({{ type: {}WhereUniqueRelationInput }}),\n",
                related_model
            ));
            content.push_str(&format!(
                "    create: t.field({{ type: {}RelationCreateInput }}),\n",
                related_model
            ));
            content.push_str(&format!(
                "    disconnect: t.field({{ type: {}WhereUniqueRelationInput }}),\n",
                related_model
            ));
        }

        content.push_str("  }),\n");
        content.push_str("});\n\n");
    }

    fs::write(dir.join("relations.ts"), content)?;

    Ok(())
}
