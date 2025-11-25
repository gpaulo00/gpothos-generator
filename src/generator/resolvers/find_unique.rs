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
import {{ {model}WhereUniqueInput }} from "../inputs/{model}WhereUniqueInput";

builder.queryField("{query_name}", (t) =>
  t.prismaField({{
    type: "{model}",
    nullable: true,
    args: {{
      where: t.arg({{ type: {model}WhereUniqueInput, required: true }}),
    }},
    resolve: async (query, _root, args, ctx) => {{
      return ctx.prisma.{prisma_model}.findUnique({{
        ...query,
        where: args.where,
      }});
    }},
  }})
);
"#,
        model = model.name,
        prisma_model = names.query_new2,  // Use query_new2 for Prisma client calls
        query_name = names.find     // Use find for GraphQL field name (camelCase)
    );

    fs::write(
        resolver_dir.join(format!("findUnique{}.ts", model.name)),
        content,
    )?;

    Ok(())
}
