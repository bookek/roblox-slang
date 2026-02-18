# Basic Test Project

Test project with 173 translation keys across 3 locales (en, es, id).

## Structure

```yaml
translations/
├── en.json    # English (base locale)
├── es.json    # Spanish
└── id.json    # Indonesian

output/
├── Translations.lua              # Generated module
├── types/Translations.d.luau     # Type definitions
└── roblox_upload.csv             # Roblox Cloud format
```

## Usage

```lua
local Translations = require(ReplicatedStorage.Translations)

-- Create instance
local t = Translations.new("en")

-- Access translations
print(t.ui.buttons:buy())  -- "Buy"

-- With parameters
print(t.ui.messages:greeting({ name = "Player" }))  -- "Hello, Player!"

-- Pluralization
print(t.ui.messages:items(1))  -- "1 item"
print(t.ui.messages:items(5))  -- "5 items"

-- Switch locale
t:setLocale("es")
print(t.ui.buttons:buy())  -- "Comprar"
```

## Building

```bash
# Build translations
cargo run -- build

# Watch mode
cargo run -- build --watch

# Validate
cargo run -- validate
```
