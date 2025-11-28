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
import {{ {model}CreateInput }} from "../inputs/{model}CreateInput";

builder.mutationField("{mutation_name}", (t) =>
  t.prismaField({{
    nullable: false,
    type: "{model}",
    args: {{
      data: t.arg({{ type: {model}CreateInput, required: true }}),
    }},
    resolve: async (query, _root, args, ctx) => {{
      return ctx.prisma.{prisma_model}.create({{
        ...query,
        data: args.data,
      }});
    }},
  }})
);
"#,
        model = model.name,
        prisma_model = names.query_new2,  // Use query_new2 for Prisma client calls
        mutation_name = names.create
    );

    fs::write(
        resolver_dir.join(format!("createOne{}.ts", model.name)),
        content,
    )?;

    Ok(())
}
