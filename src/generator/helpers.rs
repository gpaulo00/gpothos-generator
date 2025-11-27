use anyhow::Result;
use std::fs;
use std::path::Path;

/// Generate names according to the JavaScript getPrismaName function
pub fn get_prisma_name(model: &str) -> PrismaNames {
    // This implements: (model.charAt(0).toLowerCase() + model.slice(1))
    let mut chars = model.chars();
    let first_char = chars.next();
    let lower_first = match first_char {
        None => String::new(),
        Some(c) => c.to_lowercase().collect::<String>() + chars.as_str(),
    };

    // Apply the camel case transformation: /_([a-z])/g to uppercase for minusStar (used for find field)
    let camel_case = to_camel_case(&lower_first);

    PrismaNames {
        model: model.to_string(),
        update: format!("updateOne{}", model),
        create: format!("createOne{}", model),
        find: camel_case.clone(),  // This corresponds to minusStar with camelCase transformation
        find_many: pluralize_query_name(&camel_case), // This is based on minusStar (camelCase) with pluralization
        where_input: format!("{}WhereInput", model),
        where_unique_input: format!("{}WhereUniqueInput", model),
        order_by_input: format!("{}OrderByInput", model),
        create_input: format!("{}CreateInput", model),
        update_input: format!("{}UpdateInput", model),
        query_new: pluralize_find_many_name_original(&lower_first),  // Use lower_first version (without camelCase) for query_new
        query_new2: lower_first.clone(), // querynew2 is (model.charAt(0).toLowerCase() + model.slice(1)) WITHOUT camelCase transformation
    }
}

/// Convert snake_case to camelCase
fn to_camel_case(input: &str) -> String {
    let parts: Vec<&str> = input.split('_').collect();
    if parts.len() <= 1 {
        return input.to_string();
    }

    let first = parts[0];
    let rest: String = parts[1..]
        .iter()
        .map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect();

    format!("{}{}", first, rest)
}

/// Pluralize query name according to the JavaScript function (findMany)
/// This implements: minusStar.replace(/y$/g,'ie') + 's'
fn pluralize_query_name(s: &str) -> String {
    if s.ends_with('y') {
        format!("{}ie{}", &s[..s.len() - 1], 's')  // Replace y with 'ie' then add 's' as per JS
    } else {
        format!("{}s", s)
    }
}

/// Pluralize find many name according to the JavaScript function (querynew)
/// This implements: (model.toLowerCase...).replace(/y$/g,'ie') + 's', then replace(/ss$/g, 's')
fn pluralize_find_many_name_original(s: &str) -> String {
    let base = if s.ends_with('y') {
        format!("{}ie{}", &s[..s.len() - 1], 's')  // Replace y with 'ie' then add 's' as per JS
    } else {
        format!("{}s", s)
    };

    // Replace ss with s at the end as per JavaScript function
    if base.ends_with("ss") {
        format!("{}s", &base[..base.len() - 1])  // Remove one 's' to get "s" instead of "ss"
    } else {
        base
    }
}

/// Pluralize find many name according to the JavaScript function (findMany)
/// This implements: minusStar.replace(/y$/g,'ie') + 's'
fn pluralize_find_many_name(s: &str) -> String {
    if s.ends_with('y') {
        format!("{}ie{}", &s[..s.len() - 1], 's')  // Replace y with 'ie' then add 's' as per JS for findMany
    } else {
        format!("{}s", s)
    }
}

/// Struct to hold all the generated names for a model
#[derive(Debug)]
pub struct PrismaNames {
    pub model: String,
    pub update: String,
    pub create: String,
    pub find: String,
    pub find_many: String,
    pub where_input: String,
    pub where_unique_input: String,
    pub order_by_input: String,
    pub create_input: String,
    pub update_input: String,
    pub query_new: String,
    pub query_new2: String,
}

/// Generate builder file for Pothos
pub fn generate_helpers(output_dir: &Path) -> Result<()> {
    let content = r#"import SchemaBuilder from "@pothos/core";
import PrismaPlugin from "@pothos/plugin-prisma";
import { PrismaClient } from "@prisma/client";
import SimpleObjectsPlugin from '@pothos/plugin-simple-objects';
import type PrismaTypes from "@pothos/plugin-prisma/generated";

// Initialize Prisma Client
export const prisma = new PrismaClient();

// Define context type
export interface Context {
  prisma: PrismaClient;
}

// Initialize Pothos Builder with Prisma Plugin
export const builder = new SchemaBuilder<{
  // @ts-ignore
  PrismaTypes: PrismaTypes;
  Context: any; // TODO: put Context interface here (breaks)
  Scalars: {
    DateTime: {
      Input: Date;
      Output: Date;
    };
    JSON: {
      Input: unknown;
      Output: unknown;
    };
  };
  // @ts-ignore
}>({
  plugins: [PrismaPlugin, SimpleObjectsPlugin],
  prisma: {
    client: prisma,
    exposeDescriptions: true,
    filterConnectionTotalCount: true,
  },
});

// Add DateTime scalar
builder.scalarType("DateTime", {
  serialize: (value) => value.toISOString(),
  parseValue: (value) => new Date(value as string),
});

// Add JSON scalar
builder.scalarType("JSON", {
  serialize: (value) => value,
  parseValue: (value) => value,
});

// Initialize Query and Mutation types
builder.queryType({});
builder.mutationType({});
"#;

    fs::write(output_dir.join("builder.ts"), content)?;
    Ok(())
}
