---
phase: 08-client-ux
plan: 01
subsystem: cli
tags: [polling, tokio, clap, ux]

# Dependency graph
requires:
  - phase: 05-command-port
    provides: poll and match commands with HTTP client
provides:
  - poll --wait polling loop with configurable interval and timeout
  - match --wait-for-resolution polling loop until resolved
  - poll_once() and fetch_match() reusable helpers
affects: [08-02-queue-games]

# Tech tracking
tech-stack:
  added: []
  patterns: [poll_once/fetch_match helper extraction, Instant-based timeout loops]

key-files:
  created: []
  modified:
    - src/main.rs
    - src/commands/poll.rs
    - src/commands/match_detail.rs

key-decisions:
  - "Extracted poll_once() and fetch_match() helpers for reusable single-cycle logic"
  - "Table mode prints progress per cycle, JSON mode emits final result only"
  - "Match resolution default interval 10s (vs poll 3s) since resolution takes minutes"

patterns-established:
  - "Polling loop pattern: Instant::now() + tokio::time::sleep with configurable interval/timeout"
  - "Helper extraction: separate fetch from display for wait/non-wait reuse"

requirements-completed: [UX-06, UX-08]

# Metrics
duration: 2min
completed: 2026-03-20
---

# Phase 8 Plan 1: Poll --wait and Match --wait-for-resolution Summary

**Client-side HTTP polling loops for poll (--wait) and match (--wait-for-resolution) with configurable intervals, timeouts, and format-aware output**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-20T13:30:59Z
- **Completed:** 2026-03-20T13:33:23Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Poll command supports --wait flag that polls every N seconds until match reaches waiting_submissions with a problem
- Match command supports --wait-for-resolution flag that polls until match status is resolved
- Both commands have --wait-interval and --wait-timeout for full configurability
- JSON mode suppresses intermediate output, table mode shows timed progress lines
- Backward compatible: without new flags, behavior is identical to before

## Task Commits

Each task was committed atomically:

1. **Task 1: Add --wait polling loop to poll command** - `fe4eb1c` (feat)
2. **Task 2: Add --wait-for-resolution polling loop to match command** - `2935ea7` (feat)

## Files Created/Modified
- `src/main.rs` - Added --wait/--wait-interval/--wait-timeout to Poll and --wait-for-resolution/--wait-interval/--wait-timeout to Match variants
- `src/commands/poll.rs` - Extracted poll_once() helper, added polling loop with timeout tracking
- `src/commands/match_detail.rs` - Extracted fetch_match() and display_match() helpers, added resolution polling loop

## Decisions Made
- Extracted poll_once() and fetch_match() helpers so wait and non-wait paths share the same fetch logic
- Poll default interval is 3s (match readiness is fast), match resolution default is 10s (takes minutes)
- Changed match_detail error handling from eprintln to anyhow::bail for cleaner error propagation in polling loops

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Poll and match polling loops complete, ready for 08-02 (queue --games sequential multi-game loop)
- The poll_once() helper can be reused by the --games loop to detect match readiness

## Self-Check: PASSED

- All source files exist (poll.rs, match_detail.rs, main.rs)
- Commit fe4eb1c exists (Task 1)
- Commit 2935ea7 exists (Task 2)
- cargo build succeeds
- poll --help shows --wait, --wait-interval, --wait-timeout
- match --help shows --wait-for-resolution, --wait-interval, --wait-timeout

---
*Phase: 08-client-ux*
*Completed: 2026-03-20*
