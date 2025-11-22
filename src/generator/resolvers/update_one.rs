use crate::parser::Model;
use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn generate(model: &Model, resolver_dir: &Path, _args_dir: &Path) -> Result<()> {
    let model_lower = to_lowercase_first(&model.name);

    let content = format!(
        r#"import {{ builder }} from "../builder";
import {{ {model} }} from "../models/{model}";
import {{ {model}UpdateInput }} from "../inputs/{model}UpdateInput";
import {{ {model}WhereUniqueInput }} from "../inputs/{model}WhereUniqueInput";

builder.mutationField("updateOne{model}", (t) =>
  t.prismaField({{
    type: "{model}",
    nullable: true,
    args: {{
      where: t.arg({{ type: {model}WhereUniqueInput, required: true }}),
      data: t.arg({{ type: {model}UpdateInput, required: true }}),
    }},
    resolve: async (query, _root, args, ctx) => {{
      return ctx.prisma.{model_lower}.update({{
        ...query,
        where: args.where,
        data: args.data,
      }});
    }},
  }})
);
"#,
        model = model.name,
        model_lower = model_lower
    );

    fs::write(
        resolver_dir.join(format!("updateOne{}.ts", model.name)),
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
