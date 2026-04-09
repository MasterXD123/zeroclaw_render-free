# TODO - ZeroClaw P_ZEROCLAW Fork

## Project Status

This fork is a work in progress. Many features are designed but not fully implemented.

## Skills Implementation

### ✅ Completed

| Skill | Status | Notes |
|-------|--------|-------|
| watchdog | Working | Health checks via docker exec |
| code-guardian | Working | Basic docker validation |

### ⚠️ Partial Implementation

| Skill | Status | Notes |
|-------|--------|-------|
| self-healer | 30% | SKILL.md + SKILL.toml exist, scripts missing |
| agentic-bridge | 20% | System prompt layer, minimal tool |
| onboarding | 30% | SKILL files exist, no execution logic |
| features | 30% | SKILL files exist, no execution logic |
| setup-assistant | 40% | Basic structure, overlaps with auto-connect |
| auto-connect | 40% | Basic structure, overlaps with setup-assistant |

### ❌ Not Started

- health-master.sh script
- auto-tune.sh script
- predict.sh script

## Implementation Tasks

### Critical (Must Have)

- [ ] **health-master.sh** - Master health check script for self-healer
  - Health check: gateway, qdrant, memory, disk
  - Auto-heal: restart services if down
  - Anti-loop: wait 10 min between heals
  - State persistence: track restart_count, last_check

- [ ] **self-healer integration** - Connect SKILL.toml to the script
  - Currently SKILL.toml references `health-master.sh` which doesn't exist
  - Need to create the script or remove the reference

### Important (Should Have)

- [ ] **agentic-bridge enhancement** - Make it actually improve model behavior
  - Current: just `echo 'agentic-bridge active'`
  - Needed: actual system prompt injection

- [ ] **onboarding flow** - Make onboarding actually run
  - Detect first-time users
  - Guide through setup steps
  - Store configuration

- [ ] **deduplicate skills** - Remove overlap
  - auto-connect and setup-assistant do the same thing
  - watchdog and self-healer overlap
  - Consolidate into unified health system

### Nice to Have

- [ ] **auto-tune.sh** - Resource optimization script
- [ ] **predict.sh** - Predictive monitoring script
- [ ] **Telegram integration** - Better Telegram support
- [ ] **Dashboard** - Web UI for monitoring

## Technical Debt

### Security

- [ ] OTP uses 6 digits (should be 8+)
- [ ] Gateway API allows disabling security
- [ ] No skill signing/verification

### Architecture

- [ ] Skills are documentation-only, no real implementation
- [ ] Docker-specific commands in skills (not runtime-agnostic)
- [ ] No coordination between overlapping skills

### Configuration

- [ ] Multiple config files (.env.example, config.toml.example)
- [ ] Some config options hardcoded

## Testing Needed

- [ ] Test agentic-bridge with Gemma 4
- [ ] Test self-healer with actual failures
- [ ] Test watchdog health checks
- [ ] Test onboarding flow

## Documentation Gaps

- [ ] No API documentation
- [ ] No deployment guide
- [ ] No troubleshooting guide
- [ ] Skills not documented for users

## Ideas for Improvement

### High Priority

1. **Implement health-master.sh**
   - One script that does everything
   - No external dependencies beyond curl/docker
   - Simple state file for persistence

2. **Unified Health System**
   - Merge watchdog + self-healer
   - Single health check job
   - Coordinated restart logic

3. **Better Skills Documentation**
   - Clear SKILL.md for each skill
   - Usage examples
   - Expected behavior

### Medium Priority

4. **Model-Agnostic Bridge**
   - Test with multiple models
   - Document which models work best
   - Provide fallbacks

5. **Production-Ready Config**
   - Proper secrets management
   - Environment-specific configs
   - Health check endpoints

### Future Ideas

6. **Multi-Agent Support**
   - Sub-agents for different tasks
   - Agent coordination

7. **Web Dashboard**
   - Real-time metrics
   - Skill management UI
   - Log viewer

8. **Plugin System**
   - Formal plugin interface
   - Community plugin registry

---

*Last updated: 2026-04-09*
