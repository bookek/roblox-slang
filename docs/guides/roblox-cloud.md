# Roblox Cloud Sync Guide

This guide explains how to use Roblox Slang's cloud synchronization features to manage translations with Roblox Cloud Localization Tables.

## Overview

Roblox Slang provides bidirectional synchronization between your local translation files and Roblox Cloud Localization Tables. This enables:

- **Upload**: Push local translations to Roblox Cloud
- **Download**: Pull cloud translations to local files
- **Sync**: Bidirectional sync with conflict resolution

**NEW in v2.x:** You can now choose how translations are loaded at runtime:

- **Embedded mode** (default): Translations embedded in code, no cloud dependency
- **Cloud mode**: Uses Roblox Cloud LocalizationService exclusively
- **Hybrid mode**: Tries cloud first, falls back to embedded

This guide focuses on cloud sync commands. For runtime localization modes, see [Configuration Guide](configuration.md#localization).

## Prerequisites

Before using cloud sync features, you need:

1. A Roblox Cloud API key with Localization Table permissions
2. A Localization Table ID from your Roblox game

### Getting Your API Key

Roblox Cloud uses API keys to authenticate and authorize API access with granular permissions and security controls.

1. Go to [Creator Dashboard](https://create.roblox.com/dashboard/creations)
2. Navigate to **API Keys** page
3. Click **Create API Key**
4. Configure your key:
   - **Name**: Give it a descriptive name (e.g., "Slang Localization Sync")
   - **Access Permissions**:
     - Select **Localization Tables** from the **Select API System** menu
     - Select your experience (or disable **Restrict by Experience** for all experiences)
     - From **Select Operations**, choose the operations you need:
       - `Read` - For downloading translations
       - `Write` - For uploading translations
   - **Security** (optional but recommended):
     - **IP Restrictions**: Add IP addresses using CIDR notation (e.g., `192.168.0.0/24`)
       - Note: Do not use IP restrictions if using the key in Roblox places
     - **Expiration Date**: Set an expiration date for additional security

5. Click **Save & Generate Key**
6. **Copy the API key immediately** - you won't be able to see it again!

**Security Best Practices** (from Roblox recommendations):

- **Never share API keys** through public channels, forums, or social media
- **Never store keys in source code** or version control systems
- **Use secrets management** systems for storing keys securely
- **Create separate keys** for each application or use case
- **Use minimum permissions** - only select operations you actually need
- **Set IP restrictions** when possible (except for Roblox place usage)
- **Set expiration dates** for short-term use cases

**For Group-Owned Experiences:**

If managing translations for a group-owned experience, Roblox strongly recommends:

1. Create a **dedicated alternate account** for automation
2. Invite the account to your group with **minimal permissions**
3. Assign a role with only the permissions needed
4. Create the API key on this dedicated account
5. Use this key for group automation only

This prevents compromising your personal account's access to other resources.

### Finding Your Table ID

1. Open your game in Roblox Studio
2. Go to the Localization Table
3. The Table ID is in the URL or table properties

## Authentication

There are two ways to provide your API key:

### Option 1: Environment Variable (Recommended)

```bash
export ROBLOX_CLOUD_API_KEY="your_api_key_here"
```

Add this to your `~/.bashrc` or `~/.zshrc` for persistence.

### Option 2: Configuration File

Add to your `slang-roblox.yaml`:

```yaml
cloud:
  api_key: "your_api_key_here"  # Not recommended for version control
  table_id: "your_table_id"
  strategy: "merge"
```

**Security Note**:

- Never commit API keys to version control
- Use environment variables or secrets management systems
- If using config file, add it to `.gitignore`
- API keys are equivalent to passwords - treat them securely
- Keys automatically expire after 60 days of inactivity (Roblox security feature)

## Configuration

Add cloud configuration to `slang-roblox.yaml`:

```yaml
base_locale: en
supported_locales:
  - en
  - es
  - pt
input_directory: translations
output_directory: output

# Cloud sync configuration
cloud:
  table_id: "your_table_id_here"
  strategy: "merge"  # overwrite, merge, or skip-conflicts

# Localization mode (NEW in v2.x)
# Use "cloud" mode to load translations from LocalizationService at runtime
localization:
  mode: embedded  # or "cloud" or "hybrid"
```

**Note:** The `cloud` configuration above is for sync commands (upload/download/sync). The `localization.mode` controls how translations are loaded at runtime in your game. See [Using Cloud Mode](#using-cloud-mode) below.

## Using Cloud Mode

**NEW in v2.x:** After uploading translations to Roblox Cloud, you can configure your game to load translations from LocalizationService at runtime.

### Cloud Mode Configuration

```yaml
localization:
  mode: cloud
```

**Benefits:**

- Automatic Text Capture (ATC) integration
- Automatic translation via Roblox AI
- Translator Portal collaboration
- Analytics via Roblox Dashboard

**Requirements:**

1. Upload translations to Roblox Cloud first: `roblox-slang upload`
2. Set `localization.mode: cloud` in config
3. Rebuild: `roblox-slang build`

**Generated code will use LocalizationService:**

```lua
-- Generated Translations.lua uses LocalizationService
local t = Translations.new("en")
print(t.ui.buttons:buy())  -- Fetches from LocalizationService
```

### Hybrid Mode (Recommended)

For best reliability, use hybrid mode which tries cloud first, falls back to embedded:

```yaml
localization:
  mode: hybrid
```

**Benefits:**

- Cloud features when available
- Embedded fallback for reliability
- Works in Studio and production

**Workflow:**

1. Upload to cloud: `roblox-slang upload`
2. Set hybrid mode in config
3. Build: `roblox-slang build`
4. Game tries cloud first, uses embedded if cloud fails

See [Configuration Guide](configuration.md#localization) for complete localization mode documentation.

## Commands

### Upload Command

Upload local translations to Roblox Cloud:

```bash
# Upload with table ID from config
roblox-slang upload

# Upload with explicit table ID
roblox-slang upload --table-id your_table_id

# Preview changes without uploading (dry-run)
roblox-slang upload --dry-run

# Skip validation before upload
roblox-slang upload --skip-validation
```

**What it does:**

- Reads all local translation files
- Validates translations (unless skipped)
- Converts to Roblox Cloud format
- Uploads to specified table
- Shows statistics (entries uploaded, locales processed, duration)

**Example output:**

```bash
→ Running pre-upload validation...
✓ Validation passed
→ Uploading translations to cloud...
  Table ID: abc123
  Locales: en, es, pt

✓ Upload complete!
  Entries uploaded: 150
  Locales processed: 3
  Duration: 2.34s

✓ Translations successfully uploaded to Roblox Cloud
```

### Download Command

Download translations from Roblox Cloud to local files:

```bash
# Download with table ID from config
roblox-slang download

# Download with explicit table ID
roblox-slang download --table-id your_table_id

# Preview changes without writing files (dry-run)
roblox-slang download --dry-run
```

**What it does:**

- Downloads translations from Roblox Cloud
- Converts to nested JSON format
- Writes one file per locale
- Shows statistics (entries downloaded, locales created/updated, duration)

**Example output:**

```bash
→ Downloading translations from cloud...
  Table ID: abc123

✓ Download complete!
  Entries downloaded: 150
  Locales created: 1
  Locales updated: 2
  Duration: 1.87s

✓ Translations successfully downloaded from Roblox Cloud
```

### Sync Command

Bidirectional synchronization with conflict resolution:

```bash
# Sync with default strategy from config
roblox-slang sync

# Sync with explicit strategy
roblox-slang sync --strategy merge

# Preview changes without syncing (dry-run)
roblox-slang sync --dry-run

# Sync with explicit table ID
roblox-slang sync --table-id your_table_id --strategy overwrite
```

**Merge Strategies:**

1. **overwrite**: Replace all cloud translations with local
   - Use when: Local is source of truth
   - Effect: Cloud = Local (complete replacement)

2. **merge** (recommended): Merge local and cloud, prefer cloud for conflicts
   - Use when: Want both local and cloud changes
   - Effect: Union of both, cloud wins conflicts

3. **skip-conflicts**: Only sync non-conflicting entries
   - Use when: Want manual conflict resolution
   - Effect: Syncs safe changes, reports conflicts

**Example output:**

```bash
→ Synchronizing translations (strategy: merge)...
  Table ID: abc123
  Merge strategy: merge

✓ Sync complete!
  Entries added: 25
  Entries updated: 10
  Entries deleted: 0
  ⚠ Conflicts skipped: 3
  Duration: 3.12s

ℹ Conflicts saved to: output/conflicts.yaml
  Review and resolve conflicts manually

✓ Translations successfully synchronized
```

## Conflict Resolution

When using `skip-conflicts` strategy, conflicts are saved to `output/conflicts.yaml`:

```yaml
# Translation Conflicts
# Resolve these conflicts manually

en:
  ui.button.buy:
    local: "Buy Now"
    cloud: "Purchase"
  
es:
  ui.button.buy:
    local: "Comprar Ahora"
    cloud: "Comprar"
```

**To resolve conflicts:**

1. Review the conflicts file
2. Decide which value to keep
3. Update your local translation files
4. Run sync again with `merge` or `overwrite` strategy

## Dry-Run Mode

All commands support `--dry-run` to preview changes without making them:

```bash
roblox-slang upload --dry-run
roblox-slang download --dry-run
roblox-slang sync --dry-run
```

**Dry-run mode:**

- ✓ Reads local files
- ✓ Fetches from cloud
- ✓ Computes changes
- ✓ Shows statistics
- ✗ Does NOT upload to cloud
- ✗ Does NOT write local files

Use dry-run to:

- Preview changes before applying
- Test configuration
- Verify API key works
- Check for conflicts

## Workflows

### Initial Setup

```bash
# 1. Initialize project
roblox-slang init

# 2. Add cloud config to slang-roblox.yaml
# 3. Set API key environment variable
export ROBLOX_CLOUD_API_KEY="your_key"

# 4. Upload initial translations
roblox-slang upload

# 5. Verify in Roblox Creator Dashboard
```

### Regular Development

```bash
# 1. Make changes to local translations
# 2. Test locally
roblox-slang build

# 3. Upload to cloud
roblox-slang upload

# 4. Enable auto-translation in Roblox Dashboard
# 5. Download translated versions
roblox-slang download
```

### Team Collaboration

```bash
# 1. Pull latest from cloud
roblox-slang download

# 2. Make local changes
# 3. Sync with merge strategy
roblox-slang sync --strategy merge

# 4. Resolve any conflicts
# 5. Push to version control
```

### Migration from Existing Table

```bash
# 1. Download existing translations
roblox-slang download

# 2. Review downloaded files
# 3. Organize as needed
# 4. Upload back
roblox-slang upload
```

## Best Practices

### Security

- ✓ Use environment variables for API keys
- ✓ Add `slang-roblox.yaml` to `.gitignore` if it contains secrets
- ✓ Use separate API keys for dev/prod
- ✗ Never commit API keys to version control

### Workflow

- ✓ Use `--dry-run` before important operations
- ✓ Run validation before upload
- ✓ Use `merge` strategy for team collaboration
- ✓ Review conflicts before resolving
- ✓ Keep local files as source of truth

### Performance

- ✓ Upload during off-peak hours for large tables
- ✓ Use `--skip-validation` only when necessary
- ✓ Batch changes instead of frequent small uploads

## Troubleshooting

### Authentication Errors

```bash
Error: Authentication failed
```

**Solutions:**

- Verify API key is correct
- Check API key has Localization Table permissions
- Ensure API key hasn't expired
- Check API key status in Creator Dashboard

### API Key Status

API keys can have different statuses that affect their usability:

| Status | Reason | Resolution |
|--------|--------|------------|
| **Active** | No issues | Key works normally |
| **Disabled** | User disabled the key | Enable the key in Creator Dashboard |
| **Expired** | Expiration date passed | Remove or set new expiration date |
| **Auto-Expired** | Not used/updated for 60 days | Disable then enable the key, or update any property |
| **Revoked** | (Group keys) Account lost permissions | Click "Regenerate Key" in Creator Dashboard |
| **Moderated** | Roblox admin changed secret | Click "Regenerate Key" in Creator Dashboard |
| **User Moderated** | Account under moderation | Resolve moderation issue on account |

**Note**: Roblox automatically expires API keys after 60 days of inactivity for security. To prevent this, either use the key regularly or update any of its properties (name, description, expiration date).

### Table Not Found

```bash
Error: Table ID not provided
```

**Solutions:**

- Add `table_id` to config file
- Pass `--table-id` flag
- Verify table ID is correct

### Rate Limiting

```bash
Error: Rate limit exceeded. Retrying in 5s...
```

**Solutions:**

- Wait for automatic retry (exponential backoff: 1s, 2s, 4s, 8s)
- Reduce upload frequency
- Contact Roblox support for rate limit increase

Roblox Slang automatically handles rate limiting with exponential backoff and respects `Retry-After` headers from the API.

### Validation Errors

```bash
Error: Validation failed
```

**Solutions:**

- Fix reported validation errors
- Use `--skip-validation` to bypass (not recommended)
- Check translation file syntax

### IP Restriction Errors

```bash
Error: Insufficient permissions for this operation
```

**Solutions:**

- Check if your IP is in the allowed list (if IP restrictions are enabled)
- Update IP restrictions in Creator Dashboard
- Remove IP restrictions if not needed (except for Roblox place usage)

## API Reference

For detailed information about Roblox Cloud API:

- [Open Cloud API Reference](https://create.roblox.com/docs/cloud/reference) - Complete API documentation
- [Manage API Keys](https://create.roblox.com/docs/cloud/open-cloud/api-keys) - Official guide for creating and managing API keys
- [Localization API](https://create.roblox.com/docs/cloud/open-cloud/localization-api) - Localization Tables API endpoints
- [OAuth 2.0](https://create.roblox.com/docs/cloud/open-cloud/oauth2-overview) - Alternative authentication method

See also [Roblox Slang API Documentation](../reference/cloud-api.md) for implementation-specific details.

## Examples

See [examples/cloud-sync/](../../examples/cloud-sync/) for complete working examples.

## Support

- [GitHub Issues](https://github.com/mathtechstudio/roblox-slang/issues)
- [Roblox DevForum](https://devforum.roblox.com/)
- [Documentation](https://github.com/mathtechstudio/roblox-slang)
