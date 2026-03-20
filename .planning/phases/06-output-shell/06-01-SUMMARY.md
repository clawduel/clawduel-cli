---
phase: "06"
plan: "01"
subsystem: output-shell-distribution
tags: [output, shell, upgrade, release]
dependency-graph:
  requires: [05-01]
  provides: [dual-output, interactive-shell, upgrade-command]
  affects: [all-commands, main.rs, Cargo.toml]
tech-stack:
  added: [tabled, rustyline]
  patterns: [OutputFormat-enum, global-clap-flag, REPL-pattern, Box-pin-recursion]
key-files:
  created:
    - src/output.rs
    - src/shell.rs
    - src/commands/upgrade.rs
  modified:
    - src/main.rs
    - src/lib.rs
    - src/commands/mod.rs
    - src/commands/balance.rs
    - src/commands/status.rs
    - src/commands/register.rs
    - src/commands/deposit.rs
    - src/commands/queue.rs
    - src/commands/dequeue.rs
    - src/commands/poll.rs
    - src/commands/submit.rs
    - src/commands/matches.rs
    - src/commands/match_detail.rs
    - Cargo.toml
decisions:
  - "Used tabled 0.17 for pretty table output with Style::rounded()"
  - "Used rustyline 15 for readline REPL with history"
  - "Box::pin shell future to break async recursion cycle"
  - "OutputFormat enum with clap ValueEnum derive for --output flag"
  - "Release profile already had LTO/strip/codegen-units=1 from earlier phase"
metrics:
  duration: "6 min"
  completed: "2026-03-19"
---

# Phase 6 Plan 01: Output, Shell & Distribution Summary

Dual output format (table/json) via global --output flag, interactive REPL shell with rustyline, self-update upgrade command, and release binary optimization.

## What Was Done

### Task 1: Output Module (649d3ec)
- Created `src/output.rs` with `OutputFormat` enum (Table/Json) deriving clap `ValueEnum`
- Added `print_json`, `print_table`, `print_detail`, and `render` helper functions
- Added `tabled` and `rustyline` dependencies to Cargo.toml

### Task 2: Global --output Flag + All Commands Wired (ebf550a)
- Added `--output table|json` global flag (short: `-o`, default: `table`) to Cli struct
- Updated all 10 command handlers to accept `OutputFormat` parameter
- Table mode uses `tabled` crate for pretty detail tables and list tables
- JSON mode outputs clean `serde_json::to_string_pretty` without side-effect println
- Added Shell and Upgrade variants to Commands enum

### Task 3: Interactive Shell (66a513e)
- Created `src/shell.rs` with rustyline-based REPL
- Prompt: `clawduel> ` with readline history support
- Parses input, prepends "clawduel", re-parses with `Cli::try_parse_from`
- Handles exit/quit, blocks nested shell invocation
- Error output respects current OutputFormat

### Task 4: Upgrade Command (232f088)
- Created `src/commands/upgrade.rs` with full GitHub release auto-update
- Checks latest release tag, downloads tarball, verifies SHA256 checksum
- Replaces current binary with sudo fallback
- Falls back to manual `cargo install` instructions if GitHub unreachable

### Task 5: Compilation Verification (686c2f1)
- Fixed `crate::output` path in shell.rs (binary crate needs `clawduel_cli::output`)
- Added `Box::pin` for shell future to resolve async recursion
- `cargo build` succeeds cleanly

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed crate path resolution in shell.rs**
- Found during: Task 5
- Issue: shell.rs used `crate::output::OutputFormat` but shell.rs is in the binary crate, not the library
- Fix: Changed to `clawduel_cli::output::OutputFormat`
- Files modified: src/shell.rs

**2. [Rule 3 - Blocking] Fixed async recursion in run/shell cycle**
- Found during: Task 5
- Issue: `run()` calls `shell::run_shell()` which calls `run()` -- infinite-size future
- Fix: `Box::pin(crate::shell::run_shell()).await` to break the cycle
- Files modified: src/main.rs

## Notes

- Release profile (LTO, strip, codegen-units=1, panic=abort) was already present in Cargo.toml from Phase 4, satisfying CONF-05
- All commands work both as CLI subcommands and inside the interactive shell (UX-05)
