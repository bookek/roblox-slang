//! # Roblox Slang
//!
//! Type-safe Internationalization (i18n) tooling for Roblox experiences.
//!
//! The crate parses JSON/YAML translations, generates typed Luau helpers, writes
//! Roblox Cloud CSV files, and exposes the same pieces used by the CLI.
//!
//! ## Quick Start
//!
//! ```bash
//! roblox-slang init
//! roblox-slang build
//! roblox-slang build --watch
//! ```
//!
//! ## Library Usage
//!
//! ```no_run
//! use roblox_slang::{config, generator, parser};
//! use std::path::Path;
//!
//! # fn main() -> anyhow::Result<()> {
//! let cfg = config::load_config(Path::new("slang-roblox.yaml"))?;
//! let translations = parser::json::parse_json_file(
//!     Path::new("translations/en.json"),
//!     "en"
//! )?;
//! let luau_code = generator::luau::generate_luau(&translations, &cfg.base_locale)?;
//! # Ok(())
//! # }
//! ```

pub mod cli;
pub mod config;
pub mod generator;
pub mod migrator;
pub mod parser;
pub mod roblox;
pub mod utils;
pub mod validator;

pub use config::Config;
pub use parser::types::Translation;
