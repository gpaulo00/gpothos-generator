mod parser;
mod generator;
mod config;
mod scanner;

use std::fs;
use std::io::ErrorKind;
use clap::Parser as ClapParser;
use std::path::PathBuf;
use anyhow::Result;

#[derive(ClapParser, Debug)]
#[command(name = "prisma-pothos-generator")]
#[command(about = "Generate Pothos GraphQL code from Prisma schema")]
struct Args {
    /// Path to the Prisma schema file
    #[arg(short, long, default_value = "./prisma/schema.prisma")]
    schema: PathBuf,

    /// Output directory for generated files
    #[arg(short, long, default_value = "./src/generated")]
    output: PathBuf,

    /// Run as Prisma generator (reads DMMF from stdin)
    #[arg(long)]
    prisma_generator: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.prisma_generator {
        // Prisma generator mode: read DMMF from stdin
        generator::run_as_prisma_generator()?;
    } else {
        // CLI mode: parse schema file directly
        println!("Parsing schema: {:?}", args.schema);
        println!("Output directory: {:?}", args.output);

        // Load configuration from .gpothosrc.json
        let config = config::Config::load()?;
        
        // Scan for manual resolvers if enabled
        let manual_resolvers = if config.auto_scan {
            if config.verbose {
                println!("\n📋 Configuration loaded from .gpothosrc.json");
                println!("  - Auto scan: {}", config.auto_scan);
                println!("  - Scan dirs: {:?}", config.scan_dirs);
                println!("  - Verbose: {}\n", config.verbose);
            }
            scanner::scan_for_manual_resolvers(&config.scan_dirs, config.verbose)?
        } else {
            if config.verbose {
                println!("ℹ️  Auto scan disabled, skipping manual resolver detection\n");
            }
            scanner::ManualResolvers::new()
        };

        let schema_content = std::fs::read_to_string(&args.schema)?;
        let parsed = parser::parse_schema(&schema_content)?;

        let ruta_str: &str = args.output.as_path().to_str().expect("¡La ruta no es válida UTF-8!");

        // Intentar eliminar el directorio de forma recursiva
        match fs::remove_dir_all(ruta_str) {
            Ok(_) => println!("Directorio '{}' y su contenido eliminados con éxito.", ruta_str),
            Err(e) => {
                if e.kind() == ErrorKind::NotFound {
                    // Si el error es 'NotFound', simplemente lo ignoramos.
                    println!("Directorio '{}' ya no existía (no se hizo nada).", ruta_str);
                } else {
                    // Para cualquier otro error (permisos, etc.), lo mostramos.
                    eprintln!("Ocurrió un error inesperado al intentar eliminar '{}': {}", ruta_str, e);
                }
            }
        }

        generator::generate(&parsed, &args.output, &manual_resolvers)?;

        println!("Generation complete!");
    }

    Ok(())
}
