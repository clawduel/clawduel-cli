---
status: testing
phase: full-project
source: 01-01-SUMMARY.md, 02-01-SUMMARY.md, 02-02-SUMMARY.md, 03-01-SUMMARY.md
started: 2026-03-18T18:50:00Z
updated: 2026-03-18T18:50:00Z
---

## Current Test

number: 1
name: Build and Global Binary
expected: |
  Run `npm run build`. It should succeed with no errors.
  `dist/claw-cli.js` should exist and start with `#!/usr/bin/env node`.
  `node dist/claw-cli.js help` should print usage and exit 0.
awaiting: user response

## Tests

### 1. Build and Global Binary
expected: Run `npm run build`. It should succeed. `dist/claw-cli.js` exists with node shebang. `node dist/claw-cli.js help` prints usage and exits 0.
result: [pending]

### 2. npm link Global Install
expected: Run `npm link`. Then `claw-cli help` works from any directory, prints usage, exits 0. `which claw-cli` returns a path.
result: [pending]

### 3. Help Text References claw-cli
expected: `claw-cli help` output says "claw-cli" not "npx tsx claw-cli.ts". No stale references.
result: [pending]

### 4. Queue --timeout Flag
expected: `claw-cli queue --help` or help text mentions --timeout. Running `claw-cli queue --bet-tier 100 --timeout 1800` should attempt to queue (may fail without backend, but should parse the flag without error before any API call).
result: [pending]

### 5. Init --non-interactive (Missing Env Vars)
expected: Run `claw-cli init --non-interactive` WITHOUT setting AGENT_PRIVATE_KEY or CLAW_KEY_PASSWORD. Should exit 1 with a clear error message about missing env vars (not crash or prompt for input).
result: [pending]

### 6. Init --non-interactive (With Env Vars)
expected: Run `AGENT_PRIVATE_KEY=0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef CLAW_KEY_PASSWORD=testpass123 claw-cli init --non-interactive`. Should create a keystore file at `~/.clawduel/keystores/<address>.json` with 0600 permissions. No prompts.
result: [pending]

### 7. Keystore Auto-Select (Single)
expected: With one keystore in `~/.clawduel/keystores/`, any command that loads wallet (e.g., `claw-cli status`) should auto-select it without requiring `--agent`. Should attempt to use the keystore (may fail at API call, but should not error about keystore selection).
result: [pending]

### 8. --agent Flag Parsing
expected: `claw-cli --agent 0xSomeAddress status` should not crash. The `--agent` flag should be parsed and removed before the status command runs. (May fail at wallet loading if address doesn't match a keystore, but the flag parsing should work.)
result: [pending]

### 9. skill.md YAML Frontmatter
expected: `head -15 skill.md` shows valid YAML frontmatter with `name: clawduel`, `description:`, and a `metadata:` block containing `version:` and `homepage:`.
result: [pending]

### 10. skill.md Bootstrap Instructions
expected: skill.md contains bootstrap section with `git clone`, `npm install`, `npm link`, and a fallback for permission errors.
result: [pending]

### 11. skill.md Key Management Docs
expected: skill.md documents both key paths: (A) `claw-cli init --non-interactive` with `Wallet.encrypt()` and (B) `AGENT_PRIVATE_KEY` env var. Includes a security tradeoff comparison.
result: [pending]

### 12. skill.md Fight Loop
expected: skill.md has a numbered fight loop with exact `claw-cli queue`, `claw-cli poll`, `claw-cli submit` commands. Each step references the specific CLI command with flags.
result: [pending]

### 13. skill.md Env Vars and Prediction Types
expected: skill.md documents all env vars (CLAW_KEY_PASSWORD, CLAW_BACKEND_URL, CLAW_RPC_URL, etc.) with defaults. Documents prediction types (number, boolean, string, text) with format rules. Documents deadline behavior (absolute, no revisions, no-submit = loss).
result: [pending]

## Summary

total: 13
passed: 0
issues: 0
pending: 13
skipped: 0

## Gaps

[none yet]
