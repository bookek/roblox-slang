# Roblox Slang Documentation

Docs for Roblox Slang, a translation generator for Roblox games.

## Getting Started

Start here:

- **[Getting Started](getting-started.md)** - Installation and first project setup

## Guides

Learn about the main features:

- **[Configuration](guides/configuration.md)** - Configuration reference
- **[String Interpolation](guides/string-interpolation.md)** - Use parameters in translations
- **[Pluralization](guides/pluralization.md)** - Handle plural forms with CLDR rules
- **[Roblox Cloud Integration](guides/roblox-cloud.md)** - Upload to Roblox Cloud for access to Roblox features

## Integration

- **[Rojo Integration](integration/rojo.md)** - Use with Rojo for automatic file syncing

## Reference

- **[CLI Reference](reference/cli-reference.md)** - CLI command reference

## Quick Links

### Installation

```bash
# Via Rokit (recommended)
rokit add mathtechstudio/roblox-slang
rokit install

# Via Aftman
aftman add mathtechstudio/roblox-slang
aftman install
```

### Quick Start

```bash
# Initialize project
roblox-slang init

# Create translations (edit translations/en.json)

# Build
roblox-slang build

# Watch mode
roblox-slang build --watch
```

### Basic Usage

```lua
local Translations = require(ReplicatedStorage.Translations)
local t = Translations.new("en")

-- Simple translation
print(t.ui.buttons:buy())  -- "Buy"

-- With parameters
print(t:greeting({ name = "Player1" }))  -- "Hello, Player1!"

-- Pluralization
print(t:items(5))  -- "5 items"

-- Switch locale
t:setLocale("es")
print(t.ui.buttons:buy())  -- "Comprar"

-- Player locale
local playerTranslations = Translations.newForPlayer(player)
```

## Documentation Structure

```yaml
docs/
├── getting-started.md
├── guides/
│   ├── configuration.md
│   ├── pluralization.md
│   ├── roblox-cloud.md
│   └── string-interpolation.md
├── integration/
│   └── rojo.md
└── reference/
    └── cli-reference.md
```

## External Resources

### Official Documentation

- [Roblox Localization Docs](https://create.roblox.com/docs/production/localization) - Official Roblox localization guide
- [Roblox Language Codes](https://create.roblox.com/docs/production/localization/language-codes) - Supported locales
- [Open Cloud API](https://create.roblox.com/docs/cloud/open-cloud/localization-api) - API reference

### Standards

- [Unicode CLDR](https://cldr.unicode.org/) - Common Locale Data Repository
- [CLDR Plural Rules](https://www.unicode.org/cldr/charts/latest/supplemental/language_plural_rules.html) - Plural form rules
- [ICU MessageFormat](https://unicode-org.github.io/icu/userguide/format_parse/messages/) - Message formatting

### Inspiration

- [Flutter Slang](https://pub.dev/packages/slang) - Primary inspiration for this project
- [Luau Language](https://luau-lang.org/) - Roblox's scripting language

## Contributing

Want to contribute? See:

- [CONTRIBUTING.md](../CONTRIBUTING.md) - Contribution guidelines
- [GitHub Issues](https://github.com/mathtechstudio/roblox-slang/issues) - Report bugs or request features

## Support

Need help?

- [GitHub Issues](https://github.com/mathtechstudio/roblox-slang/issues) - Report bugs
- [Roblox DevForum](https://devforum.roblox.com/) - Community support

## License

Roblox Slang is licensed under the [MIT License](../LICENSE).
