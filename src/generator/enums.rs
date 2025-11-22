use crate::parser::ParsedSchema;
use anyhow::Result;
use std::fs;
use std::path::Path;

/// Generate base enums for Pothos (SortOrder, QueryMode, NullsOrder)
pub fn generate_base_enums(output_dir: &Path) -> Result<()> {
    let enums_dir = output_dir.join("enums");
    fs::create_dir_all(&enums_dir)?;

    let content = r#"import { builder } from "../builder";

// SortOrder enum
export const SortOrder = builder.enumType("SortOrder", {
  values: ["asc", "desc"] as const,
});

// NullsOrder enum
export const NullsOrder = builder.enumType("NullsOrder", {
  values: ["first", "last"] as const,
});

// QueryMode enum
export const QueryMode = builder.enumType("QueryMode", {
  values: ["default", "insensitive"] as const,
});
"#;

    fs::write(enums_dir.join("index.ts"), content)?;

    Ok(())
}

/// Generate schema-defined enums
pub fn generate_schema_enums(schema: &ParsedSchema, output_dir: &Path) -> Result<()> {
    let enums_dir = output_dir.join("enums");

    // Append schema enums to the index file
    let mut content = fs::read_to_string(enums_dir.join("index.ts")).unwrap_or_default();

    for enum_def in &schema.enums {
        content.push_str(&format!(
            "\n// {} enum\nexport const {} = builder.enumType(\"{}\", {{\n  values: [",
            enum_def.name, enum_def.name, enum_def.name
        ));

        let values: Vec<String> = enum_def
            .values
            .iter()
            .map(|v| format!("\"{}\"", v.name))
            .collect();
        content.push_str(&values.join(", "));

        content.push_str("] as const,\n});\n");
    }

    fs::write(enums_dir.join("index.ts"), content)?;

    Ok(())
}
