# Getting Started

Complete guide to get started with Roblox Slang in your project.

## Prerequisites

- Roblox Studio installed
- Basic knowledge of Luau scripting
- A Roblox game project (or create a new one)

## Installation

### Step 1: Install the CLI Tool

Choose your preferred toolchain manager. All three options work identically - they automatically download the correct pre-built binary for your platform from GitHub Releases.

#### Option A: Rokit (Recommended)

[Rokit](https://github.com/rojo-rbx/rokit) is the fastest and most modern toolchain manager.

```bash
# Add to your project
rokit add mathtechstudio/roblox-slang

# Or install globally
rokit add --global mathtechstudio/roblox-slang
```

**rokit.toml:**

```toml
[tools]
roblox-slang = "mathtechstudio/roblox-slang@2.0.2"
```

#### Option B: Aftman

> **Note:** Aftman is no longer actively maintained. We recommend using [Rokit](#option-a-rokit-recommended) or [Foreman](#option-c-foreman) for new projects.

[Aftman](https://github.com/LPGhatguy/aftman) provides exact version dependencies and trust-based security.

```bash
# Add to your project
aftman add mathtechstudio/roblox-slang

# Or install globally
aftman add --global mathtechstudio/roblox-slang
```

**aftman.toml:**

```toml
[tools]
roblox-slang = "mathtechstudio/roblox-slang@2.0.2"
```

#### Option C: Foreman

[Foreman](https://github.com/Roblox/foreman) is the original Roblox toolchain manager, battle-tested in production.

**foreman.toml:**

```toml
[tools]
roblox-slang = { github = "mathtechstudio/roblox-slang", version = "2.0.2" }
```

```bash
foreman install
```

#### Option D: Manual Installation

Download pre-built binaries from [GitHub Releases](https://github.com/mathtechstudio/roblox-slang/releases):

- `roblox-slang-2.0.2-linux-x86_64.zip`
- `roblox-slang-2.0.2-linux-aarch64.zip`
- `roblox-slang-2.0.2-windows-x86_64.zip`
- `roblox-slang-2.0.2-windows-aarch64.zip`
- `roblox-slang-2.0.2-macos-x86_64.zip`
- `roblox-slang-2.0.2-macos-aarch64.zip`

Extract the archive and add the binary to your PATH.

#### Option E: From Source (Cargo)

```bash
# Install from crates.io
cargo install roblox-slang

# Or build from source
git clone https://github.com/mathtechstudio/roblox-slang.git
cd roblox-slang
cargo install --locked --path .
```

### Step 2: Verify Installation

```bash
roblox-slang --version
```

You should see: `roblox-slang 0.1.0`

## Creating Your First Translation Project

### 1. Initialize Project

Navigate to your Roblox project directory and run:

```bash
roblox-slang init
```

This creates:

- `slang-roblox.yaml` - Configuration file
- `translations/` - Directory for translation files

### 2. Configure Your Project

Edit `slang-roblox.yaml`:

```yaml
# Base locale (fallback when translation missing)
base_locale: en

# List of supported locales
supported_locales:
  - en
  - es
  - id

# Where to find translation files
input_directory: translations

# Where to generate Luau code
output_directory: src/shared/Translations

# Localization mode (NEW in v2.0.2)
localization:
  # embedded: Translations embedded in code (default, no cloud dependency)
  # cloud: Use Roblox Cloud LocalizationService only
  # hybrid: Try cloud first, fallback to embedded
  mode: embedded
```

### 3. Create Translation Files

Create `translations/en.json`:

```json
{
  "welcome": "Welcome to my game!",
  "ui": {
    "buttons": {
      "play": "Play",
      "settings": "Settings",
      "quit": "Quit"
    },
    "messages": {
      "loading": "Loading...",
      "playerJoined": "{name} joined the game"
    }
  }
}
```

Create `translations/es.json`:

```json
{
  "welcome": "¡Bienvenido a mi juego!",
  "ui": {
    "buttons": {
      "play": "Jugar",
      "settings": "Configuración",
      "quit": "Salir"
    },
    "messages": {
      "loading": "Cargando...",
      "playerJoined": "{name} se unió al juego"
    }
  }
}
```

### 4. Build Translations

```bash
roblox-slang build
```

This generates:

- `src/shared/Translations/Translations.lua` - Main module
- `src/shared/Translations/types/Translations.d.luau` - Type definitions
- `src/shared/Translations/roblox_upload.csv` - CSV for Roblox Cloud

### 5. Use in Your Game

In a LocalScript or Script:

```lua
local ReplicatedStorage = game:GetService("ReplicatedStorage")
local Translations = require(ReplicatedStorage.Translations)

-- Create translation instance
local t = Translations.new("en")

-- Use translations
print(t:welcome())  -- "Welcome to my game!"
print(t.ui.buttons:play())  -- "Play"

-- With parameters
print(t.ui.messages:playerJoined({ name = "Player1" }))
-- Output: "Player1 joined the game"

-- Switch locale at runtime
t:setLocale("es")
print(t:welcome())  -- "¡Bienvenido a mi juego!"
```

### 6. Auto-Detect Player Locale

For multiplayer games, automatically use player's locale:

```lua
local Players = game:GetService("Players")
local ReplicatedStorage = game:GetService("ReplicatedStorage")
local Translations = require(ReplicatedStorage.Translations)

Players.PlayerAdded:Connect(function(player)
    -- Automatically detects player's country/locale
    local t = Translations.newForPlayer(player)
    
    -- Send localized welcome message
    local welcomeMsg = t.welcome()
    -- Send to player...
end)
```

## Development Workflow

### Watch Mode

For faster development, use watch mode to auto-rebuild on file changes:

```bash
roblox-slang build --watch
```

Now when you edit translation files, the Luau code is automatically regenerated.

### Validation

Check for missing translations or unused keys:

```bash
roblox-slang validate
```

This will report:

- Missing translations (keys in base locale but not in others)
- Unused keys (keys defined but never used in code)
- Conflicts (duplicate keys)

## Next Steps

- [Configuration Guide](guides/configuration.md) - Learn all configuration options including localization modes
- [Pluralization](guides/pluralization.md) - Handle plural forms correctly
- [String Interpolation](guides/string-interpolation.md) - Advanced parameter usage
- [Roblox Cloud Integration](guides/roblox-cloud.md) - Upload to Roblox Cloud for access to Roblox features
- [Rojo Integration](integration/rojo.md) - Use with Rojo for automatic file syncing

## Choosing a Localization Mode

**NEW in v2.0.2:** Roblox Slang supports three localization modes:

### Embedded Mode (Default)

Best for most games. Translations are embedded directly in the generated code.

**Pros:**

- No cloud dependency
- Works offline
- Fastest performance
- Simple setup

**Cons:**

- No Automatic Text Capture
- No automatic translation
- Manual translation management

**When to use:** Single-player games, offline games, or when you want full control over translations.

### Cloud Mode

Uses Roblox Cloud LocalizationService exclusively.

**Pros:**

- Automatic Text Capture (ATC)
- Automatic translation via Roblox AI
- Translator Portal collaboration
- Analytics via Roblox Dashboard

**Cons:**

- Requires cloud setup
- Requires uploading translations
- Depends on LocalizationService availability

**When to use:** Multiplayer games that need automatic translation or want to use Roblox's translation ecosystem.

### Hybrid Mode

Best of both worlds: tries cloud first, falls back to embedded.

**Pros:**

- Cloud features when available
- Embedded fallback for reliability
- Graceful degradation
- Works in Studio and production

**Cons:**

- Slightly larger generated file size
- More complex setup

**When to use:** Games transitioning from embedded to cloud, or games that need offline support with cloud benefits.

## Common Issues

### Issue: "Command not found: roblox-slang"

**Solution:** Make sure the toolchain manager added the binary to your PATH. Try:

```bash
# For Rokit
rokit list

# For Aftman
aftman list

# For Foreman
foreman list
```

### Issue: "Failed to read config file"

**Solution:** Make sure you're in the project directory with `slang-roblox.yaml`. Run:

```bash
ls -la slang-roblox.yaml
```

### Issue: "Translation file not found"

**Solution:** Check that translation files exist in the `input_directory` specified in config:

```bash
ls -la translations/
```

## Getting Help

- [GitHub Issues](https://github.com/mathtechstudio/roblox-slang/issues) - Report bugs or request features
