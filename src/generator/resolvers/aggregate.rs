use crate::parser::Model;
use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn generate(model: &Model, resolver_dir: &Path, _args_dir: &Path) -> Result<()> {
    let model_lower = to_lowercase_first(&model.name);

    let content = format!(
        r#"import {{ builder }} from "../builder";
import {{ {model}WhereInput }} from "../inputs/{model}WhereInput";

// Define aggregate result type
const {model}AggregateResult = builder.simpleObject("{model}AggregateResult", {{
  fields: (t) => ({{
    _count: t.int({{}}),
  }}),
}});

builder.queryField("aggregate{model}", (t) =>
  t.field({{
    type: {model}AggregateResult,
    nullable: false,
    args: {{
      where: t.arg({{ type: {model}WhereInput }}),
    }},
    resolve: async (_root, args, ctx) => {{
      const result = await ctx.prisma.{model_lower}.aggregate({{
        where: args.where ?? undefined,
        _count: true,
      }});
      return {{
        _count: result._count,
      }};
    }},
  }})
);
"#,
        model = model.name,
        model_lower = model_lower
    );

    fs::write(
        resolver_dir.join(format!("aggregate{}.ts", model.name)),
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
