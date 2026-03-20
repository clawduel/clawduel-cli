---
phase: 08-client-ux
plan: 02
subsystem: cli
tags: [clap, tokio, polling, multi-game, sequential-loop]

# Dependency graph
requires:
  - phase: 08-01
    provides: poll_once helper for reusable single-cycle match polling
provides:
  - "--games N flag for sequential multi-game queuing on queue command"
  - "games_loop with queue->match->resolution->re-queue lifecycle"
  - "wait_for_match and wait_for_resolution helpers in queue.rs"
affects: []

# Tech tracking
tech-stack:
  added: []
  patterns: [sequential-game-loop, poll-wait-resolve-requeue]

key-files:
  created: []
  modified: [src/main.rs, src/commands/queue.rs]

key-decisions:
  - "Local wait_for_resolution helper in queue.rs instead of importing match_detail to avoid coupling"
  - "Extracted queue_once from execute to separate single-queue logic from game loop"
  - "JSON mode collects results into Vec and emits single array at end"

patterns-established:
  - "Sequential loop pattern: queue_once -> wait_for_match -> wait_for_resolution -> sleep -> repeat"

requirements-completed: [UX-07]

# Metrics
duration: 2min
completed: 2026-03-20
---

# Phase 08 Plan 02: --games N Flag Summary

**Sequential multi-game queue loop with automatic match polling and resolution waiting**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-20T13:36:18Z
- **Completed:** 2026-03-20T13:38:05Z
- **Tasks:** 1
- **Files modified:** 2

## Accomplishments
- Added `--games N` flag to queue command with default 1 (backward compatible)
- Implemented `games_loop` for sequential queue->match->resolution->re-queue lifecycle
- Added `wait_for_match` helper using `poll::poll_once` for match assignment polling
- Added `wait_for_resolution` helper for match resolution polling
- JSON mode collects all game results into a single JSON array output

## Task Commits

Each task was committed atomically:

1. **Task 1: Add --games N sequential loop to queue command** - `d230d11` (feat)

## Files Created/Modified
- `src/main.rs` - Added `games: u64` to Queue variant, pass to execute
- `src/commands/queue.rs` - Full rewrite with games_loop, queue_once, wait_for_match, wait_for_resolution

## Decisions Made
- Used local `wait_for_resolution` helper instead of importing `match_detail::fetch_match` to avoid coupling between commands
- Extracted `queue_once` as a private helper so single-game mode reuses the same code path as multi-game
- JSON mode in multi-game suppresses per-queue JSON output and emits a collected array at the end

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 08 complete: all UX improvements (--wait on poll, --wait-for-resolution on match, --games on queue) implemented
- CLI ready for agent automation workflows with full game lifecycle management

---
*Phase: 08-client-ux*
*Completed: 2026-03-20*
