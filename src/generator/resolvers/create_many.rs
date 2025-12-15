use crate::parser::Model;
use crate::generator::get_prisma_name;
use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn generate(model: &Model, resolver_dir: &Path, _args_dir: &Path) -> Result<()> {
    let names = get_prisma_name(&model.name);

    let content = format!(
        r#"import {{ builder, AffectedRowsOutput }} from "../builder";
import {{ {create_many_input} }} from "../inputs/{create_many_input}";

builder.mutationField("{mutation_name}", (t) =>
  t.field({{
    type: AffectedRowsOutput,
    nullable: false,
    args: {{
      data: t.arg({{ type: [{create_many_input}], required: true }}),
      skipDuplicates: t.arg.boolean(),
    }},
    resolve: async (_root, args, ctx) => {{
      const result = await ctx.prisma.{prisma_model}.createMany({{
        data: args.data,
        skipDuplicates: args.skipDuplicates ?? undefined,
      }});
      return result;
    }},
  }})
);
"#,
        create_many_input = names.create_many_input,
        prisma_model = names.query_new2,
        mutation_name = names.create_many
    );

    fs::write(
        resolver_dir.join(format!("createMany{}.ts", model.name)),
        content,
    )?;

    Ok(())
}
