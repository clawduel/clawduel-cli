---
phase: 04-foundation
plan: 01
subsystem: cli
tags: [rust, clap, alloy, eth-keystore, wallet, config]

requires:
  - phase: v1.0
    provides: existing TypeScript CLI behavior to replicate

provides:
  - Rust binary with clap subcommands for all CLI commands
  - Config system with flag > env > config file > default priority
  - Encrypted keystore wallet management (create, import, show, delete)
  - Multi-agent wallet selection via --agent flag and CLAW_AGENT_ADDRESS env
  - Non-interactive mode support via CLAW_KEY_PASSWORD and CLAW_NON_INTERACTIVE

affects: [04-02, 05-01, 06-01]

tech-stack:
  added: [alloy 1.6, clap 4, eth-keystore 0.5, rpassword 5, serde, tokio, reqwest, anyhow, dirs, hex, rand]
  patterns: [clap derive subcommands, flag > env > config priority resolution, PrivateKeySigner keystore encryption, lib.rs + bin pattern for testability]

key-files:
  created:
    - Cargo.toml
    - src/main.rs
    - src/lib.rs
    - src/config.rs
    - src/wallet.rs
    - src/commands/mod.rs
    - src/commands/wallet.rs
    - tests/config_test.rs
    - tests/wallet_test.rs
  modified: []

key-decisions:
  - "Used PrivateKeySigner type alias (not generic LocalSigner) for ergonomic wallet management"
  - "Added lib.rs re-export layer so integration tests can import modules directly"
  - "Used eth-keystore 0.5 (not 0.6 which doesn't exist) for keystore encryption"
  - "rpassword v5 uses prompt_password_stderr (not prompt_password which is v7+)"
  - "Keystores named by lowercase address without 0x prefix for filesystem compatibility"

patterns-established:
  - "Config priority: flag > env var > config file > default constant"
  - "Keystore path: ~/.clawduel/keystores/<address_lowercase>.json"
  - "Password resolution: argument > CLAW_KEY_PASSWORD env > interactive rpassword prompt"
  - "Non-interactive detection: CLAW_NON_INTERACTIVE=1 or !stdin.is_terminal()"

requirements-completed: [CORE-01, WALLET-01, WALLET-02, WALLET-03, WALLET-04, WALLET-05, CONF-01, CONF-02, CONF-06]

duration: 9min
completed: 2026-03-19
---

# Phase 4 Plan 01: Rust Scaffold + Config + Wallet Summary

**Rust CLI binary with clap derive subcommands, encrypted keystore wallet CRUD, and config priority resolution system**

## Performance

- **Duration:** 9 min
- **Started:** 2026-03-19T22:06:20Z
- **Completed:** 2026-03-19T22:15:15Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments
- Full clap CLI with all 11 subcommand stubs (wallet, register, deposit, balance, queue, dequeue, poll, submit, status, matches, match)
- Config system with flag > env > config file > default priority chain for backend_url, rpc_url, agent_address
- Wallet lifecycle: create (random keypair), import (existing key), show (address + source), delete (with confirmation)
- Multi-agent support via --agent flag and CLAW_AGENT_ADDRESS env var
- 21 passing tests (11 config + 10 wallet)

## Task Commits

Each task was committed atomically:

1. **Task 1: Rust project scaffold + config system** - `6c2e4af` (feat)
2. **Task 2: Wallet management with encrypted keystores** - `43137ef` (feat)

## Files Created/Modified
- `Cargo.toml` - Rust project with all Phase 4 dependencies
- `src/main.rs` - CLI entry point with clap Parser, all subcommand stubs
- `src/lib.rs` - Library re-exports for integration test access
- `src/config.rs` - Config loading, save, priority resolution (backend_url, rpc_url, agent_address, is_interactive)
- `src/wallet.rs` - Keystore CRUD (create, import, decrypt, discover, select, delete, load_wallet)
- `src/commands/mod.rs` - Command module declarations
- `src/commands/wallet.rs` - Wallet subcommand handlers (create, import, show, delete)
- `tests/config_test.rs` - 11 tests for config priority, roundtrip, non-interactive detection
- `tests/wallet_test.rs` - 10 tests for keystore CRUD, selection, env var fallback

## Decisions Made
- Used `PrivateKeySigner` type alias instead of generic `LocalSigner<C>` for cleaner API
- Created `lib.rs` alongside `main.rs` so integration tests can import crate modules
- Used `eth-keystore` v0.5 (latest available, plan referenced non-existent v0.6)
- Used `rpassword` v5's `prompt_password_stderr` API (v5 doesn't have `prompt_password`)
- Keystore files named `<address_hex_lowercase>.json` (no 0x prefix) matching TS CLI convention
- Updated rust-version to 1.88.0 (alloy 1.6 requires it)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] eth-keystore v0.6 does not exist**
- **Found during:** Task 1 (Cargo.toml creation)
- **Issue:** Plan specified eth-keystore = "0.6" but latest published version is 0.5.0
- **Fix:** Used eth-keystore = "0.5"
- **Files modified:** Cargo.toml
- **Verification:** cargo build succeeds
- **Committed in:** 6c2e4af

**2. [Rule 3 - Blocking] Rust 1.90 too old for alloy 1.6**
- **Found during:** Task 1 (first cargo build)
- **Issue:** alloy 1.6.3 requires rustc 1.88+, system had 1.90 but Cargo.lock resolved to 1.7.3 needing 1.91
- **Fix:** Ran rustup update stable (got 1.94.0), set rust-version = "1.88.0", let Cargo resolve to alloy 1.6.3
- **Files modified:** Cargo.toml
- **Verification:** cargo build succeeds
- **Committed in:** 6c2e4af

**3. [Rule 1 - Bug] rpassword v5 API difference**
- **Found during:** Task 2 (wallet module)
- **Issue:** Used `rpassword::prompt_password()` which exists in v7+ but not v5
- **Fix:** Changed to `rpassword::prompt_password_stderr()` (v5 API)
- **Files modified:** src/wallet.rs, src/commands/wallet.rs
- **Verification:** cargo build succeeds, all tests pass
- **Committed in:** 43137ef

**4. [Rule 1 - Bug] LocalSigner requires generic parameter**
- **Found during:** Task 2 (wallet module)
- **Issue:** `LocalSigner` in alloy 1.6 requires a generic curve parameter
- **Fix:** Used `PrivateKeySigner` type alias which specifies `k256::ecdsa::SigningKey`
- **Files modified:** src/wallet.rs
- **Verification:** cargo build succeeds
- **Committed in:** 43137ef

---

**Total deviations:** 4 auto-fixed (2 blocking, 2 bug)
**Impact on plan:** All fixes necessary for compilation. No scope creep.

## Issues Encountered
None beyond the auto-fixed deviations above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Binary compiles and all 11 subcommands are stubbed
- Config and wallet infrastructure ready for 04-02 (security module + auth + HTTP client)
- Wallet keystore format compatible with existing TypeScript CLI keystores

---
*Phase: 04-foundation*
*Completed: 2026-03-19*
