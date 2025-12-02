use crate::parser::Model;
use crate::generator::get_prisma_name;
use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn generate(model: &Model, resolver_dir: &Path, _args_dir: &Path) -> Result<()> {
    let names = get_prisma_name(&model.name);

    let content = format!(
        r#"import {{ builder }} from "../builder";
import {{ {model} }} from "../models/{model}";
import {{ {model}WhereInput }} from "../inputs/{model}WhereInput";
import {{ {model}OrderByInput }} from "../inputs/{model}OrderByInput";

builder.queryField("{query_name}", (t) =>
  t.prismaField({{
    nullable: false,
    type: ["{model}"],
    args: {{
      where: t.arg({{ type: {model}WhereInput }}),
      orderBy: t.arg({{ type: [{model}OrderByInput] }}),
      first: t.arg.int(),
      last: t.arg.int(),
    }},
    resolve: async (query, _root, args, ctx) => {{
      return ctx.prisma.{prisma_model}.findMany({{
        ...query,
        where: args.where ?? undefined,
        orderBy: (args.orderBy ?? undefined) as any,
        take: args.first ?? undefined,
        skip: args.last ?? undefined,
      }});
    }},
  }})
);
"#,
        model = model.name,
        prisma_model = names.query_new2,  // Use query_new2 for Prisma client calls
        query_name = names.find_many     // Use find_many for GraphQL field name (camelCase + plural)
    );

    fs::write(
        resolver_dir.join(format!("findMany{}.ts", model.name)),
        content,
    )?;

    Ok(())
}
