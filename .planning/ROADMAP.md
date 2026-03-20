# Roadmap: ClawDuel Agent Skill

## Milestones

- ✅ **v1.0 Agent Skill** — Phases 1-3 (shipped 2026-03-18)
- ✅ **v2.0 Rust Rewrite** — Phases 4-7 (shipped 2026-03-19)
- 🚧 **v2.1 Client UX** — Phase 8 (in progress)
- 📋 **v3.0 Multi-Duel Support** — Phases 9-10 (planned)

## Phases

<details>
<summary>✅ v1.0 Agent Skill (Phases 1-3) — SHIPPED 2026-03-18</summary>

- [x] Phase 1: CLI Global Binary (1/1 plans) — completed 2026-03-18
- [x] Phase 2: Agent Key Management (2/2 plans) — completed 2026-03-18
- [x] Phase 3: Skill Document (1/1 plan) — completed 2026-03-18

</details>

### v2.0 Rust Rewrite

- [x] **Phase 4: Foundation** - Rust binary scaffolding, config system, wallet management, HTTP client with auth and security
- [x] **Phase 5: Command Port** - Port all existing CLI commands with EIP-712 signing and input validation
- [x] **Phase 6: Output, Shell & Distribution** - Dual output format, interactive shell, status/upgrade commands, release optimization
- [x] **Phase 7: Cleanup & Docs** - Remove old TypeScript code, update .gitignore, README, and skill.md

### v3.0 Multi-Duel Support

- [x] **Phase 9: Multi-Duel Lobby Commands** - Lobby create, join, list, and status commands with EIP-712 multi-duel attestation signing (completed 2026-03-20)
- [ ] **Phase 10: Multi-Duel Match Flow** - Multi-duel prediction submission, results display, shell integration, and skill.md updates

## Phase Details

### Phase 4: Foundation
**Goal**: User has a working Rust binary with wallet management, config system, and authenticated HTTP client ready for commands
**Depends on**: v1.0 (shipped)
**Requirements**: CORE-01, CORE-04, CORE-05, CORE-06, CORE-08, WALLET-01, WALLET-02, WALLET-03, WALLET-04, WALLET-05, CONF-01, CONF-02, CONF-06
**Success Criteria** (what must be TRUE):
  1. User can run `clawduel --help` and see the clap-generated help with all subcommand stubs
  2. User can run `wallet create`, `wallet import`, `wallet show`, and `wallet delete` to fully manage keystores
  3. User can select a wallet via `--agent` flag or `CLAW_AGENT_ADDRESS` env var and it resolves correctly
  4. CLI reads config from `~/.config/clawduel/config.json` with flag > env > config priority resolution
  5. All HTTP requests include EIP-191 auth headers, enforce timeouts, block secret leaks, and reject SSRF URLs
**Plans**: 2 plans

Plans:
- [x] 04-01-PLAN.md — Rust scaffold + config system + wallet management (CORE-01, WALLET-*, CONF-*)
- [x] 04-02-PLAN.md — Security module + auth + authenticated HTTP client (CORE-04, CORE-05, CORE-06, CORE-08)

### Phase 5: Command Port
**Goal**: User can execute every existing CLI command (register, deposit, balance, queue, dequeue, poll, submit, status, matches, match) in the Rust binary
**Depends on**: Phase 4
**Requirements**: CORE-02, CORE-03, CORE-07
**Success Criteria** (what must be TRUE):
  1. User can run all 10 commands (register, deposit, balance, queue, dequeue, poll, submit, status, matches, match) and get correct results from the backend
  2. Queue command produces valid EIP-712 attestation signatures accepted by the backend
  3. Submit command sanitizes prediction text before sending to the API
**Plans**: 1 plan

Plans:
- [x] 05-01-PLAN.md -- Port all 10 CLI commands with EIP-712 signing and text sanitization (CORE-02, CORE-03, CORE-07)

### Phase 6: Output, Shell & Distribution
**Goal**: User has polished CLI with dual output format, interactive shell, health checks, self-upgrade, and optimized release binary
**Depends on**: Phase 5
**Requirements**: UX-01, UX-02, UX-03, UX-04, UX-05, CONF-03, CONF-04, CONF-05
**Success Criteria** (what must be TRUE):
  1. User can pass `--output json` to any command and get machine-parseable JSON, or omit it for pretty table output
  2. User can launch `clawduel shell` and execute any command interactively with readline history
  3. User can run `clawduel status` to check API health and `clawduel upgrade` to self-update the binary
  4. Release binary is built with LTO, stripped symbols, and single codegen unit for minimal size
**Plans**: 1 plan

Plans:
- [x] 06-01-PLAN.md -- Dual output format, interactive shell, upgrade command, release optimization (UX-01..05, CONF-03..05)

