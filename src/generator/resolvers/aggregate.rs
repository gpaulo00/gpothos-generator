use crate::parser::Model;
use crate::generator::{get_prisma_name, helpers::capitalize_first};
use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn generate(model: &Model, resolver_dir: &Path, _args_dir: &Path) -> Result<()> {
    let names = get_prisma_name(&model.name);
    
    // Create aggregate query name: "aggregate" + capitalized model name (e.g., "aggregatePlace")
    let capitalized_model = capitalize_first(&model.name);
    let aggregate_name = format!("aggregate{}", capitalized_model);

    let content = format!(
        r#"import {{ builder }} from "../builder";
import {{ {model}WhereInput }} from "../inputs/{model}WhereInput";

// Define aggregate result type
const {model}AggregateResult = builder.simpleObject("Aggregate{model}", {{
  fields: (t) => ({{
    _count: t.int({{}}),
  }}),
}});

builder.queryField("{aggregate_name}", (t) =>
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
        aggregate_name = aggregate_name,
        prisma_model = names.query_new2  // Use query_new2 for Prisma client calls
    );

    fs::write(
        resolver_dir.join(format!("aggregate{}.ts", model.name)),
        content,
    )?;

    Ok(())
}
