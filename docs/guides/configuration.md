# Configuration Reference

Complete reference for `slang-roblox.yaml` configuration file.

## Basic Configuration

```yaml
base_locale: en
supported_locales:
  - en
  - es
  - id
input_directory: translations
output_directory: output

# Localization mode (NEW in v2.x)
localization:
  mode: embedded
```

## Configuration Options

### `base_locale` (required)

The base locale used as fallback when translations are missing.

**Type:** `string`  
**Example:** `en`, `en-US`, `id`, `es-MX`

```yaml
base_locale: en
```

**Supported Locales:**

- `en` - English
- `es` - Spanish
- `id` - Indonesian
- `pt` - Portuguese
- `de` - German
- `fr` - French
- `ja` - Japanese
- `ko` - Korean
- `zh-CN` - Chinese (Simplified)
- `zh-TW` - Chinese (Traditional)
- And [14 more Roblox-supported locales](https://create.roblox.com/docs/production/localization/language-codes)

### `supported_locales` (required)

List of locales to generate translations for.

**Type:** `array of strings`  
**Example:**

```yaml
supported_locales:
  - en
  - es
  - id
  - pt
  - de
```

**Note:** The `base_locale` should be included in this list.

### `input_directory` (required)

Directory containing translation files (JSON/YAML).

**Type:** `string`  
**Default:** `translations`

```yaml
input_directory: translations
```

**Structure:**

```yaml
translations/
├── en.json
├── es.json
└── id.json
```

### `output_directory` (required)

Directory where generated Luau code will be written.

**Type:** `string`  
**Default:** `output`

```yaml
output_directory: output
```

**Generated files:**

```yaml
output/
├── Translations.lua              # Main module
├── types/
│   └── Translations.d.luau       # Type definitions
└── roblox_upload.csv             # CSV for Roblox Cloud
```

### `namespace` (optional)

Custom namespace for generated code. If not specified, uses `Translations`.

**Type:** `string | null`  
**Default:** `null` (uses `Translations`)

```yaml
namespace: MyTranslations
```

**Usage:**

```lua
local MyTranslations = require(ReplicatedStorage.MyTranslations)
local t = MyTranslations.new("en")
```

### `localization` (optional)

**NEW in v2.x:** Configure how translations are loaded at runtime.

**Type:** `object`  
**Default:** `{ mode: "embedded" }`

```yaml
localization:
  mode: embedded
```

#### `localization.mode`

**Type:** `string`  
**Options:** `embedded`, `cloud`, `hybrid`  
**Default:** `embedded`

Controls how translations are loaded at runtime:

**`embedded` (Default)**

Translations are embedded directly in the generated Luau code. No cloud dependency.

```yaml
localization:
  mode: embedded
```

**Pros:**

- Works offline (no LocalizationService dependency)
- Fastest performance (direct table lookup)
- No cloud setup required
- Perfect for single-player or offline games

**Cons:**

- No Automatic Text Capture (ATC)
- No automatic translation via Roblox AI
- Manual translation management

**Generated code:**

```lua
-- Translations embedded in EMBEDDED_TRANSLATIONS table
local t = Translations.new("en")
print(t.ui.buttons.buy())  -- Direct lookup from embedded data
```

**`cloud`**

Uses Roblox Cloud LocalizationService exclusively. Requires uploading translations to Roblox Cloud.

```yaml
localization:
  mode: cloud
```

**Pros:**

- Automatic Text Capture (ATC) integration
- Automatic translation via Roblox AI
- Translator Portal collaboration
- Analytics via Roblox Dashboard

**Cons:**

- Requires cloud setup and upload
- Depends on LocalizationService availability
- Requires internet connection

**Generated code:**

```lua
-- Uses LocalizationService:GetTranslatorForLocaleAsync()
local t = Translations.new("en")
print(t.ui.buttons.buy())  -- Fetches from LocalizationService
```

**Requirements:**

- Must run `roblox-slang upload` to sync translations to cloud
- Requires cloud configuration in `slang-roblox.yaml`

**`hybrid`**

Best of both worlds: tries cloud first, falls back to embedded on error.

```yaml
localization:
  mode: hybrid
```

**Pros:**

- Cloud features when available (ATC, auto-translation)
- Embedded fallback for reliability
- Graceful degradation on cloud errors
- Works in Studio and production

**Cons:**

- Slightly larger generated file size (includes embedded data)
- More complex setup

**Generated code:**

```lua
-- Tries LocalizationService with pcall, falls back to embedded
local t = Translations.new("en")
print(t.ui.buttons.buy())  -- Tries cloud, falls back to embedded
```

**Use Cases:**

- Games transitioning from embedded to cloud
- Games that need offline support with cloud benefits
- Development (Studio) vs production (cloud) workflows

**Comparison:**

| Feature | Embedded | Cloud | Hybrid |
|---------|----------|-------|--------|
| Offline support | ✅ Yes | ❌ No | ✅ Yes |
| Cloud features (ATC, auto-translate) | ❌ No | ✅ Yes | ✅ Yes |
| Performance | ⚡ Fastest | 🐢 Slower | 🚀 Fast |
| Setup complexity | 🟢 Simple | 🔴 Complex | 🟡 Medium |
| File size | 📦 Larger | 📄 Smaller | 📦 Larger |
| Reliability | ✅ High | ⚠️ Depends on cloud | ✅ High |

## Advanced Configuration

### Translation Overrides

Override specific translations for A/B testing or seasonal events.

```yaml
overrides:
  enabled: true
  file: overrides.yaml
```

**`overrides.enabled`**  
**Type:** `boolean`  
**Default:** `false`

Enable/disable override system.

**`overrides.file`**  
**Type:** `string`  
**Default:** `overrides.yaml`

Path to overrides file (relative to project root).

**Example `overrides.yaml`:**

```yaml
en:
  ui.buttons.buy: "Purchase Now!"  # Override for A/B test
  
es:
  ui.buttons.buy: "¡Comprar Ahora!"
```

**Priority:** `overrides.yaml` > `translations/*.json`

### Analytics

Track missing translations and usage statistics.

```yaml
analytics:
  enabled: true
  track_missing: true
  track_usage: true
```

**`analytics.enabled`**  
**Type:** `boolean`  
**Default:** `false`

Enable analytics tracking in generated code.

**`analytics.track_missing`**  
**Type:** `boolean`  
**Default:** `false`

Track when translations are missing (logs to console).

**`analytics.track_usage`**  
**Type:** `boolean`  
**Default:** `false`

Track how often each translation is accessed.

**Usage:**

```lua
local t = Translations.new("en")

-- Get usage statistics
local stats = t:getUsageStats()
print(stats["ui.buttons.buy"])  -- Number of times accessed

-- Get missing translations
local missing = t:getMissingKeys()
for _, key in ipairs(missing) do
    print("Missing:", key)
end
```

## Complete Example

```yaml
# Base locale (fallback)
base_locale: en

# Supported locales
supported_locales:
  - en
  - es
  - id
  - pt
  - de
  - fr
  - ja
  - ko

# Input/output directories
input_directory: translations
output_directory: src/shared/Translations

# Custom namespace (optional)
namespace: null

# Localization mode (NEW in v2.x)
localization:
  mode: embedded  # or "cloud" or "hybrid"

# Translation overrides (for A/B testing, seasonal events)
overrides:
  enabled: true
  file: overrides.yaml

# Analytics tracking
analytics:
  enabled: true
  track_missing: true
  track_usage: true
```

## Environment Variables

You can use environment variables in configuration:

```yaml
base_locale: ${BASE_LOCALE:-en}
output_directory: ${OUTPUT_DIR:-output}
```

**Syntax:** `${VAR_NAME:-default_value}`

**Example:**

```bash
export BASE_LOCALE=es
export OUTPUT_DIR=dist/translations
roblox-slang build
```

## Configuration Validation

Validate your configuration:

```bash
roblox-slang validate
```

This checks:

- Required fields are present
- Locale codes are valid
- Directories exist
- Translation files are valid JSON/YAML
- No duplicate keys
- All locales have same keys

## Best Practices

### 1. Keep Base Locale Complete

Always ensure your base locale has all translation keys. Other locales can be incomplete (will fallback to base).

### 2. Use Consistent Naming

Use dot notation for nested keys:

```json
{
  "ui.buttons.buy": "Buy",
  "ui.buttons.sell": "Sell"
}
```

Or nested structure:

```json
{
  "ui": {
    "buttons": {
      "buy": "Buy",
      "sell": "Sell"
    }
  }
}
```

Both work, but nested is more readable.

### 3. Organize by Feature

Group translations by feature/screen:

```json
{
  "shop": { ... },
  "inventory": { ... },
  "settings": { ... }
}
```

### 4. Use Overrides Sparingly

Only use overrides for temporary changes (A/B tests, events). Keep main translations in translation files.

### 5. Enable Analytics in Development

Enable analytics during development to find unused translations:

```yaml
analytics:
  enabled: true
  track_usage: true
```

Then review usage stats before release.

## Migration from Old Config

If you have an old configuration format, migrate using:

```bash
roblox-slang migrate config
```

This will convert old format to new format automatically.

## See Also

- [Getting Started](../getting-started.md) - Initial setup guide
- [Pluralization](pluralization.md) - Handle plural forms
- [String Interpolation](string-interpolation.md) - Parameter usage
- [Roblox Cloud Integration](roblox-cloud.md) - Upload to Roblox Cloud
- [Rojo Integration](../integration/rojo.md) - Use with Rojo
