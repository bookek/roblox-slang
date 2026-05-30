<div align="center">
    <img src="docs/assets/featured-image.png" alt="Roblox Slang" height="300" />
</div>

<div>&nbsp;</div>

<div align="center">

[![Version](https://img.shields.io/github/v/release/mathtechstudio/roblox-slang?style=for-the-badge&logo=github)](https://github.com/mathtechstudio/roblox-slang/releases)
[![Roblox](https://img.shields.io/badge/Platform-Roblox-00A2FF?style=for-the-badge&logo=roblox&logoColor=white)](https://www.roblox.com)
[![Rust](https://img.shields.io/badge/Built_with-Rust-orange?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org)
[![Generates](https://img.shields.io/badge/Generates-Luau-00A2FF?style=for-the-badge&logo=lua&logoColor=white)](https://luau-lang.org)
[![License](https://img.shields.io/badge/License-MIT-blue?style=for-the-badge)](LICENSE)
[![Tests](https://img.shields.io/badge/Tests-681_passing-success?style=for-the-badge)](tests/)

</div>

<hr />

**Roblox Slang** is a Type-safe internationalization (i18n) code generator for Roblox experiences. Write translations in JSON/YAML, generate type-safe Luau code with autocomplete support.

## Why Roblox Slang?

Roblox's native localization system uses string literals for translation keys, leading to runtime errors from typos and no IDE support. Roblox Slang solves this by generating type-safe Luau code at build time.

**Before (Roblox native):**

```lua
local translator = LocalizationService:GetTranslatorForPlayerAsync(player)
local text = translator:FormatByKey("UI_Buttons_Confirm") -- Typo-prone, no autocomplete

-- Typo = runtime error
local text = translator:FormatByKey("UI_Buttns_Confirm") -- ERROR at runtime!
```

**After (Roblox Slang):**

```lua
local t = Translations.new("en")
local text = t.ui.buttons:confirm() -- Autocomplete with Luau types

-- Typo = Luau type error
local text = t.ui.buttns:confirm() -- Property does not exist
```

## Features

- **Type-safe translation access** - Autocomplete and type checking in your IDE
- **Three localization modes** - Embedded (default), Cloud, or Hybrid for flexibility
- **String interpolation** - `{name}`, `{count:int}` with parameter validation
- **Pluralization** - CLDR rules (zero/one/two/few/many/other)
- **Nested namespaces** - `t.ui.buttons:buy()` with deep paths such as `ui.buttons.primary.buy`
- **Watch mode** - Auto-rebuild on file changes
- **CSV generation** - Export to Roblox Cloud Localization format
- **Cloud sync** - Bidirectional sync with Roblox Cloud Localization Tables
- **Zero runtime dependencies** - Generated code is pure Luau
- **Multiple input formats** - JSON, YAML, or CSV
- **Translation overrides** - A/B testing and seasonal events support

## Installation

Roblox Slang is a CLI tool that generates code. Choose your preferred installation method:

### Via Rokit (Recommended)

[Rokit](https://github.com/rojo-rbx/rokit) is the fastest and most modern toolchain manager for Roblox projects.

```bash
# Add to your project
rokit add mathtechstudio/roblox-slang

# Or install globally
rokit add --global mathtechstudio/roblox-slang
```

**rokit.toml:**

```toml
[tools]
roblox-slang = "mathtechstudio/roblox-slang@3.0.1"
```

### Via Aftman

> Aftman is no longer actively maintained. For new projects, use [Rokit](#via-rokit-recommended) or [Foreman](#via-foreman).

[Aftman](https://github.com/LPGhatguy/aftman) provides exact version dependencies and a trust-based security model.

```bash
# Add to your project
aftman add mathtechstudio/roblox-slang

# Or install globally
aftman add --global mathtechstudio/roblox-slang
```

**aftman.toml:**

```toml
[tools]
roblox-slang = "mathtechstudio/roblox-slang@3.0.1"
```

### Via Foreman

[Foreman](https://github.com/Roblox/foreman) is the original Roblox toolchain manager, battle-tested in production.

**foreman.toml:**

```toml
[tools]
roblox-slang = { github = "mathtechstudio/roblox-slang", version = "3.0.1" }
```

```bash
foreman install
```

### From GitHub Releases (Manual)

Download pre-built binaries for your platform:

- `roblox-slang-3.0.1-linux-x86_64.zip`
- `roblox-slang-3.0.1-linux-aarch64.zip`
- `roblox-slang-3.0.1-windows-x86_64.zip`
- `roblox-slang-3.0.1-windows-aarch64.zip`
- `roblox-slang-3.0.1-macos-x86_64.zip`
- `roblox-slang-3.0.1-macos-aarch64.zip`

Extract and add to your PATH, or use a tool manager for automatic updates.

### From Source (Cargo)

```bash
# Install from crates.io
cargo install roblox-slang

# Or build from source
git clone https://github.com/mathtechstudio/roblox-slang.git
cd roblox-slang
cargo install --locked --path .
```

### Verify Installation

```bash
roblox-slang --version
# Output: roblox-slang 3.0.1
```

## Quick Start

### 1. Initialize Project

```bash
roblox-slang init
```

Creates:

- `slang-roblox.yaml` - Configuration file
- `translations/` - Translation files directory

### 2. Create Translations

`translations/en.json`:

```json
{
  "ui": {
    "buttons": {
      "buy": "Buy",
      "sell": "Sell"
    },
    "messages": {
      "greeting": "Hello, {name}!",
      "items": {
        "zero": "No items",
        "one": "1 item",
        "other": "{count} items"
      }
    }
  }
}
```

`translations/id.json`:

```json
{
  "ui": {
    "buttons": {
      "buy": "Beli",
      "sell": "Jual"
    },
    "messages": {
      "greeting": "Halo, {name}!",
      "items": {
        "zero": "Tidak ada item",
        "one": "1 item",
        "other": "{count} item"
      }
    }
  }
}
```

### 3. Build

```bash
# One-time build
roblox-slang build

# Watch mode (auto-rebuild on changes)
roblox-slang build --watch
```

Generates:

- `output/Translations.lua` - Main module
- `output/types/Translations.d.luau` - Type definitions for LSP
- `output/roblox_upload.csv` - Roblox Cloud format

### 4. Add to Your Game

#### Option A: Manual (Roblox Studio)

1. Copy `output/Translations.lua` to `ReplicatedStorage` in Roblox Studio
2. Rename to `Translations` (ModuleScript)
3. Optionally copy `output/types/Translations.d.luau` for LSP autocomplete

#### Option B: Rojo (Automatic Sync)

Set output directory to match your Rojo project structure:

**slang-roblox.yaml:**

```yaml
output_directory: src/ReplicatedStorage/Translations
```

**default.project.json:**

```json
{
  "name": "MyGame",
  "tree": {
    "$className": "DataModel",
    "ReplicatedStorage": {
      "$className": "ReplicatedStorage",
      "$path": "src/ReplicatedStorage"
    }
  }
}
```

Run `roblox-slang build` and Rojo will automatically sync to Studio!

### 5. Use in Game

```lua
local Translations = require(ReplicatedStorage.Translations)

-- Create instance for locale
local t = Translations.new("en")

-- Simple translations
print(t.ui.buttons:buy())  -- "Buy"

-- With parameters
print(t.ui.messages:greeting({ name = "Player123" }))  -- "Hello, Player123!"

-- Pluralization
print(t.ui.messages:items(0))  -- "No items"
print(t.ui.messages:items(1))  -- "1 item"
print(t.ui.messages:items(5))  -- "5 items"

-- Switch locale at runtime
t:setLocale("id")
print(t.ui.buttons:buy())  -- "Beli"

-- Auto-detect player locale
local function onPlayerAdded(player)
    local t = Translations.newForPlayer(player)
    print(t.ui.messages:greeting({ name = player.DisplayName }))
end
```

## Configuration

`slang-roblox.yaml`:

```yaml
# Base locale (fallback when translation missing)
base_locale: en

# Supported locales
supported_locales:
  - en
  - id

# Input directory for translation files
input_directory: translations

# Output directory for generated code
# For Rojo users: Set to your Rojo-tracked folder (e.g., src/ReplicatedStorage/Translations)
# For manual users: Keep as "output" and copy to Studio manually
output_directory: output

# Optional: Namespace for generated module (null = no namespace)
namespace: null

# Runtime localization mode
# Controls how translations are loaded at runtime
localization:
  # Mode: embedded (default) - Translations embedded in generated code, no cloud dependency
  # Other options:
  #   - cloud: Use Roblox Cloud LocalizationService only (requires cloud upload)
  #   - hybrid: Try cloud first, then use embedded translations on error
  mode: embedded

# Optional: Translation overrides (for A/B testing, seasonal events)
overrides:
  enabled: true
  file: overrides.yaml

# Optional: Analytics tracking
analytics:
  enabled: true
  track_missing: true
  track_usage: true
```

## Main Features

### Localization Modes

Choose how translations are loaded at runtime.

#### Embedded Mode (Default)

Translations are embedded directly in the generated Luau code. No cloud dependency required.

```yaml
localization:
  mode: embedded
```

**Benefits:**

- Works offline (no LocalizationService dependency)
- Fastest performance (direct table lookup)
- No cloud setup required
- Perfect for single-player or offline games

**Usage:**

```lua
local t = Translations.new("en")
print(t.ui.buttons:buy())  -- Direct lookup from embedded data
```

`Translations.newForPlayer(player)` is also available in embedded mode. It reads
`player.LocaleId` and falls back to `base_locale` when Roblox does not provide a
locale.

#### Cloud Mode

Uses Roblox Cloud LocalizationService exclusively. Requires uploading translations to Roblox Cloud.

```yaml
localization:
  mode: cloud
```

**Benefits:**

- Automatic Text Capture (ATC) integration
- Automatic translation via Roblox AI
- Translator Portal collaboration
- Analytics via Roblox Dashboard

**Usage:**

```lua
local t = Translations.new("en")
print(t.ui.buttons:buy())  -- Fetches from LocalizationService
```

**Requirements:**

- Must run `roblox-slang upload` to sync translations to cloud
- Requires cloud configuration in `slang-roblox.yaml`

#### Hybrid Mode

Hybrid mode tries cloud first, then uses embedded translations on error.

```yaml
localization:
  mode: hybrid
```

**Benefits:**

- Cloud features when available (ATC, auto-translation)
- Embedded fallback when LocalizationService fails
- Works in Studio and production

**Usage:**

```lua
local t = Translations.new("en")
print(t.ui.buttons:buy())  -- Tries cloud, falls back to embedded
```

**Use Cases:**

- Games transitioning from embedded to cloud
- Games that need offline support with cloud benefits
- Development (Studio) vs production (cloud) workflows

### Generated API Notes

`Translations.new(locale)` creates the namespace tables for that instance. Use
the documented colon-call form:

```lua
local t = Translations.new("en")
print(t.ui.buttons.primary:buy())
```

Hyphens and Luau keywords in translation keys are sanitized in both generated
runtime code and `.d.luau` types.

### String Interpolation

```json
{
  "welcome": "Welcome, {name}!",
  "score": "Score: {points:int}",
  "price": "Price: ${amount:fixed}"
}
```

```lua
print(t:welcome({ name = "Player" }))  -- "Welcome, Player!"
print(t:score({ points = 1234 }))      -- "Score: 1234"
print(t:price({ amount = 99.99 }))     -- "Price: $99.99"
```

### Pluralization (CLDR Rules)

```json
{
  "items": {
    "zero": "No items",
    "one": "1 item",
    "other": "{count} items"
  }
}
```

```lua
print(t:items(0))  -- "No items"
print(t:items(1))  -- "1 item"
print(t:items(5))  -- "5 items"
```

### Locale Switching

```lua
local t = Translations.new("en")
print(t.ui.buttons:buy())  -- "Buy"

t:setLocale("id")
print(t.ui.buttons:buy())  -- "Beli"
```

### Auto-Detect Player Locale

```lua
local t = Translations.newForPlayer(player)
```

`newForPlayer` works in embedded, cloud, and hybrid mode. It reads
`player.LocaleId`, normalizes the value, and uses `base_locale` as the fallback.

### Translation Overrides

For A/B testing or seasonal events:

`overrides.yaml`:

```yaml
en:
  ui.buttons.buy: "Purchase Now!"  # Override for A/B test
  
id:
  ui.buttons.buy: "Beli Sekarang!"
```

Priority: `overrides.yaml` > `translations/*.json`

## Documentation

📚 **[Complete Documentation](docs/index.md)**

Quick links:

- **[Getting Started](docs/getting-started.md)** - Installation and first project
- **[Configuration](docs/guides/configuration.md)** - Config file reference
- **[String Interpolation](docs/guides/string-interpolation.md)** - Parameter usage
- **[Pluralization](docs/guides/pluralization.md)** - CLDR plural rules
- **[Roblox Cloud Integration](docs/guides/roblox-cloud.md)** - Upload to Roblox Cloud
- **[Rojo Integration](docs/integration/rojo.md)** - Use with Rojo for automatic syncing
- **[CLI Reference](docs/reference/cli-reference.md)** - Complete command reference

## Commands

| Command | Description |
|---------|-------------|
| `roblox-slang init` | Initialize new project |
| `roblox-slang init --with-overrides` | Initialize with overrides template |
| `roblox-slang build` | Build translations once |
| `roblox-slang build --watch` | Watch mode (auto-rebuild) |
| `roblox-slang upload` | Upload translations to Roblox Cloud |
| `roblox-slang download` | Download translations from Roblox Cloud |
| `roblox-slang sync` | Bidirectional sync with merge strategies |
| `roblox-slang import <CSV_FILE>` | Import from Roblox CSV file |
| `roblox-slang validate --all` | Check for missing/unused keys and conflicts |
| `roblox-slang migrate --from <format>` | Migrate from other formats |

See [CLI Reference](docs/reference/cli-reference.md) for command details.

## Roblox Cloud Integration

Roblox Slang can sync local translation files with Roblox Cloud Localization Tables.

### Quick Start

1. **Get your API key** from [Creator Dashboard](https://create.roblox.com/dashboard/creations), go to the [API Keys](https://create.roblox.com/dashboard/credentials) page.

2. **Set environment variable**:

   ```bash
   export ROBLOX_CLOUD_API_KEY=your_api_key_here
   ```

3. **Upload translations**:

   ```bash
   roblox-slang upload --table-id YOUR_TABLE_ID
   ```

4. **Download translations**:

   ```bash
   roblox-slang download --table-id YOUR_TABLE_ID
   ```

5. **Bidirectional sync**:

   ```bash
   roblox-slang sync --table-id YOUR_TABLE_ID --strategy merge
   ```

### Cloud Sync Commands

| Command | Description |
|---------|-------------|
| `roblox-slang upload` | Upload local translations to Roblox Cloud |
| `roblox-slang download` | Download translations from Roblox Cloud |
| `roblox-slang sync` | Bidirectional sync with merge strategies |

### Merge Strategies

- **overwrite** - Upload all local translations (cloud is overwritten)
- **merge** - Upload local-only, download cloud-only, prefer cloud for conflicts
- **skip-conflicts** - Upload local-only, download cloud-only, skip conflicts

### Configuration

Add cloud settings to `slang-roblox.yaml`:

```yaml
cloud:
  table_id: YOUR_TABLE_ID
  game_id: YOUR_GAME_ID
  api_key: ${ROBLOX_CLOUD_API_KEY}  # Or set directly (not recommended)
  strategy: merge
```

### Benefits of Cloud Sync

- **Automatic Text Capture (ATC)** - Roblox captures UI strings automatically
- **Automatic translation** - Roblox AI translates to supported languages
- **Translator Portal** - Collaborate with translators via Roblox
- **Analytics** - Track translation coverage via Roblox Dashboard
- **Multi-game synchronization** - Share translations across games
- **Version control** - Keep local and cloud in sync

See [Roblox Cloud Integration Guide](docs/guides/roblox-cloud.md) for complete documentation.

## Examples

See [`tests/basic/`](tests/basic/) for a complete example with 173 translation keys across 3 locales.

## Development

For contributors:

```bash
# Clone repository
git clone https://github.com/mathtechstudio/roblox-slang.git
cd roblox-slang

# Install Rust (1.88+)
rustup override set 1.88.0

# Build
cargo build

# Run tests (103 tests)
cargo test

# Run CLI
cargo run -- --help
```

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](LICENSE)

## References

- [Roblox Localization Documentation](https://create.roblox.com/docs/production/localization)
- [Unicode CLDR Plural Rules](https://cldr.unicode.org/)
- [Luau Language Reference](https://luau-lang.org/)
