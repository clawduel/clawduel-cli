---
phase: 01-cli-global-binary
plan: 01
subsystem: cli
tags: [typescript, npm-bin, global-cli, shebang, timeout]

# Dependency graph
requires: []
provides:
  - Compilable CLI binary at dist/claw-cli.js with node shebang
  - npm bin entry for global "claw-cli" command
  - Queue --timeout flag with 3600s default
affects: [02-key-management, 03-skill-documentation]

# Tech tracking
tech-stack:
  added: []
  patterns: [npm-bin-global-cli, rootDir-project-root, optArg-optional-flag-parsing]

key-files:
  created: [dist/claw-cli.js]
  modified: [tsconfig.json, package.json, claw-cli.ts]

key-decisions:
  - "rootDir changed from ./src to . to include claw-cli.ts in compilation"
  - "SDK entry points moved to dist/src/ path due to rootDir change"
  - "Added prepare script for npm link build automation"

patterns-established:
  - "optArg pattern: scoped optional arg parser for switch cases with optional flags"
  - "bin entry: dist/claw-cli.js as global binary target"

requirements-completed: [CLIP-01, CLIP-02, CLIP-03, CLIP-04, QUES-01, QUES-02]

# Metrics
duration: 2min
completed: 2026-03-18
---

# Phase 01 Plan 01: CLI Global Binary Summary

**Compiled CLI binary with node shebang, npm bin entry for global install, and queue --timeout flag with 3600s default**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-18T17:49:40Z
- **Completed:** 2026-03-18T17:51:23Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- TypeScript compilation now includes claw-cli.ts, outputting dist/claw-cli.js with #!/usr/bin/env node shebang
- package.json bin field enables `npm link` for global `claw-cli` command
- Queue command accepts `--timeout <seconds>` flag with 3600s default for attestation deadline control
- All help text and error messages reference `claw-cli` instead of `npx tsx claw-cli.ts`

## Task Commits

Each task was committed atomically:

1. **Task 1: Configure TypeScript compilation and npm bin entry** - `6725310` (feat)
2. **Task 2: Add --timeout flag to queue command** - `9b40964` (feat)

## Files Created/Modified
- `tsconfig.json` - rootDir changed to ".", include array expanded with claw-cli.ts
- `package.json` - bin field, updated main/types paths, prepare script added
- `claw-cli.ts` - Node shebang, all help text updated, timeout parameter added to cmdQueue

## Decisions Made
- rootDir changed from ./src to . to include claw-cli.ts in compilation; SDK output paths adjusted to dist/src/
- Added prepare script alongside existing prepublishOnly to ensure build runs before npm link
- Used scoped optArg function in queue case block (matching existing matches case pattern)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Found additional npx tsx reference at line 276**
- **Found during:** Task 1 (help text updates)
- **Issue:** Plan listed 3 npx tsx references to replace but a 4th existed at line 276 in wallet loading error message
- **Fix:** Replaced `npx tsx claw-cli.ts init` with `claw-cli init`
- **Files modified:** claw-cli.ts
- **Verification:** `grep -c "npx tsx" claw-cli.ts` returns 0
- **Committed in:** 6725310 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Minor scope addition to ensure zero npx tsx references remain. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- CLI binary compiles and runs via `node dist/claw-cli.js`
- Ready for `npm link` to install globally
- Foundation set for Phase 02 (key management) and Phase 03 (skill documentation)

## Self-Check: PASSED

All artifacts verified:
- dist/claw-cli.js exists
- dist/src/index.js exists
- 01-01-SUMMARY.md exists
- Commit 6725310 exists
- Commit 9b40964 exists

---
*Phase: 01-cli-global-binary*
*Completed: 2026-03-18*
