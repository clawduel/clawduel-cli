---
phase: 2
slug: agent-key-management
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-18
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Manual CLI verification (no test framework in project) |
| **Config file** | none |
| **Quick run command** | `npm run build && node dist/clawduel-cli.js help` |
| **Full suite command** | `npm run build && AGENT_PRIVATE_KEY=0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef CLAW_KEY_PASSWORD=test123 node dist/clawduel-cli.js init --non-interactive 2>&1` |
| **Estimated runtime** | ~8 seconds (includes scrypt KDF) |

---

## Sampling Rate

- **After every task commit:** Run `npm run build && node dist/clawduel-cli.js help`
- **After every plan wave:** Run full suite command
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 8 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 2-01-01 | 01 | 1 | MAGT-01 | cli | `grep -q "keystores" dist/clawduel-cli.js` | ❌ W0 | ⬜ pending |
| 2-01-02 | 01 | 1 | KEYS-01 | cli | `npm run build && AGENT_PRIVATE_KEY=0x... CLAW_KEY_PASSWORD=test node dist/clawduel-cli.js init --non-interactive` | ❌ W0 | ⬜ pending |
| 2-01-03 | 01 | 1 | KEYS-02 | cli | `grep -q "CLAW_KEY_PASSWORD" dist/clawduel-cli.js` | ❌ W0 | ⬜ pending |
| 2-01-04 | 01 | 1 | MAGT-02 | cli | `grep -q "agent" dist/clawduel-cli.js && grep -q "CLAW_AGENT_ADDRESS" dist/clawduel-cli.js` | ❌ W0 | ⬜ pending |
| 2-01-05 | 01 | 1 | MAGT-03 | cli | `grep -q "auto-select" dist/clawduel-cli.js || grep -q "single keystore" dist/clawduel-cli.js` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] TypeScript compilation with new keystore code must succeed (`npm run build`)
- [ ] `~/.clawduel/keystores/` directory creation logic in compiled output

*Existing infrastructure covers build; no test framework to install.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Keystore file permissions 0600 | KEYS-01 | Requires filesystem inspection | `stat -f "%Lp" ~/.clawduel/keystores/*.json` should show 600 |
| Multi-agent concurrent operation | MAGT-02 | Requires two separate processes | Run two CLI instances with different --agent flags |
| Legacy keyfile fallback | v2 | Not in v1 scope | Verify `~/.clawduel/claw-keyfile.json` still loads if no keystores/ dir |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 8s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
