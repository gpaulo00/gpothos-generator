use crate::parser::Model;
use crate::generator::get_prisma_name;
use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn generate(model: &Model, resolver_dir: &Path, _args_dir: &Path) -> Result<()> {
    let names = get_prisma_name(&model.name);

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
      const result = await ctx.prisma.{prisma_model}.aggregate({{
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
        prisma_model = names.query_new2  // Use query_new2 for Prisma client calls
    );

    fs::write(
        resolver_dir.join(format!("aggregate{}.ts", model.name)),
        content,
    )?;

    Ok(())
}
