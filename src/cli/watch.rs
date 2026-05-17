use anyhow::{Context, Result};
use colored::Colorize;
use notify_debouncer_full::{new_debouncer, notify::*, DebounceEventResult};
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;

use crate::cli;
pub fn watch(config_path: &Path) -> Result<()> {
    println!("{} Starting watch mode...", "→".blue());
    println!("Watching for changes in translation files...");
    println!("Press Ctrl+C to stop\n");
    cli::build(config_path)?;
    let (tx, rx) = channel();

    let mut debouncer = new_debouncer(
        Duration::from_millis(500),
        None,
        move |result: DebounceEventResult| match result {
            Ok(events) => {
                for event in events {
                    if let Err(e) = tx.send(event) {
                        log::error!("Failed to send event: {}", e);
                    }
                }
            }
            Err(errors) => {
                for error in errors {
                    log::error!("Watch error: {:?}", error);
                }
            }
        },
    )
    .context("Failed to create file watcher")?;
    let translations_dir = Path::new("translations");
    if translations_dir.exists() {
        debouncer
            .watcher()
            .watch(translations_dir, RecursiveMode::Recursive)
            .context("Failed to watch translations directory")?;

        println!("{} Watching: {}", "✓".green(), translations_dir.display());
    }
    if config_path.exists() {
        debouncer
            .watcher()
            .watch(config_path, RecursiveMode::NonRecursive)
            .context("Failed to watch config file")?;

        println!("{} Watching: {}", "✓".green(), config_path.display());
    }

    println!();
    loop {
        match rx.recv() {
            Ok(event) => {
                let should_rebuild = event.paths.iter().any(|path| {
                    path.extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| ext == "json" || ext == "yaml" || ext == "yml")
                        .unwrap_or(false)
                });

                if should_rebuild {
                    println!("\n{} File changed, rebuilding...", "→".blue());

                    match cli::build(config_path) {
                        Ok(_) => {
                            println!("\n{} Watching for changes...\n", "✓".green());
                        }
                        Err(e) => {
                            eprintln!("\n{} Build failed: {}\n", "✗".red(), e);
                            eprintln!("{} Watching for changes...\n", "→".yellow());
                        }
                    }
                }
            }
            Err(e) => {
                log::error!("Watch error: {}", e);
                break;
            }
        }
    }

    Ok(())
}
