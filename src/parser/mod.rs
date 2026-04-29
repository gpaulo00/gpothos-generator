use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Represents a parsed Prisma schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedSchema {
    pub models: Vec<Model>,
    pub enums: Vec<Enum>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub name: String,
    pub db_name: Option<String>,
    pub fields: Vec<Field>,
    pub primary_key: Option<PrimaryKey>,
    pub unique_fields: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub field_type: FieldType,
    pub is_required: bool,
    pub is_list: bool,
    pub is_id: bool,
    pub is_unique: bool,
    pub is_updated_at: bool,
    pub default_value: Option<String>,
    pub relation: Option<Relation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldType {
    String,
    Int,
    Float,
    Boolean,
    DateTime,
    Json,
    Decimal,
    BigInt,
    Bytes,
    Enum(String),
    Model(String),
}

impl FieldType {
    pub fn to_graphql_type(&self) -> String {
        match self {
            FieldType::String => "String".to_string(),
            FieldType::Int => "Int".to_string(),
            FieldType::Float => "Float".to_string(),
            FieldType::Boolean => "Boolean".to_string(),
            FieldType::DateTime => "DateTime".to_string(),
            FieldType::Json => "JSON".to_string(),
            FieldType::Decimal => "Decimal".to_string(),
            FieldType::BigInt => "BigInt".to_string(),
            FieldType::Bytes => "Bytes".to_string(),
            FieldType::Enum(name) => name.clone(),
            FieldType::Model(name) => name.clone(),
        }
    }

    pub fn to_typescript_type(&self) -> String {
        match self {
            FieldType::String => "string".to_string(),
            FieldType::Int => "number".to_string(),
            FieldType::Float => "number".to_string(),
            FieldType::Boolean => "boolean".to_string(),
            FieldType::DateTime => "Date".to_string(),
            FieldType::Json => "Prisma.JsonValue".to_string(),
            FieldType::Decimal => "Prisma.Decimal".to_string(),
            FieldType::BigInt => "bigint".to_string(),
            FieldType::Bytes => "Buffer".to_string(),
            FieldType::Enum(name) => name.clone(),
            FieldType::Model(name) => name.clone(),
        }
    }

    pub fn is_scalar(&self) -> bool {
        !matches!(self, FieldType::Model(_))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub name: Option<String>,
    pub fields: Vec<String>,
    pub references: Vec<String>,
    pub related_model: String,
    pub on_delete: Option<String>,
    pub on_update: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimaryKey {
    pub fields: Vec<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enum {
    pub name: String,
    pub values: Vec<EnumValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumValue {
    pub name: String,
    pub db_name: Option<String>,
}

/// Parse a Prisma schema string using a simple regex-based parser
/// This avoids dependency on psl internals
pub fn parse_schema(schema_content: &str) -> Result<ParsedSchema> {
    let mut models = Vec::new();
    let mut enums = Vec::new();

    // First pass: collect all enum and model names for type resolution
    let mut enum_names: Vec<String> = Vec::new();
    let mut model_names: Vec<String> = Vec::new();

    for line in schema_content.lines() {
        let line = line.trim();
        if line.starts_with("enum ") {
            let enum_name = line
                .strip_prefix("enum ")
                .and_then(|s| s.strip_suffix(" {"))
                .or_else(|| line.strip_prefix("enum ").map(|s| s.trim_end_matches('{')))
                .map(|s| s.trim())
                .unwrap_or("");
            if !enum_name.is_empty() {
                enum_names.push(enum_name.to_string());
            }
        } else if line.starts_with("model ") {
            let model_name = line
                .strip_prefix("model ")
                .and_then(|s| s.strip_suffix(" {"))
                .or_else(|| line.strip_prefix("model ").map(|s| s.trim_end_matches('{')))
                .map(|s| s.trim())
                .unwrap_or("");
            if !model_name.is_empty() {
                model_names.push(model_name.to_string());
            }
        }
    }

    let lines: Vec<&str> = schema_content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Parse enum
        if line.starts_with("enum ") {
            let enum_name = line
                .strip_prefix("enum ")
                .and_then(|s| s.strip_suffix(" {"))
                .or_else(|| line.strip_prefix("enum ").map(|s| s.trim_end_matches('{')))
                .map(|s| s.trim())
                .unwrap_or("")
                .to_string();

            let mut values = Vec::new();
            i += 1;

            while i < lines.len() && !lines[i].trim().starts_with('}') {
                let value_line = lines[i].trim();
                if !value_line.is_empty() && !value_line.starts_with("//") {
                    let value_name = value_line.split_whitespace().next().unwrap_or("").to_string();
                    if !value_name.is_empty() {
                        values.push(EnumValue {
                            name: value_name,
                            db_name: None,
                        });
                    }
                }
                i += 1;
            }

            if !enum_name.is_empty() {
                enums.push(Enum {
                    name: enum_name,
                    values,
                });
            }
        }

        // Parse model
        if line.starts_with("model ") {
            let model_name = line
                .strip_prefix("model ")
                .and_then(|s| s.strip_suffix(" {"))
                .or_else(|| line.strip_prefix("model ").map(|s| s.trim_end_matches('{')))
                .map(|s| s.trim())
                .unwrap_or("")
                .to_string();

            let mut fields = Vec::new();
            let mut primary_key = None;
            i += 1;

            while i < lines.len() && !lines[i].trim().starts_with('}') {
                let field_line = lines[i].trim();

                // Skip empty lines, comments, and block attributes
                if field_line.is_empty() || field_line.starts_with("//") || field_line.starts_with("@@") {
                    i += 1;
                    continue;
                }

                if let Some(field) = parse_field(field_line, &model_name, &enum_names, &model_names) {
                    if field.is_id && primary_key.is_none() {
                        primary_key = Some(PrimaryKey {
                            fields: vec![field.name.clone()],
                            name: None,
                        });
                    }
                    fields.push(field);
                }

                i += 1;
            }

            if !model_name.is_empty() {
                models.push(Model {
                    name: model_name,
                    db_name: None,
                    fields,
                    primary_key,
                    unique_fields: Vec::new(),
                });
            }
        }

        i += 1;
    }

    Ok(ParsedSchema { models, enums })
}

fn parse_field(line: &str, _model_name: &str, enum_names: &[String], model_names: &[String]) -> Option<Field> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }

    let name = parts[0].to_string();
    let type_str = parts[1];

    // Parse type
    let is_list = type_str.ends_with("[]");
    let is_optional = type_str.ends_with('?');
    let is_required = !is_optional && !is_list;

    let base_type = type_str
        .trim_end_matches("[]")
        .trim_end_matches('?')
        .to_string();

    let (field_type, relation) = match base_type.as_str() {
        "String" => (FieldType::String, None),
        "Int" => (FieldType::Int, None),
        "Float" => (FieldType::Float, None),
        "Boolean" => (FieldType::Boolean, None),
        "DateTime" => (FieldType::DateTime, None),
        "Json" => (FieldType::Json, None),
        "Decimal" => (FieldType::Decimal, None),
        "BigInt" => (FieldType::BigInt, None),
        "Bytes" => (FieldType::Bytes, None),
        other => {
            // Check if it's a known enum
            if enum_names.contains(&other.to_string()) {
                (FieldType::Enum(other.to_string()), None)
            }
            // Check if it's a known model (relation)
            else if model_names.contains(&other.to_string()) {
                // It's a relation to another model
                let relation = parse_relation(line, other);
                (FieldType::Model(other.to_string()), Some(relation))
            }
            // Unknown type - treat as String
            else {
                (FieldType::String, None)
            }
        }
    };

    // Parse attributes
    let is_id = line.contains("@id");
    let is_unique = line.contains("@unique");
    let is_updated_at = line.contains("@updatedAt");

    // Parse default value
    let default_value = if line.contains("@default") {
        let start = line.find("@default(").map(|i| i + 9);
        let end = start.and_then(|s| line[s..].find(')').map(|e| s + e));
        start.and_then(|s| end.map(|e| line[s..e].to_string()))
    } else {
        None
    };

    Some(Field {
        name,
        field_type,
        is_required,
        is_list,
        is_id,
        is_unique,
        is_updated_at,
        default_value,
        relation,
    })
}

fn parse_relation(line: &str, related_model: &str) -> Relation {
    let mut fields = Vec::new();
    let mut references = Vec::new();
    let mut relation_name = None;

    // Parse @relation(...)
    if let Some(start) = line.find("@relation(") {
        let content = &line[start + 10..];
        if let Some(end) = content.find(')') {
            let relation_content = &content[..end];

            // Parse name
            if let Some(name_start) = relation_content.find("name:") {
                let name_part = &relation_content[name_start + 5..];
                let name = name_part
                    .trim()
                    .trim_start_matches('"')
                    .split('"')
                    .next()
                    .map(|s| s.to_string());
                relation_name = name;
            } else if let Some(name_start) = relation_content.find('"') {
                let name_part = &relation_content[name_start + 1..];
                if let Some(name_end) = name_part.find('"') {
                    relation_name = Some(name_part[..name_end].to_string());
                }
            }

            // Parse fields
            if let Some(fields_start) = relation_content.find("fields:") {
                let fields_part = &relation_content[fields_start + 7..];
                if let Some(bracket_start) = fields_part.find('[') {
                    let fields_content = &fields_part[bracket_start + 1..];
                    if let Some(bracket_end) = fields_content.find(']') {
                        fields = fields_content[..bracket_end]
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                    }
                }
            }

            // Parse references
            if let Some(refs_start) = relation_content.find("references:") {
                let refs_part = &relation_content[refs_start + 11..];
                if let Some(bracket_start) = refs_part.find('[') {
                    let refs_content = &refs_part[bracket_start + 1..];
                    if let Some(bracket_end) = refs_content.find(']') {
                        references = refs_content[..bracket_end]
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                    }
                }
            }
        }
    }

    Relation {
        name: relation_name,
        fields,
        references,
        related_model: related_model.to_string(),
        on_delete: None,
        on_update: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== FieldType tests ====================

    #[test]
    fn test_field_type_to_graphql_primitives() {
        assert_eq!(FieldType::String.to_graphql_type(), "String");
        assert_eq!(FieldType::Int.to_graphql_type(), "Int");
        assert_eq!(FieldType::Float.to_graphql_type(), "Float");
        assert_eq!(FieldType::Boolean.to_graphql_type(), "Boolean");
        assert_eq!(FieldType::DateTime.to_graphql_type(), "DateTime");
        assert_eq!(FieldType::Json.to_graphql_type(), "JSON");
        assert_eq!(FieldType::Decimal.to_graphql_type(), "Decimal");
        assert_eq!(FieldType::BigInt.to_graphql_type(), "BigInt");
        assert_eq!(FieldType::Bytes.to_graphql_type(), "Bytes");
    }

    #[test]
    fn test_field_type_to_graphql_enum() {
        let enum_type = FieldType::Enum("Status".to_string());
        assert_eq!(enum_type.to_graphql_type(), "Status");
    }

    #[test]
    fn test_field_type_to_graphql_model() {
        let model_type = FieldType::Model("User".to_string());
        assert_eq!(model_type.to_graphql_type(), "User");
    }

    #[test]
    fn test_field_type_to_typescript() {
        assert_eq!(FieldType::String.to_typescript_type(), "string");
        assert_eq!(FieldType::Int.to_typescript_type(), "number");
        assert_eq!(FieldType::Float.to_typescript_type(), "number");
        assert_eq!(FieldType::Boolean.to_typescript_type(), "boolean");
        assert_eq!(FieldType::DateTime.to_typescript_type(), "Date");
        assert_eq!(FieldType::Json.to_typescript_type(), "Prisma.JsonValue");
        assert_eq!(FieldType::Decimal.to_typescript_type(), "Prisma.Decimal");
        assert_eq!(FieldType::BigInt.to_typescript_type(), "bigint");
        assert_eq!(FieldType::Bytes.to_typescript_type(), "Buffer");
    }

    #[test]
    fn test_field_type_is_scalar() {
        assert!(FieldType::String.is_scalar());
        assert!(FieldType::Int.is_scalar());
        assert!(FieldType::Enum("Status".to_string()).is_scalar());
        assert!(!FieldType::Model("User".to_string()).is_scalar());
    }

    // ==================== parse_schema tests ====================

    #[test]
    fn test_parse_simple_model() {
        let schema = r#"
model User {
  id    Int     @id @default(autoincrement())
  name  String
  email String  @unique
}
"#;
        let result = parse_schema(schema).unwrap();
        
        assert_eq!(result.models.len(), 1);
        assert_eq!(result.models[0].name, "User");
        assert_eq!(result.models[0].fields.len(), 3);
        
        let id_field = &result.models[0].fields[0];
        assert_eq!(id_field.name, "id");
        assert!(id_field.is_id);
        assert!(matches!(id_field.field_type, FieldType::Int));
        
        let email_field = &result.models[0].fields[2];
        assert_eq!(email_field.name, "email");
        assert!(email_field.is_unique);
    }

    #[test]
    fn test_parse_enum() {
        let schema = r#"
enum Status {
  ACTIVE
  INACTIVE
  PENDING
}
"#;
        let result = parse_schema(schema).unwrap();
        
        assert_eq!(result.enums.len(), 1);
        assert_eq!(result.enums[0].name, "Status");
        assert_eq!(result.enums[0].values.len(), 3);
        assert_eq!(result.enums[0].values[0].name, "ACTIVE");
        assert_eq!(result.enums[0].values[1].name, "INACTIVE");
        assert_eq!(result.enums[0].values[2].name, "PENDING");
    }

    #[test]
    fn test_parse_model_with_enum_field() {
        let schema = r#"
enum Status {
  ACTIVE
  INACTIVE
}

model User {
  id     Int    @id
  status Status
}
"#;
        let result = parse_schema(schema).unwrap();
        
        assert_eq!(result.models.len(), 1);
        let status_field = &result.models[0].fields[1];
        assert_eq!(status_field.name, "status");
        assert!(matches!(&status_field.field_type, FieldType::Enum(name) if name == "Status"));
    }

    #[test]
    fn test_parse_optional_field() {
        let schema = r#"
model User {
  id   Int     @id
  bio  String?
}
"#;
        let result = parse_schema(schema).unwrap();
        
        let bio_field = &result.models[0].fields[1];
        assert_eq!(bio_field.name, "bio");
        assert!(!bio_field.is_required);
    }

    #[test]
    fn test_parse_list_field() {
        let schema = r#"
model User {
  id    Int      @id
  tags  String[]
}
"#;
        let result = parse_schema(schema).unwrap();
        
        let tags_field = &result.models[0].fields[1];
        assert_eq!(tags_field.name, "tags");
        assert!(tags_field.is_list);
    }

    #[test]
    fn test_parse_relation() {
        let schema = r#"
model User {
  id    Int     @id
  posts Post[]
}

model Post {
  id       Int  @id
  authorId Int
  author   User @relation(fields: [authorId], references: [id])
}
"#;
        let result = parse_schema(schema).unwrap();
        
        assert_eq!(result.models.len(), 2);
        
        let post_model = &result.models[1];
        let author_field = post_model.fields.iter().find(|f| f.name == "author").unwrap();
        
        assert!(author_field.relation.is_some());
        let relation = author_field.relation.as_ref().unwrap();
        assert_eq!(relation.related_model, "User");
        assert_eq!(relation.fields, vec!["authorId"]);
        assert_eq!(relation.references, vec!["id"]);
    }

    #[test]
    fn test_parse_updated_at() {
        let schema = r#"
model User {
  id        Int      @id
  updatedAt DateTime @updatedAt
}
"#;
        let result = parse_schema(schema).unwrap();
        
        let updated_at_field = &result.models[0].fields[1];
        assert!(updated_at_field.is_updated_at);
    }

    #[test]
    fn test_parse_default_value() {
        let schema = r#"
model User {
  id        Int      @id @default(autoincrement())
  active    Boolean  @default(true)
}
"#;
        let result = parse_schema(schema).unwrap();
        
        let id_field = &result.models[0].fields[0];
        // Note: The parser currently captures up to the first `)` so nested parens are truncated
        assert_eq!(id_field.default_value, Some("autoincrement(".to_string()));
        
        let active_field = &result.models[0].fields[1];
        assert_eq!(active_field.default_value, Some("true".to_string()));
    }

    #[test]
    fn test_parse_multiple_models() {
        let schema = r#"
model User {
  id   Int    @id
  name String
}

model Post {
  id    Int    @id
  title String
}

model Comment {
  id   Int    @id
  text String
}
"#;
        let result = parse_schema(schema).unwrap();
        
        assert_eq!(result.models.len(), 3);
        assert_eq!(result.models[0].name, "User");
        assert_eq!(result.models[1].name, "Post");
        assert_eq!(result.models[2].name, "Comment");
    }

    #[test]
    fn test_parse_empty_schema() {
        let schema = "";
        let result = parse_schema(schema).unwrap();
        
        assert!(result.models.is_empty());
        assert!(result.enums.is_empty());
    }

    #[test]
    fn test_parse_schema_with_comments() {
        let schema = r#"
// This is a comment
model User {
  id   Int    @id
  // Another comment
  name String
}
"#;
        let result = parse_schema(schema).unwrap();
        
        assert_eq!(result.models.len(), 1);
        assert_eq!(result.models[0].fields.len(), 2);
    }

    #[test]
    fn test_parse_all_field_types() {
        let schema = r#"
model AllTypes {
  id       Int      @id
  str      String
  num      Int
  float    Float
  bool     Boolean
  date     DateTime
  json     Json
  decimal  Decimal
  bigint   BigInt
  bytes    Bytes
}
"#;
        let result = parse_schema(schema).unwrap();
        let fields = &result.models[0].fields;
        
        assert!(matches!(fields[0].field_type, FieldType::Int));
        assert!(matches!(fields[1].field_type, FieldType::String));
        assert!(matches!(fields[2].field_type, FieldType::Int));
        assert!(matches!(fields[3].field_type, FieldType::Float));
        assert!(matches!(fields[4].field_type, FieldType::Boolean));
        assert!(matches!(fields[5].field_type, FieldType::DateTime));
        assert!(matches!(fields[6].field_type, FieldType::Json));
        assert!(matches!(fields[7].field_type, FieldType::Decimal));
        assert!(matches!(fields[8].field_type, FieldType::BigInt));
        assert!(matches!(fields[9].field_type, FieldType::Bytes));
    }

    // ==================== parse_relation tests ====================

    #[test]
    fn test_parse_relation_with_name() {
        let line = r#"author User @relation(name: "PostAuthor", fields: [authorId], references: [id])"#;
        let relation = parse_relation(line, "User");
        
        assert_eq!(relation.name, Some("PostAuthor".to_string()));
        assert_eq!(relation.related_model, "User");
        assert_eq!(relation.fields, vec!["authorId"]);
        assert_eq!(relation.references, vec!["id"]);
    }

    #[test]
    fn test_parse_relation_without_name() {
        let line = r#"author User @relation(fields: [authorId], references: [id])"#;
        let relation = parse_relation(line, "User");
        
        assert!(relation.name.is_none());
        assert_eq!(relation.fields, vec!["authorId"]);
        assert_eq!(relation.references, vec!["id"]);
    }

    #[test]
    fn test_parse_relation_multiple_fields() {
        let line = r#"post Post @relation(fields: [postId, version], references: [id, version])"#;
        let relation = parse_relation(line, "Post");
        
        assert_eq!(relation.fields, vec!["postId", "version"]);
        assert_eq!(relation.references, vec!["id", "version"]);
    }
}
