use anyhow::Result;
use clap::{Parser, Subcommand};
use roblox_slang::{cli, utils::validation};
use std::path::Path;

#[derive(Parser)]
#[command(name = "roblox-slang")]
#[command(version)]
#[command(about = "Type-safe internationalization for Roblox experiences")]
#[command(
    long_about = "Roblox Slang is a CLI tool that generates type-safe Luau code from translation files.\n\
                        Write translations in JSON/YAML, generate type-safe code with autocomplete support.\n\n\
                        For more information, visit: https://github.com/bookek/roblox-slang"
)]
#[command(author = "Iqbal Fauzien <iqbalfauzien@proton.me>")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create the config file and translation directory.
    Init {
        #[arg(long, help = "Create overrides.yaml template")]
        with_overrides: bool,
    },

    /// Build translations into Luau, types, and Roblox Cloud CSV.
    Build {
        #[arg(short, long, help = "Enable watch mode (auto-rebuild on changes)")]
        watch: bool,
    },

    /// Convert a Roblox Cloud CSV file into JSON translation files.
    Import {
        #[arg(value_name = "CSV_FILE", help = "Path to Roblox CSV file")]
        csv_file: String,
    },

    /// Check translations for missing keys, conflicts, unused keys, and coverage.
    Validate {
        #[arg(long, help = "Check for missing translations")]
        missing: bool,

        #[arg(long, help = "Check for unused keys")]
        unused: bool,

        #[arg(long, help = "Check for conflicts")]
        conflicts: bool,

        #[arg(long, help = "Show coverage report")]
        coverage: bool,

        #[arg(long, value_name = "DIR", help = "Source directory to scan")]
        source: Option<String>,

        #[arg(long, help = "Run all checks")]
        all: bool,
    },

    /// Convert custom-json or gettext files to Roblox Slang JSON.
    Migrate {
        #[arg(
            long,
            value_name = "FORMAT",
            help = "Source format (custom-json, gettext)"
        )]
        from: String,

        #[arg(long, value_name = "FILE", help = "Input file path")]
        input: String,

        #[arg(long, value_name = "FILE", help = "Output file path")]
        output: String,

        #[arg(
            long,
            value_name = "TRANSFORM",
            help = "Key transformation (snake-to-camel, upper-to-lower, dot-to-nested, none)"
        )]
        transform: Option<String>,
    },

    /// Upload local translations to Roblox Cloud.
    Upload {
        #[arg(
            long,
            value_name = "TABLE_ID",
            help = "Localization table ID (or use config: cloud.table_id)"
        )]
        table_id: Option<String>,

        #[arg(long, help = "Preview changes without uploading (shows statistics)")]
        dry_run: bool,

        #[arg(long, help = "Skip validation before upload (not recommended)")]
        skip_validation: bool,
    },

    /// Download Roblox Cloud translations into local JSON files.
    Download {
        #[arg(
            long,
            value_name = "TABLE_ID",
            help = "Localization table ID (or use config: cloud.table_id)"
        )]
        table_id: Option<String>,

        #[arg(
            long,
            help = "Preview changes without writing files (shows statistics)"
        )]
        dry_run: bool,
    },

    /// Synchronize local and Roblox Cloud translations.
    Sync {
        #[arg(
            long,
            value_name = "TABLE_ID",
            help = "Localization table ID (or use config: cloud.table_id)"
        )]
        table_id: Option<String>,

        #[arg(
            long,
            value_name = "STRATEGY",
            help = "overwrite | merge | skip-conflicts (or use config: cloud.strategy)"
        )]
        strategy: Option<String>,

        #[arg(long, help = "Preview changes without syncing (shows statistics)")]
        dry_run: bool,
    },
}

fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    let runtime = tokio::runtime::Runtime::new()?;

    match cli.command {
        Commands::Init { with_overrides } => {
            cli::init(with_overrides)?;
        }
        Commands::Build { watch } => {
            let config_path = Path::new("slang-roblox.yaml");

            if watch {
                cli::watch(config_path)?;
            } else {
                cli::build(config_path)?;
            }
        }
        Commands::Import { csv_file } => {
            let csv_path = Path::new(&csv_file);

            validation::validate_safe_path(csv_path)?;
            validation::validate_file_exists(csv_path, "CSV file")?;

            let config_path = Path::new("slang-roblox.yaml");
            cli::import_csv(csv_path, config_path)?;
        }
        Commands::Validate {
            missing,
            unused,
            conflicts,
            coverage,
            source,
            all,
        } => {
            let config_path = Path::new("slang-roblox.yaml");

            let check_missing = all || missing;
            let check_unused = all || unused;
            let check_conflicts = all || conflicts;
            let show_coverage = all || coverage;

            let source_dir = if let Some(ref s) = source {
                let path = Path::new(s.as_str());
                validation::validate_safe_path(path)?;
                validation::validate_directory_exists(path, "source directory")?;
                Some(path)
            } else {
                None
            };

            cli::validate(
                config_path,
                check_missing,
                check_unused,
                check_conflicts,
                show_coverage,
                source_dir,
            )?;
        }
        Commands::Migrate {
            from,
            input,
            output,
            transform,
        } => {
            let input_path = Path::new(&input);
            let output_path = Path::new(&output);

            validation::validate_safe_path(input_path)?;
            validation::validate_file_exists(input_path, "input file")?;
            validation::validate_safe_path(output_path)?;

            let transform_str = transform.as_deref();

            cli::migrate(&from, input_path, output_path, transform_str)?;
        }
        Commands::Upload {
            table_id,
            dry_run,
            skip_validation,
        } => {
            runtime.block_on(cli::upload(table_id, dry_run, skip_validation))?;
        }
        Commands::Download { table_id, dry_run } => {
            runtime.block_on(cli::download(table_id, dry_run))?;
        }
        Commands::Sync {
            table_id,
            strategy,
            dry_run,
        } => {
            runtime.block_on(cli::sync(table_id, strategy, dry_run))?;
        }
    }

    Ok(())
}