### Phase 7: Cleanup & Docs
**Goal**: Remove old TypeScript code, update .gitignore for Rust, update README and skill.md to reflect the Rust CLI
**Depends on**: Phase 6
**Requirements**: CLEAN-01, CLEAN-02, CLEAN-03, CLEAN-04
**Success Criteria** (what must be TRUE):
  1. All old TypeScript files (clawduel-cli.ts, tsconfig.json, package.json, dist/, node_modules/) are removed
  2. .gitignore is updated for Rust (target/, *.pdb, etc.) and removes Node entries
  3. README reflects Rust CLI installation (cargo install / binary download), commands, and usage
  4. skill.md is updated to reference the Rust binary instead of npm package
**Plans**: 1 plan

Plans:
- [x] 07-01-PLAN.md -- Remove old TS code, update .gitignore, README, and skill.md (CLEAN-01..04)

### Phase 8: Client-side UX Improvements
**Goal**: Agent can use --wait on poll, --games on queue, and --wait-for-resolution on match for autonomous multi-game play without manual re-running
**Depends on**: Phase 7
**Requirements**: UX-06, UX-07, UX-08
**Success Criteria** (what must be TRUE):
  1. `clawduel poll --wait` polls until match has status waiting_submissions with a problem present
  2. `clawduel queue 100 --games 3` queues for 3 sequential games, waiting for each to complete before re-queuing
  3. `clawduel match --id X --wait-for-resolution` polls until match status is resolved
  4. All new flags have configurable intervals and timeouts
  5. JSON mode emits final result only (no intermediate polling noise)
**Plans**: 2 plans

Plans:
- [x] 08-01-PLAN.md -- Poll --wait and match --wait-for-resolution polling loops (UX-06, UX-08)
- [ ] 08-02-PLAN.md -- Queue --games sequential multi-game loop (UX-07)

## Progress

**Execution Order:** Phase 4 -> Phase 5 -> Phase 6 -> Phase 7 -> Phase 8 -> Phase 9 -> Phase 10

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. CLI Global Binary | v1.0 | 1/1 | Complete | 2026-03-18 |
| 2. Agent Key Management | v1.0 | 2/2 | Complete | 2026-03-18 |
| 3. Skill Document | v1.0 | 1/1 | Complete | 2026-03-18 |
| 4. Foundation | v2.0 | 2/2 | Complete | 2026-03-19 |
| 5. Command Port | v2.0 | 1/1 | Complete | 2026-03-19 |
| 6. Output, Shell & Distribution | v2.0 | 1/1 | Complete | 2026-03-19 |
| 7. Cleanup & Docs | v2.0 | 1/1 | Complete | 2026-03-19 |
| 8. Client-side UX | v2.1 | 1/2 | In Progress | - |
| 9. Multi-Duel Lobby Commands | 2/2 | Complete   | 2026-03-20 | - |
| 10. Multi-Duel Match Flow | 1/2 | In Progress|  | - |

### Phase 9: Multi-Duel Lobby Commands
**Goal**: Agent can create, join, list, and inspect multi-duel lobbies via CLI with proper EIP-712 multi-duel attestation signing
**Depends on**: Phase 8
**Requirements**: MULTI-01, MULTI-02, MULTI-03, MULTI-04, MULTI-05, MULTI-06
**Success Criteria** (what must be TRUE):
  1. `clawduel lobby create 100 --max-participants 5` creates a multi-duel lobby and returns the lobby ID
  2. `clawduel lobby join <lobby-id>` signs a JoinMultiAttestation (EIP-712) and joins an existing lobby
  3. `clawduel lobby list` shows open lobbies with participant count, bet size, and status
  4. `clawduel lobby status <lobby-id>` shows lobby details including all joined participants
  5. All lobby commands support `--output json` for machine-parseable output
  6. EIP-712 signing uses MultiDuel contract domain and JoinMultiAttestation types matching the contract
**Plans**: 2 plans

Plans:
- [x] 09-01-PLAN.md — Contract types + lobby command implementation (MULTI-01..06)
- [x] 09-02-PLAN.md — Wire lobby into CLI entry point and shell (MULTI-01..05)

### Phase 10: Multi-Duel Match Flow
**Goal**: Agent can participate in multi-duel matches end-to-end — submit predictions, track match progress, and view ranked results with payouts
**Depends on**: Phase 9
**Requirements**: MULTI-07, MULTI-08, MULTI-09, MULTI-10, MULTI-11
**Success Criteria** (what must be TRUE):
  1. `clawduel submit` works for multi-duel matches (uses `/matches/:id/submit/multi` endpoint)
  2. `clawduel poll --wait` correctly handles multi-duel match states
  3. `clawduel match --id X` displays multi-duel results with all participant rankings and payouts (1st/2nd/3rd)
  4. Shell mode supports all lobby subcommands
  5. skill.md is updated to document multi-duel commands and the lobby workflow for autonomous agents
**Plans**: 2 plans

Plans:
- [ ] 10-01-PLAN.md — Multi-duel submit endpoint + ranked match results display (MULTI-07, MULTI-08, MULTI-09)
- [ ] 10-02-PLAN.md — Skill.md multi-duel documentation + shell verification (MULTI-10, MULTI-11)
