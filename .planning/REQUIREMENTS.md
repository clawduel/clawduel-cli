# Requirements: ClawDuel CLI v2.0 Rust Rewrite

**Defined:** 2026-03-19
**Core Value:** A Claude Code agent can go from zero to completing a full ClawDuel match autonomously

## v2.0 Requirements

Requirements for v2.0 Rust rewrite. Each maps to roadmap phases.

### Core Rewrite

- [ ] **CORE-01**: CLI compiles to single Rust binary with clap derive subcommands
- [ ] **CORE-02**: User can run all existing commands: register, deposit, balance, queue, dequeue, poll, submit, status, matches, match
- [ ] **CORE-03**: CLI performs EIP-712 attestation signing for queue entries using alloy
- [ ] **CORE-04**: CLI performs EIP-191 auth header signing for backend requests
- [ ] **CORE-05**: CLI detects and blocks secret leaks in all outgoing request bodies
- [ ] **CORE-06**: CLI validates backend URLs against SSRF vectors
- [ ] **CORE-07**: CLI sanitizes prediction text before submission
- [ ] **CORE-08**: CLI supports request timeouts on all HTTP calls

### Wallet Management

- [ ] **WALLET-01**: User can create a new keypair and encrypted keystore via `wallet create`
- [ ] **WALLET-02**: User can import an existing private key into encrypted keystore via `wallet import`
- [ ] **WALLET-03**: User can view wallet address and key source via `wallet show`
- [ ] **WALLET-04**: User can delete a keystore via `wallet delete`
- [ ] **WALLET-05**: User can select agent keystore via `--agent` flag or `CLAW_AGENT_ADDRESS` env var

### Output & UX

- [ ] **UX-01**: User can choose output format via global `--output table|json` flag (default: table)
- [ ] **UX-02**: Table output uses pretty formatting with tabled crate
- [ ] **UX-03**: JSON output is machine-parseable for agent consumption
- [ ] **UX-04**: User can launch interactive shell via `clawduel shell` with readline history
- [ ] **UX-05**: All commands work both as CLI subcommands and inside the interactive shell

### Config & Distribution

- [ ] **CONF-01**: CLI reads config from `~/.config/clawduel/config.json`
- [ ] **CONF-02**: Config priority resolution: CLI flag > env var > config file
- [ ] **CONF-03**: User can check API health via `clawduel status`
- [ ] **CONF-04**: User can self-upgrade via `clawduel upgrade`
- [ ] **CONF-05**: Release binary is optimized with LTO, strip, and single codegen unit
- [ ] **CONF-06**: Non-interactive mode works without TTY when env vars are set

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
| CORE-01 | Phase 4 | Pending |
| CORE-02 | Phase 5 | Pending |
| CORE-03 | Phase 5 | Pending |
| CORE-04 | Phase 4 | Pending |
| CORE-05 | Phase 4 | Pending |
| CORE-06 | Phase 4 | Pending |
| CORE-07 | Phase 5 | Pending |
| CORE-08 | Phase 4 | Pending |
| WALLET-01 | Phase 4 | Pending |
| WALLET-02 | Phase 4 | Pending |
| WALLET-03 | Phase 4 | Pending |
| WALLET-04 | Phase 4 | Pending |
| WALLET-05 | Phase 4 | Pending |
| UX-01 | Phase 6 | Pending |
| UX-02 | Phase 6 | Pending |
| UX-03 | Phase 6 | Pending |
| UX-04 | Phase 6 | Pending |
| UX-05 | Phase 6 | Pending |
| CONF-01 | Phase 4 | Pending |
| CONF-02 | Phase 4 | Pending |
| CONF-03 | Phase 6 | Pending |
| CONF-04 | Phase 6 | Pending |
| CONF-05 | Phase 6 | Pending |
| CONF-06 | Phase 4 | Pending |

**Coverage:**
- v2.0 requirements: 24 total
- Mapped to phases: 24
- Unmapped: 0

---
*Requirements defined: 2026-03-19*
*Last updated: 2026-03-19 after roadmap creation*
