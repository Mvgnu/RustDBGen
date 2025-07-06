use anyhow::Result;
use chrono::Utc;
use clap::{Parser, Subcommand};
use rustdbgen::ir::{SchemaIR, TypeAlias};
use rustdbgen::{
    apply_macros, apply_migrations, apply_model_options, apply_seed_data, apply_type_aliases,
    generate_code_multi, generate_graphql_schema, generate_migration, generate_seed_sql,
    generate_ts_client, generate_typescript, introspect_schema, lint_schema, load_schema,
    pull_schema, push_schema, run_plugin,
};
use std::fs;
use which::which;

// load type aliases from type_map.toml if present
fn load_type_aliases() -> std::collections::HashMap<String, TypeAlias> {
    fs::read_to_string("type_map.toml")
        .ok()
        .and_then(|t| toml::from_str::<toml::Value>(&t).ok())
        .and_then(|v| v.as_table().cloned())
        .map(|tbl| {
            tbl.into_iter()
                .filter_map(|(k, val)| {
                    if k == "db_types" {
                        return None;
                    }
                    toml::Value::try_into(val).ok().map(|alias| (k, alias))
                })
                .collect()
        })
        .unwrap_or_default()
}

// load database type mapping from type_map.toml if present
fn load_db_type_map() -> std::collections::HashMap<String, String> {
    fs::read_to_string("type_map.toml")
        .ok()
        .and_then(|t| toml::from_str::<toml::Value>(&t).ok())
        .and_then(|v| v.get("db_types").cloned())
        .and_then(|v| v.try_into().ok())
        .unwrap_or_default()
}

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate Rust code from the schema
    Generate {
        /// Path to write the generated code. Prints to stdout if not set.
        #[arg(long)]
        out: Option<String>,
    },
    /// Generate TypeScript types and route constants from the schema
    GenerateTs {
        /// Path to write the generated TypeScript. Prints to stdout if not set.
        #[arg(long)]
        out: Option<String>,
    },
    /// Generate a TypeScript fetch client for all routes
    GenerateClient {
        /// Path to write the generated client. Prints to stdout if not set.
        #[arg(long)]
        out: Option<String>,
    },
    /// Generate GraphQL schema from the models and relations
    GenerateGraphql {
        /// Path to write the generated GraphQL schema. Prints to stdout if not set.
        #[arg(long)]
        out: Option<String>,
    },
    /// Introspect an existing database and print a schema snapshot
    Introspect {
        /// Output path for the introspected schema TOML
        #[arg(long)]
        out: Option<String>,
        /// Database URL; defaults to the DATABASE_URL environment variable
        #[arg(long)]
        url: Option<String>,
    },
    /// Generate a SQL migration
    Migrate {
        #[command(subcommand)]
        command: MigrateCommands,
    },
    /// Lint the schema for common errors
    Lint,
    /// Run an external plugin with the schema JSON on stdin.
    ///
    /// WARNING: plugins are arbitrary executables. Only run code you trust.
    Plugin {
        /// Path to the plugin executable
        #[arg(long, conflicts_with = "name")]
        exe: Option<String>,
        /// Name of the plugin defined in the schema
        #[arg(long, conflicts_with = "exe")]
        name: Option<String>,
        /// Write plugin stdout to this path instead of printing
        #[arg(long)]
        out: Option<String>,
        /// Working directory for the plugin
        #[arg(long)]
        cwd: Option<String>,
        /// Additional arguments to pass to the plugin
        #[arg(long, value_name = "ARG")]
        arg: Vec<String>,
    },
    /// Generate SQL inserts from seed data
    Seed {
        /// Output path for the seed SQL
        #[arg(long)]
        out: Option<String>,
        /// Database URL to apply the seed data; if omitted only SQL is generated
        #[arg(long)]
        url: Option<String>,
    },
    /// Push or pull the schema to/from a registry path
    Registry {
        #[command(subcommand)]
        command: RegistryCommands,
    },
    /// Serve a simple web editor for the schema
    Serve {
        /// Address to bind the HTTP server
        #[arg(long, default_value = "127.0.0.1:8080")]
        addr: String,
        /// Schema file to edit
        #[arg(long, default_value = "schema.model.toml")]
        schema: String,
    },
}

#[derive(Subcommand)]
enum MigrateCommands {
    /// Generate migration files based on the current schema
    Generate {
        /// A descriptive name for the migration
        name: String,
        /// Optionally introspect this database to use as the previous schema
        #[arg(long)]
        url: Option<String>,
    },
    /// Check if a migration is needed without creating files
    Check {
        /// Optionally introspect this database to use as the previous schema
        #[arg(long)]
        url: Option<String>,
    },
    /// Apply pending migrations to the target database
    Apply {
        /// Database URL; defaults to the DATABASE_URL environment variable
        #[arg(long)]
        url: Option<String>,
    },
    /// Create a template data migration SQL file
    GenerateData {
        /// Descriptive name for the data migration
        name: String,
    },
}

