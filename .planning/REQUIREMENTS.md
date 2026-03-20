# Requirements: ClawDuel CLI v2.0 Rust Rewrite

**Defined:** 2026-03-19
**Core Value:** A Claude Code agent can go from zero to completing a full ClawDuel match autonomously

## v2.0 Requirements

Requirements for v2.0 Rust rewrite. Each maps to roadmap phases.

### Core Rewrite

- [x] **CORE-01**: CLI compiles to single Rust binary with clap derive subcommands
- [x] **CORE-02**: User can run all existing commands: register, deposit, balance, queue, dequeue, poll, submit, status, matches, match
- [x] **CORE-03**: CLI performs EIP-712 attestation signing for queue entries using alloy
- [x] **CORE-04**: CLI performs EIP-191 auth header signing for backend requests
- [x] **CORE-05**: CLI detects and blocks secret leaks in all outgoing request bodies
- [x] **CORE-06**: CLI validates backend URLs against SSRF vectors
- [x] **CORE-07**: CLI sanitizes prediction text before submission
- [x] **CORE-08**: CLI supports request timeouts on all HTTP calls

### Wallet Management

- [x] **WALLET-01**: User can create a new keypair and encrypted keystore via `wallet create`
- [x] **WALLET-02**: User can import an existing private key into encrypted keystore via `wallet import`
- [x] **WALLET-03**: User can view wallet address and key source via `wallet show`
- [x] **WALLET-04**: User can delete a keystore via `wallet delete`
- [x] **WALLET-05**: User can select agent keystore via `--agent` flag or `CLAW_AGENT_ADDRESS` env var

### Output & UX

- [x] **UX-01**: User can choose output format via global `--output table|json` flag (default: table)
- [x] **UX-02**: Table output uses pretty formatting with tabled crate
- [x] **UX-03**: JSON output is machine-parseable for agent consumption
- [x] **UX-04**: User can launch interactive shell via `clawduel shell` with readline history
- [x] **UX-05**: All commands work both as CLI subcommands and inside the interactive shell
- [x] **UX-06**: `poll --wait` polls with configurable interval until match has status waiting_submissions with a problem
- [ ] **UX-07**: `queue --games N` queues for N sequential games, waiting for each to complete before re-queuing
- [x] **UX-08**: `match --wait-for-resolution` polls until match status is resolved

### Config & Distribution

- [x] **CONF-01**: CLI reads config from `~/.config/clawduel/config.json`
- [x] **CONF-02**: Config priority resolution: CLI flag > env var > config file
- [x] **CONF-03**: User can check API health via `clawduel status`
- [x] **CONF-04**: User can self-upgrade via `clawduel upgrade`
- [x] **CONF-05**: Release binary is optimized with LTO, strip, and single codegen unit
- [x] **CONF-06**: Non-interactive mode works without TTY when env vars are set

## v3.0 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Advanced

- **ADV-01**: WebSocket-based real-time match notifications
- **ADV-02**: Match history analytics and win/loss tracking
- **ADV-03**: Plugin system for custom prediction strategies

## Out of Scope

| Feature | Reason |
|---------|--------|
| Backend/API changes | Separate repo |
| Smart contract changes | Contracts deployed and stable |
| Programmatic SDK/library | CLI is sole interface |
| Web frontend changes | Separate concern |
| Legacy TypeScript CLI maintenance | Replaced by Rust binary |
| `compete` command | Agent orchestrates via skill.md |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| CORE-01 | Phase 4 | Complete (04-01) |
| CORE-02 | Phase 5 | Complete (05-01) |
| CORE-03 | Phase 5 | Complete (05-01) |
| CORE-04 | Phase 4 | Complete |
| CORE-05 | Phase 4 | Complete |
| CORE-06 | Phase 4 | Complete |
| CORE-07 | Phase 5 | Complete (05-01) |
| CORE-08 | Phase 4 | Complete |
| WALLET-01 | Phase 4 | Complete (04-01) |
| WALLET-02 | Phase 4 | Complete (04-01) |
| WALLET-03 | Phase 4 | Complete (04-01) |
| WALLET-04 | Phase 4 | Complete (04-01) |
| WALLET-05 | Phase 4 | Complete (04-01) |
| UX-01 | Phase 6 | Complete (06-01) |
| UX-02 | Phase 6 | Complete (06-01) |
| UX-03 | Phase 6 | Complete (06-01) |
| UX-04 | Phase 6 | Complete (06-01) |
| UX-05 | Phase 6 | Complete (06-01) |
| UX-06 | Phase 8 | Complete (08-01) |
| UX-07 | Phase 8 | Planned (08-02) |
| UX-08 | Phase 8 | Complete (08-01) |
| CONF-01 | Phase 4 | Complete (04-01) |
| CONF-02 | Phase 4 | Complete (04-01) |
| CONF-03 | Phase 6 | Complete (06-01) |
| CONF-04 | Phase 6 | Complete (06-01) |
| CONF-05 | Phase 6 | Complete (06-01) |
| CONF-06 | Phase 4 | Complete (04-01) |

**Coverage:**
- v2.0 requirements: 24 total (all complete)
- v2.1 requirements: 3 total (UX-06, UX-07, UX-08)
- Mapped to phases: 27
- Unmapped: 0

---
*Requirements defined: 2026-03-19*
*Last updated: 2026-03-20 after Phase 8 planning*
