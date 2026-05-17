# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [3.0.0] - 2026-05-18

Do not hate **monday**, **monday** hates you too. Despite a busy schedule, significant effort went into this major release to deliver critical improvements in namespace binding, identifier safety, and synchronization reliability.

### Added

- Added `Translations.detectLocale(player)` and `Translations.newForPlayer(player)` to the default `embedded` localization mode.
- Added build-time collision checks to prevent conflicting translation keys that sanitize to the same identifier.

### Changed

- **BREAKING**: Replaced class-level namespaces with per-instance closures bound inside `_setupNamespaces` at instantiation. Namespace API methods now support both colon (`t.ui.buttons:buy()`) and dot (`t.ui.buttons.buy()`) syntax correctly.
- Improved the `sync` command to safely merge cloud downloads with existing local translations, resolving the risk of deleting local-only keys.
- Sanitized translation keys to produce valid Luau identifiers in generated code and `.d.luau` types.

### Security

- Updated `rustls-webpki` and `rand` dependencies to patch security vulnerabilities.

## [2.0.4] - 2026-04-04

### Changed

- The Luau code generator used to guess user languages based on their IP address country code. That felt wrong when a player explicitly chose a different language in their Roblox account settings. We changed the generator to read `player.LocaleId` instead.
- We completely removed the old country mapping utility (`get_country_locale_map`) since the generator no longer uses it, which keeps the codebase clean.

### Security

- Upgraded transitive dependencies `quinn-proto` (to `v0.11.14`) and `rustls-webpki` (to `v0.103.10`) to patch DoS and CRL validation vulnerabilities.

## [2.0.3] - 2026-02-25

### Changed

- Reduced sizes of generated translation files in `hybrid` mode (up to 44% reduction) by extracting the duplicate fallback logic block into a shared `_resolve` helper method.

### Fixed

- Fixed an issue where the `_resolve` fallback logic and the tracking analytics logic would be hardcoded to `"en"` rather than appropriately using the project's configured `base_locale`.

## [2.0.2] - 2026-02-19

### Fixed

- Namespace methods now use method syntax (`:`) instead of static function syntax requiring explicit self parameter
- Generated namespace functions like `Translations.ui.buttons:buy()` now work correctly without passing self
- Loop variable renamed from `key` to `paramKey` in parameter interpolation to avoid variable shadow warnings
- All namespace methods (regular and plural) now consistently use method syntax for better DX

### Changed

- Documentation updated to reflect correct method syntax usage with colon (`:`) for namespace methods
- All examples and guides now show proper method call syntax

## [2.0.1] - 2026-02-18

### Fixed

- Cloud upload now uses batching (20 entries per request) to avoid Roblox API limit causing Error 500
- Large translation sets (90+ keys across multiple locales) now upload successfully in multiple batches
- Upload duration improved with parallel batch processing

## [2.0.0] - 2026-02-18

### Added

- Three localization modes: embedded (default), cloud, hybrid
- Embedded mode with zero runtime dependencies (no LocalizationService required)
- Hybrid mode with automatic cloud-to-embedded fallback
- `EMBEDDED_TRANSLATIONS` table generated at build time for embedded and hybrid modes
- Localization configuration section in config schema with mode selection
- Mode-specific constructor generation (embedded/cloud/hybrid)
- Mode-specific translation method generation for all three modes
- Mode-specific plural method generation for all three modes
- Comprehensive documentation for all localization modes
- Configuration guide with mode comparison table
- Getting started guide with mode selection instructions
- Cloud mode usage guide with setup instructions
- 395 additional unit tests (681 total, up from 286)
- Integration tests for all three localization modes
- Test projects for cloud-mode and hybrid-mode validation

### Changed

- **BREAKING**: Default mode is now `embedded` (was implicitly cloud-only)
- **BREAKING**: Config field `localization.mode` now required in init template
- Constructor behavior varies by mode (embedded has no LocalizationService dependency)
- Translation methods now dispatch to mode-specific implementations
- Plural methods now use correct data source per mode (embedded table vs cloud API)
- Init template now includes localization mode configuration with inline documentation
- Build command passes localization config to generator
- Analytics config handling simplified (removed redundant if-else)
- Match statement fallback arms replaced with `unreachable!()` for fail-fast behavior

### Fixed

- Critical bug where plural methods always used `FormatByKey()` causing crashes in embedded mode
- Plural translations now work correctly in all three modes
- Embedded mode plural methods use `EMBEDDED_TRANSLATIONS` table instead of translator
- Hybrid mode plural methods try cloud first, fall back to embedded data
- Cloud mode error messages now provide helpful hints about uploading translations

### Performance

- Embedded mode has zero LocalizationService overhead
- Embedded mode uses direct table lookup (faster than API calls)
- Hybrid mode reduces cloud API calls by caching in embedded table

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
