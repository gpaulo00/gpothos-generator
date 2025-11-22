use anyhow::Result;
use std::fs;
use std::path::Path;

/// Generate all base filter input types for Pothos
pub fn generate_filters(output_dir: &Path) -> Result<()> {
    let inputs_dir = output_dir.join("inputs");
    fs::create_dir_all(&inputs_dir)?;

    let content = r#"import { builder } from "../builder";

// String Filter
export const StringFilter = builder.inputType("StringFilter", {
  fields: (t) => ({
    equals: t.string(),
    in: t.stringList(),
    notIn: t.stringList(),
    lt: t.string(),
    lte: t.string(),
    gt: t.string(),
    gte: t.string(),
    contains: t.string(),
    startsWith: t.string(),
    endsWith: t.string(),
    mode: t.string(),
    not: t.string(),
  }),
});

// Int Filter
export const IntFilter = builder.inputType("IntFilter", {
  fields: (t) => ({
    equals: t.int(),
    in: t.intList(),
    notIn: t.intList(),
    lt: t.int(),
    lte: t.int(),
    gt: t.int(),
    gte: t.int(),
    not: t.int(),
  }),
});

// Float Filter
export const FloatFilter = builder.inputType("FloatFilter", {
  fields: (t) => ({
    equals: t.float(),
    in: t.field({ type: ["Float"] }),
    notIn: t.field({ type: ["Float"] }),
    lt: t.float(),
    lte: t.float(),
    gt: t.float(),
    gte: t.float(),
    not: t.float(),
  }),
});

// Bool Filter
export const BoolFilter = builder.inputType("BoolFilter", {
  fields: (t) => ({
    equals: t.boolean(),
    not: t.boolean(),
  }),
});

// DateTime Filter
export const DateTimeFilter = builder.inputType("DateTimeFilter", {
  fields: (t) => ({
    equals: t.field({ type: "DateTime" }),
    in: t.field({ type: ["DateTime"] }),
    notIn: t.field({ type: ["DateTime"] }),
    lt: t.field({ type: "DateTime" }),
    lte: t.field({ type: "DateTime" }),
    gt: t.field({ type: "DateTime" }),
    gte: t.field({ type: "DateTime" }),
    not: t.field({ type: "DateTime" }),
  }),
});
"#;

    fs::write(inputs_dir.join("filters.ts"), content)?;

    Ok(())
}
