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
import {{ {model}UpdateInput }} from "../inputs/{model}UpdateInput";
import {{ {model}WhereUniqueInput }} from "../inputs/{model}WhereUniqueInput";

builder.mutationField("{mutation_name}", (t) =>
  t.prismaField({{
    type: "{model}",
    nullable: true,
    args: {{
      where: t.arg({{ type: {model}WhereUniqueInput, required: true }}),
      data: t.arg({{ type: {model}UpdateInput, required: true }}),
    }},
    resolve: async (query, _root, args, ctx) => {{
      return ctx.prisma.{prisma_model}.update({{
        ...query,
        where: args.where,
        data: args.data,
      }});
    }},
  }})
);
"#,
        model = model.name,
        prisma_model = names.query_new2,  // Use query_new2 for Prisma client calls
        mutation_name = names.update
    );

    fs::write(
        resolver_dir.join(format!("updateOne{}.ts", model.name)),
        content,
    )?;

    Ok(())
}
