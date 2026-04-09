# ZeroClaw P_ZEROCLAW Fork

> Custom fork with enhanced skills, self-healing system, and local Docker deployment

## What's This?

This is a customized fork of ZeroClaw with:

- **Self-Healer Skill** - Auto-repair system with health monitoring
- **Skills System** - Enhanced capabilities via SKILL.md/SKILL.toml
- **Docker-Ready** - Optimized for local development and deployment
- **Agentic-Bridge** - System prompt layer for better model behavior

## Quick Start

```bash
# Clone
git clone https://github.com/MasterXD123/zeroclaw-free.git
cd zeroclaw-free

# Configure
cp .env.example .env
# Edit .env with your API keys

# Start
docker compose up -d

# Access gateway
open http://localhost:42617
```

## What's Different in This Fork

| Component | Status | Notes |
|-----------|--------|-------|
| Self-Healer | ⚠️ Partial | SKILL.md/SKILL.toml exist, script pending |
| Agentic-Bridge | ⚠️ Partial | System prompt layer, needs testing |
| Onboarding Skill | ⚠️ Partial | Basic structure only |
| Features Skill | ⚠️ Partial | Basic structure only |
| Watchdog | ✅ Working | Health checks implemented |
| Code-Guardian | ✅ Working | Config protection |

## Skills Included

```
zeroclaw-data/workspace/skills/
├── self-healer/       # Auto-repair system
├── agentic-bridge/     # Model enhancement layer
├── onboarding/         # First-time setup
├── features/           # Capability listing
├── watchdog/           # Health monitoring
├── code-guardian/      # Config protection
├── setup-assistant/    # API configuration
└── auto-connect/       # Service auto-connection
```

## Configuration

### Environment Variables

```bash
# Required
OPENAI_API_KEY=sk-your-key-here

# Recommended
ZEROCLAW_MODEL=openrouter/google/gemma-4-27b-it

# Optional
TELEGRAM_BOT_TOKEN=your_bot_token
```

### Gateway

- Port: 42617
- Default binding: 127.0.0.1 (local only)

## Architecture

```
┌─────────────────────────────────────────────┐
│              ZeroClaw Runtime                 │
├─────────────────────────────────────────────┤
│                                               │
│  ┌──────────┐    ┌──────────┐              │
│  │  Agent   │───▶│  Tools   │              │
│  │  Loop    │    │ Registry │              │
│  └──────────┘    └──────────┘              │
│       │              │                        │
│       ▼              ▼                        │
│  ┌──────────┐    ┌──────────┐              │
│  │ Provider │    │  Skills  │              │
│  │ (LLM)   │    │  System  │              │
│  └──────────┘    └──────────┘              │
│                                               │
└─────────────────────────────────────────────┘
```

## Project Status

**⚠️ This fork is under development**

- Skills provide framework/interface but some lack full implementation
- Self-healer, onboarding, features need script implementation
- Agentic-bridge needs testing with different models

## Known Issues

1. Skills are documentation + tool definitions, not full implementations
2. Some skills overlap in functionality (watchdog + self-healer)
3. Scripts referenced in SKILL.md don't exist yet

## TODO

- [ ] Implement health-master.sh for self-healer
- [ ] Test agentic-bridge with Gemma 4
- [ ] Complete onboarding skill flow
- [ ] Add auto-tuner script
- [ ] Add predictive monitor script

## Docker Commands

```bash
# Build image
docker build -t zeroclaw:local .

# Start services
docker compose up -d

# View logs
docker logs zeroclaw-enterprise -f

# Restart
docker restart zeroclaw-enterprise
```

## License

MIT or Apache 2.0 (same as upstream)

---

*Based on [ZeroClaw](https://github.com/zeroclaw-labs/zeroclaw) by zeroclaw-labs*