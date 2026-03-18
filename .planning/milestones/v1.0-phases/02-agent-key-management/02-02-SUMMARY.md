---
phase: 02-agent-key-management
plan: 02
subsystem: auth
tags: [ethers, keystore, multi-agent, cli]

# Dependency graph
requires:
  - phase: 02-agent-key-management
    provides: "init command writing keystores to ~/.clawduel/keystores/<address>.json"
provides:
  - "loadWallet with keystore discovery, selection, and --agent flag support"
  - "discoverKeystores() and selectKeystore() helper functions"
  - "CLAW_AGENT_ADDRESS env var support"
affects: [03-skill-documentation]

# Tech tracking
tech-stack:
  added: []
  patterns: ["keystore discovery with auto-select for single agent", "global --agent flag spliced from args before command dispatch"]

key-files:
  created: []
  modified: ["claw-cli.ts"]

key-decisions:
  - "Keystore discovery checks ~/.clawduel/keystores/ before legacy keyfile.json for backward compat"
  - "args.splice removes --agent from args to prevent command handler confusion"

patterns-established:
  - "Global flag parsing: parse and splice before command dispatch to avoid arg conflicts"
  - "Wallet loading priority: keystores dir > legacy keyfile > AGENT_PRIVATE_KEY env var"

requirements-completed: [KEYS-02, MAGT-02, MAGT-03]

# Metrics
duration: 2min
completed: 2026-03-18
---

# Phase 02 Plan 02: Wallet Loading with Multi-Agent Keystore Discovery Summary

**loadWallet refactored to discover keystores from ~/.clawduel/keystores/, auto-select single keystore, and support --agent/CLAW_AGENT_ADDRESS for multi-agent selection**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-18T18:20:33Z
- **Completed:** 2026-03-18T18:22:26Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments
- loadWallet now discovers keystores from ~/.clawduel/keystores/ directory first, falls back to legacy keyfile.json, then AGENT_PRIVATE_KEY env var
- Single keystore auto-selected without --agent flag; multiple keystores require explicit selection
- --agent flag parsed globally in main() and spliced from args before command dispatch
- Help text and header comments document --agent flag, CLAW_AGENT_ADDRESS, and Global Options section

## Task Commits

Each task was committed atomically:

1. **Task 1: Add keystore discovery/selection functions and refactor loadWallet** - `8280020` (feat)
2. **Task 2: Update help text for --agent flag and CLAW_AGENT_ADDRESS env var** - `39b2c93` (feat)

## Files Created/Modified
- `claw-cli.ts` - Added discoverKeystores(), selectKeystore(), refactored loadWallet(agentAddress?), --agent parsing in main(), updated help text

## Decisions Made
- Keystore discovery checks ~/.clawduel/keystores/ before legacy keyfile.json to prioritize new multi-agent path while maintaining backward compatibility
- args.splice(agentIdx, 2) removes --agent and its value from args before command handlers process them, preventing misinterpretation by getArg()

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Phase 02 (agent-key-management) is now complete with both init (02-01) and wallet loading (02-02) done
- Ready for Phase 03 (skill documentation) which documents these capabilities in SKILL.md

---
*Phase: 02-agent-key-management*
*Completed: 2026-03-18*
