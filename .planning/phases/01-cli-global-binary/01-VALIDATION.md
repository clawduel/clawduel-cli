---
phase: 1
slug: cli-global-binary
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-18
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Manual CLI verification (no test framework in project) |
| **Config file** | none |
| **Quick run command** | `npm run build && node dist/claw-cli.js help` |
| **Full suite command** | `npm run build && node dist/claw-cli.js help && node dist/claw-cli.js queue --help 2>&1 | grep -q timeout` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `npm run build && node dist/claw-cli.js help`
- **After every plan wave:** Run full suite command
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 1-01-01 | 01 | 1 | CLIP-01 | build | `grep '"bin"' package.json` | ✅ | ⬜ pending |
| 1-01-02 | 01 | 1 | CLIP-02 | cli | `npm run build && node dist/claw-cli.js help` | ❌ W0 | ⬜ pending |
| 1-01-03 | 01 | 1 | CLIP-03 | cli | `node dist/claw-cli.js help; echo $?` | ❌ W0 | ⬜ pending |
| 1-01-04 | 01 | 1 | CLIP-04 | cli | `node dist/claw-cli.js status 2>&1` | ❌ W0 | ⬜ pending |
| 1-02-01 | 02 | 1 | QUES-01 | cli | `grep -q 'timeout' dist/claw-cli.js` | ❌ W0 | ⬜ pending |
| 1-02-02 | 02 | 1 | QUES-02 | cli | `grep -q '3600' dist/claw-cli.js` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] TypeScript compilation of `claw-cli.ts` must succeed (`npm run build`)
- [ ] Compiled output must be executable via `node dist/claw-cli.js`

*Existing infrastructure covers build; no test framework to install.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `npm link` creates global binary | CLIP-02 | Requires system-level npm link | Run `npm link` then `which claw-cli` |
| All commands work via global binary | CLIP-04 | Requires live backend for most commands | Run `claw-cli status` against test backend |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
