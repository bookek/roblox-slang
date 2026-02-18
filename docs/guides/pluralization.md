# Pluralization

Handle plural forms correctly using CLDR (Common Locale Data Repository) rules.

## Why Pluralization Matters

Different languages have different plural rules. English has 2 forms (singular/plural), but other languages can have more:

- **English:** 1 item, 2 items
- **Polish:** 1 przedmiot, 2 przedmioty, 5 przedmiotów (3 forms!)
- **Arabic:** 0 عناصر, 1 عنصر, 2 عنصران, 3 عناصر, 11 عنصرًا, 100 عنصر (6 forms!)

Roblox Slang uses CLDR rules to handle this automatically.

## Basic Pluralization

### English Example

```json
{
  "items": {
    "zero": "No items",
    "one": "1 item",
    "other": "{count} items"
  }
}
```

**Usage:**

```lua
local t = Translations.new("en")

print(t.items(0))   -- "No items"
print(t.items(1))   -- "1 item"
print(t.items(5))   -- "5 items"
print(t.items(100)) -- "100 items"
```

### Spanish Example

```json
{
  "items": {
    "one": "1 artículo",
    "other": "{count} artículos"
  }
}
```

**Usage:**

```lua
local t = Translations.new("es")

print(t.items(1))  -- "1 artículo"
print(t.items(5))  -- "5 artículos"
```

## CLDR Plural Categories

Roblox Slang supports all 6 CLDR plural categories:

| Category | Description | Example (English) |
|----------|-------------|-------------------|
| `zero` | Exactly zero | 0 items |
| `one` | Singular | 1 item |
| `two` | Exactly two | 2 items (some languages) |
| `few` | Small number | 3-10 items (some languages) |
| `many` | Large number | 11+ items (some languages) |
| `other` | Default/fallback | Any other count |

**Note:** Not all languages use all categories. English only uses `one` and `other`.

## Language-Specific Rules

### English (en)

**Rules:**

- `one`: n = 1
- `other`: everything else

```json
{
  "items": {
    "one": "1 item",
    "other": "{count} items"
  }
}
```

### Spanish (es)

**Rules:**

- `one`: n = 1
- `other`: everything else

```json
{
  "items": {
    "one": "1 artículo",
    "other": "{count} artículos"
  }
}
```

### Indonesian (id)

**Rules:**

- `other`: all numbers (no singular/plural distinction)

```json
{
  "items": {
    "other": "{count} item"
  }
}
```

### Polish (pl)

**Rules:**

- `one`: n = 1
- `few`: n = 2-4, 22-24, 32-34, etc.
- `many`: n = 0, 5-21, 25-31, etc.
- `other`: fractions

```json
{
  "items": {
    "one": "1 przedmiot",
    "few": "{count} przedmioty",
    "many": "{count} przedmiotów",
    "other": "{count} przedmiotu"
  }
}
```

### Arabic (ar)

**Rules:**

- `zero`: n = 0
- `one`: n = 1
- `two`: n = 2
- `few`: n = 3-10
- `many`: n = 11-99
- `other`: n = 100+, fractions

```json
{
  "items": {
    "zero": "لا عناصر",
    "one": "عنصر واحد",
    "two": "عنصران",
    "few": "{count} عناصر",
    "many": "{count} عنصرًا",
    "other": "{count} عنصر"
  }
}
```

## Advanced Usage

### With Parameters

Combine pluralization with other parameters:

```json
{
  "playerItems": {
    "one": "{name} has 1 item",
    "other": "{name} has {count} items"
  }
}
```

**Usage:**

```lua
print(t.playerItems(1, { name = "Player1" }))
-- "Player1 has 1 item"

print(t.playerItems(5, { name = "Player1" }))
-- "Player1 has 5 items"
```

### Zero Category

Use `zero` for special handling of zero:

```json
{
  "coins": {
    "zero": "No coins",
    "one": "1 coin",
    "other": "{count} coins"
  }
}
```

**Usage:**

```lua
print(t.coins(0))  -- "No coins" (not "0 coins")
print(t.coins(1))  -- "1 coin"
print(t.coins(5))  -- "5 coins"
```

### Nested Plurals

Plurals can be nested in objects:

