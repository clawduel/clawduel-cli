---
phase: 02-agent-key-management
plan: 01
subsystem: auth
tags: [ethers, keystore, cli, non-interactive]

# Dependency graph
requires:
  - phase: 01-cli-global-binary
    provides: compiled claw-cli binary with cmdInit function
provides:
  - "--non-interactive mode for cmdInit"
  - "KEYSTORES_DIR constant for multi-agent keystore directory"
  - "Keystore files at ~/.clawduel/keystores/<address>.json"
affects: [02-agent-key-management plan 02, skill.md documentation]

# Tech tracking
tech-stack:
  added: []
  patterns: [non-interactive CLI flag pattern, multi-agent keystores directory]

key-files:
  created: []
  modified: [claw-cli.ts]

key-decisions:
  - "Keystore filename uses lowercase address with 0x prefix for human readability"
  - "Old KEYFILE_DIR/KEYFILE_PATH constants preserved for loadWallet compatibility (updated in plan 02)"

patterns-established:
  - "Non-interactive flag: args.includes('--non-interactive') pattern for CI/agent use"
  - "Keystore directory permissions: 0700 dir, 0600 files"

requirements-completed: [KEYS-01, MAGT-01]

# Metrics
duration: 1min
completed: 2026-03-18
---

# Phase 02 Plan 01: Non-Interactive Init and Keystores Directory Summary

**cmdInit --non-interactive mode reads AGENT_PRIVATE_KEY and CLAW_KEY_PASSWORD from env, writes encrypted keystore to ~/.clawduel/keystores/<address>.json with 0600 permissions**

## Performance

- **Duration:** 1 min
- **Started:** 2026-03-18T18:17:47Z
- **Completed:** 2026-03-18T18:19:01Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments
- cmdInit accepts --non-interactive flag to skip all TTY prompts
- Non-interactive mode validates both AGENT_PRIVATE_KEY and CLAW_KEY_PASSWORD env vars, exits 1 with clear error if missing
- Keystores written to ~/.clawduel/keystores/<address>.json with directory 0700 and file 0600 permissions
- Help text updated to document --non-interactive flag and keystores directory

## Task Commits

Each task was committed atomically:

1. **Task 1: Add KEYSTORES_DIR constant and refactor cmdInit** - `512525c` (feat)
2. **Task 2: Update help text for init command** - `39f5176` (feat)

## Files Created/Modified
- `claw-cli.ts` - Added --non-interactive mode to cmdInit, KEYSTORES_DIR constant, updated help text and header comment

## Decisions Made
- Keystore filename uses `tempWallet.address.toLowerCase()` which includes 0x prefix for human readability
- Old KEYFILE_DIR and KEYFILE_PATH constants preserved since loadWallet still references them (will be updated in plan 02)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- KEYSTORES_DIR constant available for plan 02 to use in loadWallet refactor
- cmdInit writing to new location; plan 02 will update loadWallet to read from keystores directory
- --agent flag can build on the args pattern established here

---
*Phase: 02-agent-key-management*
*Completed: 2026-03-18*
