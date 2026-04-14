#!/bin/sh
set -e

CONFIG_FILE="/zeroclaw-data/.zeroclaw/config.toml"

# Debug: print environment variables 
echo "=== Environment Debug ==="
if [ -n "$OPENROUTER_API_KEY" ]; then
  echo "OPENROUTER_API_KEY: set"
else
  echo "OPENROUTER_API_KEY: not set"
fi
echo "==========================="

# Get API key from OPENROUTER_API_KEY env var (zeroclaw reads this directly)
# Also store in config for backwards compatibility
API_KEY="${OPENROUTER_API_KEY:-}"

# Ensure OPENROUTER_API_KEY is exported for zeroclaw to find
if [ -n "$API_KEY" ]; then
    export OPENROUTER_API_KEY="$API_KEY"
fi

# Generate base config.toml
cat > "$CONFIG_FILE" << EOF
workspace_dir = "/zeroclaw-data/workspace"
default_provider = "openrouter"
default_model = "openrouter/free"
default_temperature = 0.7
api_key = "$API_KEY"

[gateway]
port = 42617
host = "[::]"
allow_public_bind = true
require_pairing = false

[autonomy]
level = "full"
workspace_only = false
block_high_risk_commands = false
require_approval_for_medium_risk = false
max_actions_per_hour = 10000
max_cost_per_day_cents = 1000000
allowed_commands = ["*"]
forbidden_paths = []

[skills]
open_skills_enabled = true
prompt_injection_mode = "full"

[scheduler]
enabled = true

[heartbeat]
enabled = true
interval_minutes = 180
target = "telegram"

[cron]
enabled = true

[memory]
backend = "sqlite"
auto_save = true
hygiene_enabled = true

[channels_config]
cli = true
EOF

# Add Telegram if configured
if [ -n "$TELEGRAM_BOT_TOKEN" ]; then
    USERS="${TELEGRAM_ALLOWED_USERS:-*}"
    echo "" >> "$CONFIG_FILE"
    echo "[channels_config.telegram]" >> "$CONFIG_FILE"
    echo "bot_token = \"$TELEGRAM_BOT_TOKEN\"" >> "$CONFIG_FILE"
    echo "allowed_users = [\"$USERS\"]" >> "$CONFIG_FILE"
fi

# Add Notion if configured
if [ -n "$NOTION_KEY" ]; then
    echo "" >> "$CONFIG_FILE"
    echo "[channels_config.notion]" >> "$CONFIG_FILE"
    echo "api_key = \"$NOTION_KEY\"" >> "$CONFIG_FILE"
fi

# Add GitHub if configured
if [ -n "$GITHUB_TOKEN" ]; then
    echo "" >> "$CONFIG_FILE"
    echo "[channels_config.github]" >> "$CONFIG_FILE"
    echo "token = \"$GITHUB_TOKEN\"" >> "$CONFIG_FILE"
fi

# Add Google Workspace if configured
if [ -n "$GOOGLE_REFRESH_TOKEN" ] && [ -n "$GOOGLE_CLIENT_ID" ] && [ -n "$GOOGLE_CLIENT_SECRET" ]; then
    echo "" >> "$CONFIG_FILE"
    echo "[google_workspace]" >> "$CONFIG_FILE"
    echo "enabled = true" >> "$CONFIG_FILE"
fi

echo "Config generated:"
echo "==================="
cat "$CONFIG_FILE"
echo "==================="

exec "$@"
