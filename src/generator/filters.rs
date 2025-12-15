use anyhow::Result;
use std::fs;
use std::path::Path;

/// Generate all base filter input types for Pothos
pub fn generate_filters(output_dir: &Path) -> Result<()> {
    let inputs_dir = output_dir.join("inputs");
    fs::create_dir_all(&inputs_dir)?;

    let content = r#"import { builder } from "../builder";

// Nested String Filter (for use inside `not`)
export const NestedStringFilter = builder.inputType("NestedStringFilter", {
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
    not: t.field({ type: NestedStringFilter }),
  }),
});

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
    not: t.field({ type: NestedStringFilter }),
  }),
});

// Nested Int Filter (for use inside `not`)
export const NestedIntFilter = builder.inputType("NestedIntFilter", {
  fields: (t) => ({
    equals: t.int(),
    in: t.intList(),
    notIn: t.intList(),
    lt: t.int(),
    lte: t.int(),
    gt: t.int(),
    gte: t.int(),
    not: t.field({ type: NestedIntFilter }),
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
    not: t.field({ type: NestedIntFilter }),
  }),
});

// Nested Float Filter (for use inside `not`)
export const NestedFloatFilter = builder.inputType("NestedFloatFilter", {
  fields: (t) => ({
    equals: t.float(),
    in: t.field({ type: ["Float"] }),
    notIn: t.field({ type: ["Float"] }),
    lt: t.float(),
    lte: t.float(),
    gt: t.float(),
    gte: t.float(),
    not: t.field({ type: NestedFloatFilter }),
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
    not: t.field({ type: NestedFloatFilter }),
  }),
});

// Nested Bool Filter (for use inside `not`)
export const NestedBoolFilter = builder.inputType("NestedBoolFilter", {
  fields: (t) => ({
    equals: t.boolean(),
    not: t.field({ type: NestedBoolFilter }),
  }),
});

// Bool Filter
export const BoolFilter = builder.inputType("BoolFilter", {
  fields: (t) => ({
    equals: t.boolean(),
    not: t.field({ type: NestedBoolFilter }),
  }),
});

// Nested DateTime Filter (for use inside `not`)
export const NestedDateTimeFilter = builder.inputType("NestedDateTimeFilter", {
  fields: (t) => ({
    equals: t.field({ type: "DateTime" }),
    in: t.field({ type: ["DateTime"] }),
    notIn: t.field({ type: ["DateTime"] }),
    lt: t.field({ type: "DateTime" }),
    lte: t.field({ type: "DateTime" }),
    gt: t.field({ type: "DateTime" }),
    gte: t.field({ type: "DateTime" }),
    not: t.field({ type: NestedDateTimeFilter }),
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
    not: t.field({ type: NestedDateTimeFilter }),
  }),
});
"#;

    fs::write(inputs_dir.join("filters.ts"), content)?;

    Ok(())
}
