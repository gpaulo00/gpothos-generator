use anyhow::Result;
use std::fs;
use std::path::Path;

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
  PrismaTypes: PrismaTypes;
  Context: Context;
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