#[derive(Subcommand)]
enum RegistryCommands {
    /// Copy local schema to registry path
    Push {
        /// Registry path to store the schema
        #[arg(long)]
        path: Option<String>,
    },
    /// Fetch schema from registry path
    Pull {
        /// Registry path to read the schema from
        #[arg(long)]
        path: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate { out } => {
            let mut ir: SchemaIR = load_schema("schema.model.toml")?;
            apply_macros(&mut ir);
            let aliases = load_type_aliases();
            apply_type_aliases(&mut ir, &aliases);
            apply_model_options(&mut ir);
            
            let path = out.unwrap_or_else(|| "generated".to_string());
            generate_code_multi(&ir, std::path::Path::new(&path))?;
        }
        Commands::GenerateTs { out } => {
            let mut ir: SchemaIR = load_schema("schema.model.toml")?;
            apply_macros(&mut ir);
            let aliases = load_type_aliases();
            apply_type_aliases(&mut ir, &aliases);
            apply_model_options(&mut ir);
            let ts = generate_typescript(&ir);
            if let Some(path) = out {
                fs::write(path, ts)?;
            } else {
                println!("{}", ts);
            }
        }
        Commands::GenerateClient { out } => {
            let mut ir: SchemaIR = load_schema("schema.model.toml")?;
            apply_macros(&mut ir);
            let aliases = load_type_aliases();
            apply_type_aliases(&mut ir, &aliases);
            apply_model_options(&mut ir);
            let client = generate_ts_client(&ir);
            if let Some(path) = out {
                fs::write(path, client)?;
            } else {
                println!("{}", client);
            }
        }
        Commands::GenerateGraphql { out } => {
            let mut ir: SchemaIR = load_schema("schema.model.toml")?;
            apply_macros(&mut ir);
            let aliases = load_type_aliases();
            apply_type_aliases(&mut ir, &aliases);
            apply_model_options(&mut ir);
            let gql = generate_graphql_schema(&ir);
            if let Some(path) = out {
                fs::write(path, gql)?;
            } else {
                println!("{}", gql);
            }
        }
        Commands::Introspect { out, url } => {
            let url = url
                .or_else(|| std::env::var("DATABASE_URL").ok())
                .ok_or_else(|| anyhow::anyhow!("DATABASE_URL not specified"))?;
            let db_map = load_db_type_map();
            let ir = introspect_schema(&url, &db_map).await?;
            let toml = toml::to_string_pretty(&ir)?;
            if let Some(path) = out {
                fs::write(path, toml)?;
            } else {
                println!("{}", toml);
            }
        }
        Commands::Migrate {
            command: MigrateCommands::Generate { name, url },
        } => {
            let mut ir: SchemaIR = load_schema("schema.model.toml")?;
            apply_macros(&mut ir);
            let aliases = load_type_aliases();
            apply_type_aliases(&mut ir, &aliases);
            apply_model_options(&mut ir);
            let prev_ir: Option<SchemaIR> =
                if let Some(url) = url.or_else(|| std::env::var("DATABASE_URL").ok()) {
                    let db_map = load_db_type_map();
                    Some(introspect_schema(&url, &db_map).await?)
                } else {
                    fs::read_to_string("migrations/schema.json")
                        .ok()
                        .and_then(|t| serde_json::from_str(&t).ok())
                };
            let (up_sql, down_sql) = generate_migration(prev_ir.as_ref(), &ir)?;
            fs::create_dir_all("migrations")?;
            let ts = Utc::now().format("%Y%m%d%H%M%S");
            let up_path = format!("migrations/{}_{}.up.sql", ts, name);
            let down_path = format!("migrations/{}_{}.down.sql", ts, name);
            fs::write(&up_path, up_sql)?;
            fs::write(&down_path, down_sql)?;
            fs::write("migrations/schema.json", serde_json::to_string_pretty(&ir)?)?;
            println!("Created migration: {}", up_path);
        }
        Commands::Migrate {
            command: MigrateCommands::GenerateData { name },
        } => {
            let path = rustdbgen::create_data_migration(&name)?;
            println!("Created data migration: {}", path);
        }
        Commands::Migrate {
            command: MigrateCommands::Check { url },
        } => {
            let mut ir: SchemaIR = load_schema("schema.model.toml")?;
            apply_macros(&mut ir);
            let aliases = load_type_aliases();
            apply_type_aliases(&mut ir, &aliases);
            apply_model_options(&mut ir);
            let prev_ir: Option<SchemaIR> =
                if let Some(url) = url.or_else(|| std::env::var("DATABASE_URL").ok()) {
                    let db_map = load_db_type_map();
                    Some(introspect_schema(&url, &db_map).await?)
                } else {
                    fs::read_to_string("migrations/schema.json")
                        .ok()
                        .and_then(|t| serde_json::from_str(&t).ok())
                };
            let (up_sql, _down_sql) = generate_migration(prev_ir.as_ref(), &ir)?;
            if up_sql.trim().is_empty() {
                println!("Schema is up to date");
            } else {
                anyhow::bail!("Pending migration detected");
            }
        }
        Commands::Migrate {
            command: MigrateCommands::Apply { url },
        } => {
            let url = url
                .or_else(|| std::env::var("DATABASE_URL").ok())
                .ok_or_else(|| anyhow::anyhow!("DATABASE_URL not specified"))?;
            let pool = rustdbgen::connect_any_pool(&url).await?;
            apply_migrations(&pool, "migrations").await?;
            println!("Migrations applied");
        }
        Commands::Lint => {
            let mut ir: SchemaIR = load_schema("schema.model.toml")?;
            apply_macros(&mut ir);
            let aliases = load_type_aliases();
            apply_type_aliases(&mut ir, &aliases);
            apply_model_options(&mut ir);
            let errors = lint_schema(&ir);
            if errors.is_empty() {
                println!("Schema lint passed");
            } else {
                for e in &errors {
                    eprintln!("{}", e);
                }
                anyhow::bail!("Schema lint failed");
            }
        }
        Commands::Plugin {
            exe,
            name,
            out,
            cwd,
            arg,
        } => {
            let mut ir: SchemaIR = load_schema("schema.model.toml")?;
            apply_macros(&mut ir);
            let aliases = load_type_aliases();
            apply_type_aliases(&mut ir, &aliases);
            apply_model_options(&mut ir);
            let (exe_path, mut args, env_map, schema_cwd) = match (exe, name) {
                (Some(p), None) => (p, Vec::new(), std::collections::HashMap::new(), None),
                (None, Some(n)) => {
                    if let Some(def) = ir.plugins.get(&n) {
                        (
                            def.command.clone(),
                            def.args.clone(),
                            def.env.clone(),
                            def.cwd.clone(),
                        )
                    } else if let Ok(path) = which(&n) {
                        (
                            path.to_string_lossy().to_string(),
                            Vec::new(),
                            std::collections::HashMap::new(),
                            None,
                        )
                    } else {
                        anyhow::bail!(format!("Unknown plugin {}", n));
                    }
                }
                _ => anyhow::bail!("Specify --exe or --name"),
            };
            args.extend(arg);
            let cwd_final = cwd.or(schema_cwd);
            eprintln!(
                "Warning: executing plugin '{}'. Ensure you trust this executable.",
                exe_path
            );
            let output = run_plugin(&exe_path, &args, &env_map, &cwd_final, &ir)?;
            if let Some(path) = out {
                fs::write(path, output)?;
            } else {
                println!("{}", output);
            }
        }
        Commands::Seed { out, url } => {
            let mut ir: SchemaIR = load_schema("schema.model.toml")?;
            apply_macros(&mut ir);
            let aliases = load_type_aliases();
            apply_type_aliases(&mut ir, &aliases);
            apply_model_options(&mut ir);
            let sql = generate_seed_sql(&ir);
            if let Some(url) = url.or_else(|| std::env::var("DATABASE_URL").ok()) {
                let pool = rustdbgen::connect_any_pool(&url).await?;
                apply_seed_data(&pool, &ir).await?;
            }
            if let Some(path) = out {
                fs::write(path, sql)?;
            } else {
                println!("{}", sql);
            }
        }
        Commands::Registry { command } => {
            let path = |p: Option<String>| {
                p.or_else(|| std::env::var("SCHEMA_REGISTRY_PATH").ok())
                    .ok_or_else(|| anyhow::anyhow!("registry path not specified"))
            };
            match command {
                RegistryCommands::Push { path: dest } => {
                    let dest = path(dest)?;
                    push_schema("schema.model.toml", &dest)?;
                    println!("Pushed schema to {}", dest);
                }
                RegistryCommands::Pull { path: src } => {
                    let src = path(src)?;
                    pull_schema(&src, "schema.model.toml")?;
                    println!("Pulled schema from {}", src);
                }
            }
        }
        Commands::Serve { addr, schema } => {
            rustdbgen::serve_editor(&addr, &schema).await?;
        }
    }

    Ok(())
}
