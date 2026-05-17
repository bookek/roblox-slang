use crate::config;
use crate::roblox::{AuthConfig, MergeStrategy, RobloxCloudClient, SyncOrchestrator};
use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::path::Path;
pub async fn sync(table_id: Option<String>, strategy: Option<String>, dry_run: bool) -> Result<()> {
    let config_path = Path::new("slang-roblox.yaml");
    let config = config::load_config(config_path).context("Failed to load configuration")?;
    let table_id = table_id
        .or_else(|| config.cloud.as_ref().and_then(|c| c.table_id.clone()))
        .context("Table ID not provided. Specify via --table-id or set cloud.table_id in config")?;
    let strategy_str = strategy
        .or_else(|| config.cloud.as_ref().and_then(|c| c.strategy.clone()))
        .unwrap_or_else(|| "merge".to_string());

    let merge_strategy = match strategy_str.to_lowercase().as_str() {
        "overwrite" => MergeStrategy::Overwrite,
        "merge" => MergeStrategy::Merge,
        "skip-conflicts" | "skip_conflicts" => MergeStrategy::SkipConflicts,
        _ => {
            bail!(
                "Invalid merge strategy: '{}'\n\
                 \n\
                 Valid strategies:\n\
                 - overwrite: Replace all cloud translations with local\n\
                 - merge: Merge local and cloud, prefer cloud for conflicts\n\
                 - skip-conflicts: Only sync non-conflicting entries",
                strategy_str
            );
        }
    };
    let auth = AuthConfig::load(&config).context("Failed to load authentication")?;
    let client =
        RobloxCloudClient::new(auth.api_key).context("Failed to create Roblox Cloud client")?;
    let orchestrator = SyncOrchestrator::new(client, config.clone());
    if dry_run {
        println!("{} Dry-run mode: No changes will be made", "ℹ".cyan());
    }

    println!(
        "{} Synchronizing translations (strategy: {})...",
        "→".blue(),
        strategy_str
    );
    println!("  Table ID: {}", table_id);
    println!("  Merge strategy: {}", strategy_str);

    let stats = orchestrator
        .sync(&table_id, merge_strategy, dry_run)
        .await
        .context("Sync failed")?;
    println!("\n{} Sync complete!", "✓".green().bold());
    println!("  Entries added: {}", stats.entries_added);
    println!("  Entries updated: {}", stats.entries_updated);
    println!("  Entries deleted: {}", stats.entries_deleted);

    if stats.conflicts_skipped > 0 {
        println!(
            "  {} Conflicts skipped: {}",
            "⚠".yellow(),
            stats.conflicts_skipped
        );

        let conflicts_path = Path::new(&config.output_directory).join("conflicts.yaml");
        println!(
            "\n{} Conflicts saved to: {}",
            "ℹ".cyan(),
            conflicts_path.display()
        );
        println!("  Review and resolve conflicts manually");
    }

    println!("  Duration: {:.2}s", stats.duration.as_secs_f64());

    if dry_run {
        println!("\n{} This was a dry-run. No changes were made.", "ℹ".cyan());
    } else {
        println!("\n{} Translations successfully synchronized", "✓".green());
    }

    Ok(())
}