```json
{
  "inventory": {
    "items": {
      "one": "1 item",
      "other": "{count} items"
    },
    "weapons": {
      "one": "1 weapon",
      "other": "{count} weapons"
    }
  }
}
```

**Usage:**

```lua
print(t.inventory:items(5))    -- "5 items"
print(t.inventory:weapons(1))  -- "1 weapon"
```

## Format Specifiers with Plurals

Use format specifiers to control number formatting:

```json
{
  "price": {
    "one": "${amount:fixed} for 1 item",
    "other": "${amount:fixed} for {count} items"
  }
}
```

**Usage:**

```lua
print(t.price(1, { amount = 9.99 }))
-- "$9.99 for 1 item"

print(t.price(5, { amount = 49.95 }))
-- "$49.95 for 5 items"
```

## Best Practices

### 1. Always Include `other`

The `other` category is required as fallback:

```json
{
  "items": {
    "one": "1 item",
    "other": "{count} items"  // Required!
  }
}
```

### 2. Use `zero` for Better UX

Instead of "0 items", use more natural language:

```json
{
  "items": {
    "zero": "No items",      // Better UX
    "one": "1 item",
    "other": "{count} items"
  }
}
```

### 3. Test with Different Numbers

Test your plurals with various numbers:

- 0 (zero)
- 1 (one)
- 2 (two, if applicable)
- 3-10 (few, if applicable)
- 11-99 (many, if applicable)
- 100+ (other)

### 4. Check Language Rules

Always check CLDR rules for your target language:

- [CLDR Plural Rules](https://www.unicode.org/cldr/charts/latest/supplemental/language_plural_rules.html)

### 5. Avoid Hardcoding Numbers

Don't hardcode "1" in translations:

❌ **Bad:**

```json
{
  "items": {
    "one": "1 item",
    "other": "{count} items"
  }
}
```

✅ **Good:**

```json
{
  "items": {
    "one": "{count} item",  // Use {count} even for singular
    "other": "{count} items"
  }
}
```

This allows flexibility for languages where "1" might be displayed differently.

## Validation

Validate your plural translations:

```bash
roblox-slang validate
```

This checks:

- All plural forms have `other` category
- Plural categories are valid for the locale
- Parameters are consistent across categories

## Common Mistakes

### Mistake 1: Missing `other`

❌ **Wrong:**

```json
{
  "items": {
    "one": "1 item"
    // Missing "other"!
  }
}
```

✅ **Correct:**

```json
{
  "items": {
    "one": "1 item",
    "other": "{count} items"
  }
}
```

### Mistake 2: Using Wrong Categories

❌ **Wrong (English doesn't use `few`):**

```json
{
  "items": {
    "one": "1 item",
    "few": "{count} items",  // English doesn't have "few"
    "other": "{count} items"
  }
}
```

✅ **Correct:**

```json
{
  "items": {
    "one": "1 item",
    "other": "{count} items"
  }
}
```

### Mistake 3: Inconsistent Parameters

❌ **Wrong:**

```json
{
  "items": {
    "one": "{name} has 1 item",
    "other": "{count} items"  // Missing {name}!
  }
}
```

✅ **Correct:**

```json
{
  "items": {
    "one": "{name} has 1 item",
    "other": "{name} has {count} items"
  }
}
```

## Reference

### Supported Locales and Their Plural Rules

| Locale | Categories Used |
|--------|----------------|
| English (en) | one, other |
| Spanish (es) | one, other |
| Indonesian (id) | other |
| Portuguese (pt) | one, other |
| German (de) | one, other |
| French (fr) | one, other |
| Japanese (ja) | other |
| Korean (ko) | other |
| Chinese (zh) | other |
| Polish (pl) | one, few, many, other |
| Russian (ru) | one, few, many, other |
| Arabic (ar) | zero, one, two, few, many, other |

For complete rules, see [CLDR Plural Rules](https://www.unicode.org/cldr/charts/latest/supplemental/language_plural_rules.html).

## See Also

- [Getting Started](../getting-started.md) - Initial setup
- [String Interpolation](string-interpolation.md) - Parameter usage
- [Configuration](configuration.md) - Config options
- [Rojo Integration](../integration/rojo.md) - Use with Rojo
- [CLDR Plural Rules](https://www.unicode.org/cldr/charts/latest/supplemental/language_plural_rules.html) - Official reference
