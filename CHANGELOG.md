# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.3] - 2026-02-11

### Added

- 64 new unit tests across core modules (config, parser, roblox, utils, migrator, validator)
- Test coverage for config schema validation and loader error handling
- Test coverage for Translation and CloudSync type structures
- Test coverage for plural key detection and edge cases
- Test coverage for flatten/unflatten operations with edge cases
- Test coverage for migration format detection and key transformations
- Test coverage for validator coverage info structures

### Fixed

- Override test now correctly validates CSV output instead of Luau (overrides are uploaded to Roblox Cloud, not embedded in Luau)
- Override test format corrected to match parser expectations (locale-first structure)
- Test count increased from 178 to 242 unit tests in lib (36% increase)

## [1.1.2] - 2026-02-09

### Fixed

- Duplicate country code mapping for Singapore (SG) in generated locale detection code - now correctly maps to English only instead of conflicting English/Chinese mappings
- Type definition generation now produces deterministic, alphabetically sorted output to reduce git diff noise (#6)
- Luau code generation now produces deterministic, alphabetically sorted output to reduce git diff noise (#7)

## [1.1.1] - 2026-02-09

### Fixed

- Rate limit handling now respects `Retry-After` header from Roblox API instead of hardcoding 1 second backoff
- Upload command now performs comprehensive validation (parsing, missing keys, conflicts) instead of only config validation

## [1.1.0] - 2026-02-09

### Added

- Cloud sync feature for bidirectional synchronization with Roblox Cloud Localization Tables
- `upload` command to push local translations to Roblox Cloud
- `download` command to pull translations from Roblox Cloud
- `sync` command with merge strategies (merge/overwrite/skip-conflicts)
- Authentication via environment variable (`ROBLOX_CLOUD_API_KEY`) or config file
- Rate limiting (600 req/min) with exponential backoff
- Retry-After header support for 429 responses
- Table ID discovery from game ID
- Base locale handling from cloud source text
- Cloud configuration section in init template with inline documentation
- Comprehensive unit tests for cloud sync (10 new tests)

### Changed

- Config field renamed: `default_merge_strategy` → `strategy`

## [1.0.0] - 2026-02-08

### Added

- Initial release of Roblox Slang
- Type-safe translation access with autocomplete support
- String interpolation with parameter validation (`{name}`, `{count:int}`)
- Pluralization support using CLDR rules (zero/one/two/few/many/other)
- Nested namespace syntax (`t.ui.buttons.buy()`)
- Watch mode for auto-rebuild on file changes
- CSV generation for Roblox Cloud Localization format
- Multiple input format support (JSON, YAML, CSV)
- Translation overrides for A/B testing and seasonal events
- CLI commands: `init`, `build`, `import`, `validate`, `migrate`
- Comprehensive test suite (168 tests: 88 unit + 30 CLI + 25 edge case + 13 integration + 12 stress)
- Performance benchmarks with criterion
- GitHub Actions CI/CD pipeline with multi-platform builds
- Distribution via Rokit, Aftman, Foreman, and Cargo
- Complete documentation with guides and examples
- Support for 17 Roblox locales
- Type definitions generator (`.d.luau`) for LSP autocomplete
- Rojo integration support
- Module-level documentation for all public modules
- Comprehensive error messages with file paths, line numbers, and helpful hints
- Input validation for locale codes, translation keys, and config files
- Security audit with cargo-audit

### Changed

- Optimized binary size to 1.5 MB
- Optimized build performance to <10ms
- Optimized watch mode latency to <10ms
- Removed 8 unused dependencies (tokio, toml, jsonc-parser, reqwest, thiserror, handlebars, pathdiff, prettytable-rs)
- Updated release profile for maximum optimization (opt-level="z", lto=true, strip=true)
- Improved error messages with context snippets and suggestions
- Fixed all compiler and clippy warnings
- Regenerated Cargo.lock with lockfile version 3
- Updated all repository references from protheeuz to mathtechstudio organization
- Removed MSRV constraint to allow using latest Rust stable features

### Fixed

- Parser errors now include file path, line number, and context snippet
- Config validation errors now include helpful hints and examples
- Empty files handled gracefully with warnings
- Missing files show clear error messages with suggestions
- Permission errors explained clearly
- Invalid UTF-8 detected and reported with file path
- URL warnings in documentation fixed
- Doctest example in library documentation now uses correct API functions
- Watch mode test now handles both interrupt and exit code 1 for Windows compatibility
