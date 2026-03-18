---
phase: 3
slug: skill-document
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-18
---

# Phase 3 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Manual content verification + grep checks |
| **Config file** | none |
| **Quick run command** | `test -f skill.md && head -5 skill.md` |
| **Full suite command** | `grep -c "^#" skill.md && grep -q "clawduel-cli" skill.md && grep -q "CLAW_KEY_PASSWORD" skill.md && echo "PASS"` |
| **Estimated runtime** | ~1 second |

---

## Sampling Rate

- **After every task commit:** Run quick run command
- **After every plan wave:** Run full suite command
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 1 second

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 3-01-01 | 01 | 1 | SKIL-01 | grep | `head -10 skill.md \| grep -q "name:"` | ❌ W0 | ⬜ pending |
| 3-01-02 | 01 | 1 | SKIL-02 | grep | `grep -q "npm link" skill.md` | ❌ W0 | ⬜ pending |
| 3-01-03 | 01 | 1 | SKIL-03 | grep | `grep -q "clawduel-cli queue" skill.md && grep -q "clawduel-cli poll" skill.md && grep -q "clawduel-cli submit" skill.md` | ❌ W0 | ⬜ pending |
| 3-01-04 | 01 | 1 | SKIL-04 | grep | `grep -q "CLAW_BACKEND_URL" skill.md && grep -q "CLAW_RPC_URL" skill.md` | ❌ W0 | ⬜ pending |
| 3-01-05 | 01 | 1 | SKIL-05 | grep | `grep -q "number" skill.md && grep -q "boolean" skill.md` | ❌ W0 | ⬜ pending |
| 3-01-06 | 01 | 1 | SKIL-06 | grep | `grep -q "deadline" skill.md` | ❌ W0 | ⬜ pending |
| 3-01-07 | 01 | 1 | SKIL-07 | grep | `grep -q "strategy" skill.md \|\| grep -q "Strategy" skill.md` | ❌ W0 | ⬜ pending |
| 3-01-08 | 01 | 1 | KEYS-03 | grep | `grep -q "Wallet.encrypt" skill.md` | ❌ W0 | ⬜ pending |
| 3-01-09 | 01 | 1 | KEYS-04 | grep | `grep -q "AGENT_PRIVATE_KEY" skill.md` | ❌ W0 | ⬜ pending |
| 3-01-10 | 01 | 1 | KEYS-05 | grep | `grep -q "tradeoff\|trade-off\|security" skill.md` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `skill.md` file must exist at project root

*No test framework needed — documentation-only phase.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Agent can follow instructions end-to-end | All | Requires live agent test | Have a Claude Code agent read skill.md and attempt to compete |
| YAML frontmatter parses correctly | SKIL-01 | Requires YAML parser | `node -e "console.log(require('js-yaml').load(...))"` |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 1s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
