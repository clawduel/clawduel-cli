---
phase: "05"
plan: "01"
title: "Port All CLI Commands to Rust"
subsystem: commands
tags: [commands, eip712, alloy, on-chain, api]
dependency_graph:
  requires: [04-02]
  provides: [all-commands, eip712-signing, prediction-sanitization]
  affects: [main.rs, commands/*, contracts.rs]
tech_stack:
  added: [alloy-contract, alloy-provider, alloy-sol-types]
  patterns: [sol-macro-rpc, eip712-hash-then-sign, text-sanitization]
key_files:
  created:
    - src/contracts.rs
    - src/commands/register.rs
    - src/commands/deposit.rs
    - src/commands/balance.rs
    - src/commands/queue.rs
    - src/commands/dequeue.rs
    - src/commands/poll.rs
    - src/commands/submit.rs
    - src/commands/status.rs
    - src/commands/matches.rs
    - src/commands/match_detail.rs
  modified:
    - Cargo.toml
    - Cargo.lock
    - src/lib.rs
    - src/commands/mod.rs
    - src/main.rs
decisions:
  - "Used alloy sol! macro with #[sol(rpc)] for contract ABIs instead of manual ABI encoding"
  - "Computed EIP-712 hash via SolStruct::eip712_signing_hash then sign_hash, since sign_typed_data requires eip712 feature"
  - "Upgraded alloy from 1.6.3 to 1.7.3 for contract and provider support"
  - "Implemented ISO 8601 parser manually for poll wait to avoid chrono dependency"
metrics:
  duration: "11 min"
  completed: "2026-03-19"
  tasks_completed: 6
  tasks_total: 6
  files_created: 11
  files_modified: 5
---

# Phase 5 Plan 01: Port All CLI Commands to Rust Summary

All 10 CLI commands ported from TypeScript to Rust with alloy on-chain interactions, EIP-712 attestation signing, and prediction text sanitization.

## What Was Built

### Contracts Module (src/contracts.rs)
- ABI definitions via sol! macro for USDC (ERC20), Bank, and ClawDuel contracts
- EIP-712 JoinDuelAttestation struct for queue signing
- Contract address resolution from env vars with defaults
- alloy HTTP provider construction
- USDC amount parsing (f64 to U256) and formatting (U256 to string)

### Command Implementations
1. **register** - POST /agents/register with nickname
2. **deposit** - On-chain USDC approve + bank deposit with balance check
3. **balance** - On-chain bank balanceOf + lockedBalanceOf, formatted as USDC
4. **queue** - Random nonce generation (checked against usedNonces), EIP-712 typed data signing, POST /duels/queue
5. **dequeue** - DELETE /duels/queue with betTier
6. **poll** - GET /matches/active/{address}, handles waiting_ready (POST readyUrl), waiting_start (sleep until startsAt), re-poll
7. **submit** - Text sanitization (control chars, whitespace normalization, newline limits), POST /matches/{matchId}/submit
8. **status** - GET /api/agents/{address} + on-chain balance data
9. **matches** - GET /api/matches with query params (status, page, category, from, to)
10. **match** - GET /api/matches/{matchId} with resolution summary for resolved matches

### Main Wiring (src/main.rs)
- All commands wired with wallet loading, HTTP client, and provider creation
- Wallet command doesn't require wallet loading; all others do
- Private key hex extracted for secret scanning in HTTP client

## Decisions Made

1. Used `sol!` macro with `#[sol(rpc)]` for type-safe contract ABIs
2. Computed EIP-712 signing hash via `SolStruct::eip712_signing_hash()` then `sign_hash()` - avoids needing the optional `eip712` feature flag on the signer
3. Upgraded alloy from 1.6.3 to 1.7.3 because sub-crates (alloy-contract, alloy-provider) at 1.6.3 were not published to crates.io
4. Built minimal ISO 8601 parser for poll wait times instead of adding chrono dependency

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] alloy 1.6.3 sub-crates missing from crates.io**
- **Found during:** Task 1
- **Issue:** alloy-contract and alloy-provider at 1.6.3 were not published
- **Fix:** Upgraded alloy to 1.7.3, updated rust-version to 1.91.0
- **Files modified:** Cargo.toml, Cargo.lock

**2. [Rule 1 - Bug] sol! macro return type handling**
- **Found during:** Task 2-3
- **Issue:** Single-return-value contract calls return the value directly, not a struct wrapper
- **Fix:** Removed struct destructuring, use values directly
- **Files modified:** balance.rs, status.rs, deposit.rs, queue.rs

**3. [Rule 3 - Blocking] sign_typed_data requires feature flag**
- **Found during:** Task 4
- **Issue:** `Signer::sign_typed_data` gated behind `eip712` feature
- **Fix:** Used `SolStruct::eip712_signing_hash` + `sign_hash` instead
- **Files modified:** queue.rs

## Commits

| Task | Description | Commit | Key Files |
|------|-------------|--------|-----------|
| 1 | Contracts module + alloy deps | 61db8d9 | contracts.rs, Cargo.toml |
| 2-5 | All 10 command modules | b4c0cf7 | commands/*.rs |
| 6 | Wire commands in main.rs | ace733e | main.rs |

## Verification

- `cargo build` passes with zero errors
- `cargo test` passes: 17 tests (10 wallet + 7 sanitization)
- `clawduel --help` shows all 10 commands
- EIP-712 signing implemented in queue command
- Prediction sanitization implemented with 7 unit tests
