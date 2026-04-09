# ZeroClaw P_ZEROCLAW Fork

> Enhanced autonomous AI agent runtime with self-healing, skills system, and Docker deployment

**Repository:** https://github.com/MasterXD123/zeroclaw-free

---

## Table of Contents

1. [Introduction](#introduction)
2. [Features](#features)
3. [Prerequisites](#prerequisites)
4. [Installation](#installation)
5. [Configuration](#configuration)
6. [Usage](#usage)
7. [Skills System](#skills-system)
8. [Docker Reference](#docker-reference)
9. [Architecture](#architecture)
10. [Security](#security)
11. [Troubleshooting](#troubleshooting)
12. [FAQ](#faq)
13. [Project Status](#project-status)

---

## Introduction

ZeroClaw is a Rust-based autonomous AI agent runtime. This fork adds:

- **Self-Healer** - Automatic detection and repair of service issues
- **Skills System** - Extensible capability system via `SKILL.md`/`SKILL.toml`
- **Agentic-Bridge** - System prompt layer to enhance less-capable models
- **Docker-Ready** - Optimized for local development and deployment

### What is an AI Agent Runtime?

An agent runtime is software that allows an AI model to:
- Execute actions (not just respond to questions)
- Use tools (shell commands, file operations, API calls)
- Remember context across conversations
- Run autonomously on a schedule (cron jobs)

ZeroClaw provides the infrastructure; you provide the AI model via OpenRouter.

---

## Features

| Feature | Status | Description |
|---------|--------|-------------|
| Autonomous Agent Loop | ✅ | Rust-powered agent orchestration |
| Skills System | ⚠️ Partial | Framework complete, some scripts pending |
| Self-Healer | ⚠️ Partial | Health checks work, auto-repair pending |
| Agentic-Bridge | ⚠️ Partial | Prompt enhancement layer |
| Watchdog | ✅ Working | Health monitoring via docker exec |
| Code-Guardian | ✅ Working | Configuration protection |
| Telegram Integration | ✅ | Bot-based interaction |
| Web Gateway | ✅ | Browser-based UI at port 42617 |
| Cron Scheduler | ✅ | Periodic task execution |
| Memory System | ✅ | SQLite-backed persistent memory |

---

## Prerequisites

### Required Software

| Software | Version | Install Guide |
|----------|---------|--------------|
| Docker | 20.10+ | [docs.docker.com/get-docker](https://docs.docker.com/get-docker/) |
| Docker Compose | 2.0+ | Included with Docker Desktop |
| Git | Any recent | [git-scm.com/downloads](https://git-scm.com/downloads) |

### Required Accounts

| Service | Required | Notes |
|---------|----------|-------|
| OpenRouter | Yes | Free tier available at [openrouter.ai](https://openrouter.ai/) |
| Telegram Bot | No | Optional for bot-based interaction |

### System Requirements

- **OS:** Linux, macOS, or Windows with WSL2
- **RAM:** 4GB minimum (8GB recommended)
- **Disk:** 2GB free space
- **Network:** Internet access for API calls

---

## Installation

### Step 1: Clone the Repository

```bash
git clone https://github.com/MasterXD123/zeroclaw-free.git
cd zeroclaw-free
```

### Step 2: Get Your API Key

1. Go to [openrouter.ai](https://openrouter.ai/)
2. Sign up for a free account
3. Navigate to **Keys** section
4. Create a new API key
5. Copy the key (starts with `sk-or-v1-`)

### Step 3: Configure Environment

Create your environment file:

```bash
# On Linux/macOS/WSL2
cp .env.example .env

# On Windows (PowerShell)
copy .env.example .env
```

Edit `.env` with your text editor:

```bash
# REQUIRED: Your OpenRouter API key
OPENAI_API_KEY=sk-or-v1-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx

# Model selection
# Recommended models for agentic behavior:
#   - openrouter/google/gemma-4-27b-it (best for instructions)
#   - openrouter/anthropic/claude-3-haiku (fast, capable)
#   - openrouter/mistral/mistral-7b-instruct (balanced)
ZEROCLAW_MODEL=openrouter/google/gemma-4-27b-it

# Security: keep false to restrict to localhost
ZEROCLAW_ALLOW_PUBLIC_BIND=false

# Gateway port
HOST_PORT=42617

# Optional: Telegram bot token (see Telegram Setup section)
# TELEGRAM_BOT_TOKEN=
```

### Step 4: Start Services

Build and start the Docker containers:

```bash
docker compose up -d
```

This will:
1. Build the ZeroClaw Docker image
2. Start the zeroclaw-enterprise container
3. Start dependent services (if any)

### Step 5: Verify Installation

Check the container status:

```bash
docker compose ps
```

You should see:

```
NAME                STATUS          PORTS
zeroclaw-enterprise running         0.0.0.0:42617->42617/tcp
```

View logs to confirm startup:

```bash
docker compose logs -f
```

Look for messages indicating successful startup, then press `Ctrl+C` to exit logs.

---

## Configuration

### Environment Variables (.env)

The `.env` file is the primary configuration method. Each variable:

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `OPENAI_API_KEY` | **Yes** | - | Your OpenRouter API key |
| `PROVIDER` | No | openrouter | Model provider |
| `ZEROCLAW_MODEL` | No | gemma-4-27b-it | Model identifier |
| `ZEROCLAW_ALLOW_PUBLIC_BIND` | No | false | Allow public gateway access |
| `HOST_PORT` | No | 42617 | Host port for gateway |
| `QDRANT_HOST` | No | qdrant | Vector database host |
| `QDRANT_PORT` | No | 6333 | Vector database port |
| `GITHUB_TOKEN` | No | - | GitHub API token |
| `NOTION_KEY` | No | - | Notion integration key |
| `TELEGRAM_BOT_TOKEN` | No | - | Telegram bot token |
| `RUST_LOG` | No | info | Logging level (error, warn, info, debug) |
| `RUST_BACKTRACE` | No | 1 | Enable backtraces (0 or 1) |

### Advanced Configuration (config.toml)

For advanced settings, create `config.toml` from the example:

```bash
cp config.toml.example config.toml
```

Key configuration sections:

#### Autonomy Level

Controls what the agent can do autonomously:

```toml
[autonomy]
# Options:
#   - "read_only": Agent can only read, no writes
#   - "supervised": Agent asks for approval on risky actions
#   - "full": Agent acts autonomously
level = "supervised"
workspace_only = true
```

#### Gateway Settings

```toml
[gateway]
port = 42617
host = "127.0.0.0"  # Change to "0.0.0.0" for public access
require_pairing = true
allow_public_bind = false
```

#### Security Settings

```toml
[security.sandbox]
backend = "auto"

[security.resources]
max_memory_mb = 512
max_cpu_time_seconds = 60
max_subprocesses = 10

[security.otp]
enabled = false
method = "totp"
```

#### Allowed Commands

```toml
[autonomy]
allowed_commands = [
    "git", "npm", "cargo", "ls", "cat", "grep",
    "find", "echo", "pwd", "curl", "wget"
]
```

### SKILL.toml Per-Skill Configuration

Each skill has its own `SKILL.toml` that defines its tools and behavior. See [Skills System](#skills-system) below.

---

## Usage

### Web Gateway (Browser)

Access the gateway at: **http://localhost:42617**

Features:
- Chat interface
- Conversation history
- Skill activation
- Configuration panel

### Telegram Bot

If configured, interact with your bot on Telegram:

1. Open Telegram and search for your bot
2. Send `/start` to begin
3. Send commands or questions

Available commands:
- `/start` - Start interaction
- `/help` - Show help
- `/status` - Show system status
- `/skills` - List available skills

### API Access

The gateway exposes a REST API:

```bash
# Health check
curl http://localhost:42617/health

# Send message
curl -X POST http://localhost:42617/api/message \
  -H "Content-Type: application/json" \
  -d '{"message": "hello"}'
```

---

## Skills System

The skills system extends ZeroClaw with custom capabilities.

### What is a Skill?

A skill is a package containing:
- `SKILL.md` - Documentation and behavior definition
- `SKILL.toml` - Tool definitions and configuration
- Optional scripts/commands

### Included Skills

#### ✅ Watchdog (Working)

**Location:** `zeroclaw-data/workspace/skills/watchdog/`

Health monitoring skill that checks service status.

**Tools:**
- `check_gateway` - Verify gateway is responding
- `check_qdrant` - Verify vector database is running
- `docker_stats` - Get container resource usage

**Usage:**
```
@watchdog check gateway status
```

#### ✅ Code-Guardian (Working)

**Location:** `zeroclaw-data/workspace/skills/code-guardian/`

Protects configuration files from accidental corruption.

**Tools:**
- `validate_config_toml` - Validate config syntax
- `backup_config` - Create config backup
- `restore_config` - Restore from backup

**Usage:**
```
@code-guardian validate my config
```

#### ⚠️ Self-Healer (Partial)

**Location:** `zeroclaw-data/workspace/skills/self-healer/`

Auto-repair system for critical issues.

**Status:** SKILL.md and SKILL.toml exist; execution scripts pending.

**Planned Tools:**
- `health-check` - Comprehensive health diagnosis
- `auto-repair` - Fix detected issues
- `restart_service` - Restart failed containers

**Usage (when implemented):**
```
@self-healer run health check
```

#### ⚠️ Agentic-Bridge (Partial)

**Location:** `zeroclaw-data/workspace/skills/agentic-bridge/`

Enhances less-capable models with better system prompts.

**Status:** Basic structure exists; testing pending.

**Purpose:** Improves model behavior for models that don't natively follow instructions well.

#### ⚠️ Onboarding (Partial)

**Location:** `zeroclaw-data/workspace/skills/onboarding/`

First-time setup and configuration assistant.

**Status:** Framework exists; execution logic pending.

#### ⚠️ Features (Partial)

**Location:** `zeroclaw-data/workspace/skills/features/`

Lists available capabilities.

**Status:** Basic structure only.

#### ⚠️ Setup-Assistant (Partial)

**Location:** `zeroclaw-data/workspace/skills/setup-assistant/`

API configuration helper.

**Status:** Basic structure; overlaps with onboarding.

#### ⚠️ Auto-Connect (Partial)

**Location:** `zeroclaw-data/workspace/skills/auto-connect/`

Automatic service connection and discovery.

**Status:** Basic structure; overlaps with setup-assistant.

### Creating Custom Skills

Create a new skill directory:

```
zeroclaw-data/workspace/skills/my-skill/
├── SKILL.md      # Documentation
└── SKILL.toml    # Tool definitions
```

Example `SKILL.toml`:

```toml
name = "my-skill"
version = "1.0.0"
description = "My custom skill"

[[tools]]
name = "my_tool"
description = "What this tool does"
kind = "shell"
command = "echo 'Hello from my tool'"
```

---

## Docker Reference

### Container Management

```bash
# Start services
docker compose up -d

# Stop services
docker compose down

# Restart a container
docker compose restart zeroclaw-enterprise

# View logs
docker compose logs -f zeroclaw-enterprise

# Rebuild and start
docker compose up -d --build
```

### Container Shell Access

```bash
# Enter container shell
docker exec -it zeroclaw-enterprise sh

# Run a command inside container
docker exec zeroclaw-enterprise zeroclaw status
```

### Resource Monitoring

```bash
# View container resource usage
docker stats

# View detailed container info
docker inspect zeroclaw-enterprise

# View container disk usage
docker system df
```

### Cleanup

```bash
# Remove stopped containers
docker compose rm

# Remove all volumes (WARNING: deletes data)
docker compose down -v

# Full cleanup (removes containers, volumes, images)
docker compose down -v --rmi local
```

### Volume Mounts

Data persists in these volumes:

| Volume | Purpose |
|--------|---------|
| zeroclaw_zeroclaw-data | Skills, workspace, runtime data |
| zeroclaw_zeroclaw-config | Configuration files |

### Ports

| Port | Service |
|------|---------|
| 42617 | ZeroClaw Gateway (HTTP) |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      ZeroClaw Runtime                          │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────┐         ┌──────────────┐                │
│  │    Agent     │────────▶│    Tools      │                │
│  │    Loop      │         │   Registry    │                │
│  └──────────────┘         └──────────────┘                │
│         │                         │                          │
│         ▼                         ▼                          │
│  ┌──────────────┐         ┌──────────────┐                │
│  │   Provider    │         │    Skills     │                │
│  │     (LLM)     │         │    System     │                │
│  └──────────────┘         └──────────────┘                │
│                                                               │
│  ┌──────────────┐         ┌──────────────┐                │
│  │    Cron      │         │   Channels    │                │
│  │  Scheduler   │         │  Telegram/HTTP│                │
│  └──────────────┘         └──────────────┘                │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

### Core Components

| Component | Description |
|-----------|-------------|
| Agent Loop | Orchestrates message processing and tool execution |
| Provider | Manages LLM API calls (OpenRouter) |
| Tools Registry | Registers and executes available tools |
| Skills System | Loads and manages skill definitions |
| Cron Scheduler | Executes tasks on schedule |
| Channels | Handles external interfaces (Telegram, web) |

### Data Flow

```
User Message → Channel → Gateway → Agent Loop → Provider (LLM)
                                                      │
                                              ┌───────┴───────┐
                                              │               │
                                         Tool Execution   Response
                                              │               │
                                              ▼               ▼
                                        Skills/Cron    User Output
```

---

## Security

### Security Best Practices

1. **Never commit `.env`** - Contains API keys
   - The file is in `.gitignore` by default
   - Use `.env.example` as template

2. **Restrict Gateway Access**
   ```toml
   [gateway]
   allow_public_bind = false  # Keep restricted to localhost
   ```

3. **Use Supervised Autonomy**
   ```toml
   [autonomy]
   level = "supervised"  # Agent asks before risky actions
   ```

4. **Limit Allowed Commands**
   ```toml
   [autonomy]
   allowed_commands = ["git", "ls", "cat"]  # Minimal set
   ```

5. **Enable OTP for Critical Actions**
   ```toml
   [security.otp]
   enabled = true
   method = "totp"
   ```

### Files to Never Commit

- `.env` - API keys and secrets
- `config.toml` - Contains encrypted secrets
- `zeroclaw-data/.zeroclaw/` - Runtime secrets
- `*.key`, `*.pem` - SSL certificates
- `credentials.json` - API credentials

### Security Audit

The codebase includes a security audit system that:
- Blocks dangerous shell operators (`||`, `&&`, `;`)
- Blocks script-like files (`.sh` files in skills)
- Validates skill commands before execution

---

## Troubleshooting

### Container Won't Start

**Symptom:** `docker compose up -d` fails

**Solutions:**

1. Check if port is already in use:
   ```bash
   netstat -an | grep 42617
   # or on Windows
   Get-NetTCPConnection -LocalPort 42617
   ```

2. Check Docker is running:
   ```bash
   docker version
   ```

3. View detailed logs:
   ```bash
   docker compose up  # Without -d to see output
   ```

### API Key Errors

**Symptom:** `Unauthorized` or `Invalid API key`

**Solutions:**

1. Verify your API key in `.env`:
   ```bash
   grep OPENAI_API_KEY .env
   ```

2. Check OpenRouter key is valid:
   - Go to [openrouter.ai/keys](https://openrouter.ai/keys)
   - Verify key hasn't expired

3. Check key has sufficient credits:
   - OpenRouter free tier has limits

### Gateway Not Accessible

**Symptom:** `http://localhost:42617` shows connection refused

**Solutions:**

1. Verify container is running:
   ```bash
   docker compose ps
   ```

2. Check logs for startup errors:
   ```bash
   docker compose logs zeroclaw-enterprise
   ```

3. Verify port mapping:
   ```bash
   docker port zeroclaw-enterprise
   ```

### Model Not Responding

**Symptom:** Gateway works but model doesn't respond

**Solutions:**

1. Check model name is correct:
   ```bash
   grep ZEROCLAW_MODEL .env
   ```

2. Verify model is available on OpenRouter:
   - Check [openrouter.ai/models](https://openrouter.ai/models)

3. Try a different model:
   ```bash
   # Edit .env
   ZEROCLAW_MODEL=openrouter/anthropic/claude-3-haiku
   # Restart
   docker compose restart
   ```

### Telegram Bot Not Working

**Symptom:** Bot doesn't respond to messages

**Solutions:**

1. Verify bot token:
   ```bash
   grep TELEGRAM_BOT_TOKEN .env
   ```

2. Test bot token:
   ```bash
   curl https://api.telegram.org/bot${TELEGRAM_BOT_TOKEN}/getMe
   ```

3. Check webhook status:
   ```bash
   curl https://api.telegram.org/bot${TELEGRAM_BOT_TOKEN}/getWebhookInfo
   ```

4. Restart with new token if needed:
   ```bash
   docker compose restart
   ```

### High Memory Usage

**Symptom:** Container using excessive memory

**Solutions:**

1. Check resource limits in `config.toml`:
   ```toml
   [security.resources]
   max_memory_mb = 512
   ```

2. Reduce context window:
   ```toml
   [agent]
   max_history_messages = 20  # Reduced from 50
   ```

3. Clean old logs:
   ```bash
   docker compose exec zeroclaw-enterprise truncate -s 0 /var/log/*.log
   ```

### Skills Not Loading

**Symptom:** Skills exist but don't appear available

**Solutions:**

1. Check skill directory structure:
   ```
   zeroclaw-data/workspace/skills/
   └── skill-name/
       ├── SKILL.md
       └── SKILL.toml
   ```

2. Validate SKILL.toml syntax:
   ```bash
   # Check for common errors in TOML
   docker compose exec zeroclaw-enterprise zeroclaw skill validate
   ```

3. Check logs for skill errors:
   ```bash
   docker compose logs | grep -i skill
   ```

---

## FAQ

### Q: Do I need a powerful computer?

**A:** No. ZeroClaw runs in Docker with minimal resource usage. A basic laptop or miniPC is sufficient. The actual AI model runs on OpenRouter's servers.

### Q: How much does it cost to run?

**A:** ZeroClaw itself is free. You pay for:
- OpenRouter API calls (free tier: ~$5/month)
- Your own compute for Docker (minimal)

### Q: Can I use my own API key?

**A:** Yes. ZeroClaw supports OpenRouter. Get a key at [openrouter.ai](https://openrouter.ai/).

### Q: What models work best?

**A:** Models tagged as "instruct" or "agentic" work best:
- `google/gemma-4-27b-it` (recommended)
- `anthropic/claude-3-haiku`
- `mistral/mistral-7b-instruct`

Avoid basic chat models that don't follow instructions well.

### Q: Can I run without Docker?

**A:** Yes, but requires Rust toolchain. See [DOCKER-BUILD.md](./DOCKER-BUILD.md) for native installation.

### Q: How do I update ZeroClaw?

**A:**

```bash
git pull origin master
docker compose up -d --build
```

### Q: Where does data persist?

**A:** In Docker volumes:
- `zeroclaw_zeroclaw-data` - Skills, workspace, memory
- `zeroclaw_zeroclaw-config` - Configuration

### Q: Can I add custom tools?

**A:** Yes. Create a skill with SKILL.md and SKILL.toml. See [Skills System](#skills-system).

### Q: Why is my model responding poorly?

**A:** Try these solutions:
1. Use a better model (gemma-4-27b-it recommended)
2. Enable Agentic-Bridge skill for prompt enhancement
3. Adjust temperature in config.toml:
   ```toml
   default_temperature = 0.7
   ```

### Q: How do I reset everything?

**A:**

```bash
# Stop and remove containers
docker compose down

# Remove volumes (deletes all data)
docker compose down -v

# Fresh start
docker compose up -d
```

---

## Project Status

### Implementation Status

| Component | Status | Notes |
|-----------|--------|-------|
| Core Agent Loop | ✅ Complete | Working |
| Docker Deployment | ✅ Complete | Working |
| Skills Framework | ⚠️ Partial | Some skills need scripts |
| Self-Healer | ⚠️ Partial | 30% implementation |
| Agentic-Bridge | ⚠️ Partial | 20% implementation |
| Onboarding | ⚠️ Partial | 30% implementation |
| Watchdog | ✅ Complete | Working |
| Code-Guardian | ✅ Complete | Working |

### Known Issues

1. ⚠️ Skills are framework-only, some lack execution scripts
2. ⚠️ Self-healer auto-repair not yet implemented
3. ⚠️ Agentic-bridge needs testing with production models
4. ⚠️ Some skills overlap in functionality

### TODO

- [ ] Implement health-master.sh for self-healer
- [ ] Test agentic-bridge with Gemma 4
- [ ] Complete onboarding skill flow
- [ ] Add auto-tuner script
- [ ] Add predictive monitor script
- [ ] Create comprehensive API documentation

---

## License

MIT or Apache 2.0 (same as upstream ZeroClaw)

---

## Contributing

Contributions welcome! Please:

1. Read the existing codebase structure
2. Follow Rust coding conventions
3. Test changes before submitting
4. Update documentation for any new features

---

## Support

- **Issues:** https://github.com/MasterXD123/zeroclaw-free/issues
- **Discussion:** Open an issue for questions

---

*Based on [ZeroClaw](https://github.com/zeroclaw-labs/zeroclaw) by zeroclaw-labs*
