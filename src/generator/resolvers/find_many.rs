use crate::parser::Model;
use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn generate(model: &Model, resolver_dir: &Path, _args_dir: &Path) -> Result<()> {
    let model_lower = to_lowercase_first(&model.name);
    let plural = pluralize(&model_lower);

    let content = format!(
        r#"import {{ builder }} from "../builder";
import {{ {model} }} from "../models/{model}";
import {{ {model}WhereInput }} from "../inputs/{model}WhereInput";
import {{ {model}OrderByInput }} from "../inputs/{model}OrderByInput";

builder.queryField("{plural}", (t) =>
  t.prismaField({{
    type: ["{model}"],
    args: {{
      where: t.arg({{ type: {model}WhereInput }}),
      orderBy: t.arg({{ type: [{model}OrderByInput] }}),
      first: t.arg.int(),
      last: t.arg.int(),
    }},
    resolve: async (query, _root, args, ctx) => {{
      return ctx.prisma.{model_lower}.findMany({{
        ...query,
        where: args.where ?? undefined,
        orderBy: args.orderBy ?? undefined,
        take: args.first ?? undefined,
        skip: args.last ?? undefined,
      }});
    }},
  }})
);
"#,
        model = model.name,
        model_lower = model_lower,
        plural = plural
    );

    fs::write(
        resolver_dir.join(format!("findMany{}.ts", model.name)),
        content,
    )?;

    Ok(())
}

fn to_lowercase_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_lowercase().collect::<String>() + chars.as_str(),
    }
}

fn pluralize(s: &str) -> String {
    if s.ends_with('s') || s.ends_with('x') || s.ends_with("ch") || s.ends_with("sh") {
        format!("{}es", s)
    } else if s.ends_with('y') && !s.ends_with("ay") && !s.ends_with("ey") && !s.ends_with("oy") && !s.ends_with("uy") {
        format!("{}ies", &s[..s.len() - 1])
    } else {
        format!("{}s", s)
    }
}
