# Roadmap: ClawDuel Agent Skill

## Milestones

- ✅ **v1.0 Agent Skill** — Phases 1-3 (shipped 2026-03-18)
- 🚧 **v2.0 Rust Rewrite** — Phases 4-7 (in progress)

## Phases

<details>
<summary>✅ v1.0 Agent Skill (Phases 1-3) — SHIPPED 2026-03-18</summary>

- [x] Phase 1: CLI Global Binary (1/1 plans) — completed 2026-03-18
- [x] Phase 2: Agent Key Management (2/2 plans) — completed 2026-03-18
- [x] Phase 3: Skill Document (1/1 plan) — completed 2026-03-18

</details>

### v2.0 Rust Rewrite

- [ ] **Phase 4: Foundation** - Rust binary scaffolding, config system, wallet management, HTTP client with auth and security
- [ ] **Phase 5: Command Port** - Port all existing CLI commands with EIP-712 signing and input validation
- [ ] **Phase 6: Output, Shell & Distribution** - Dual output format, interactive shell, status/upgrade commands, release optimization
- [ ] **Phase 7: Cleanup & Docs** - Remove old TypeScript code, update .gitignore, README, and skill.md

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
**Plans**: TBD

Plans:
- [ ] 06-01: TBD

### Phase 7: Cleanup & Docs
**Goal**: Remove old TypeScript code, update .gitignore for Rust, update README and skill.md to reflect the Rust CLI
**Depends on**: Phase 6
**Requirements**: CLEAN-01, CLEAN-02, CLEAN-03, CLEAN-04
**Success Criteria** (what must be TRUE):
  1. All old TypeScript files (clawduel-cli.ts, tsconfig.json, package.json, dist/, node_modules/) are removed
  2. .gitignore is updated for Rust (target/, *.pdb, etc.) and removes Node entries
  3. README reflects Rust CLI installation (cargo install / binary download), commands, and usage
  4. skill.md is updated to reference the Rust binary instead of npm package
**Plans**: TBD

Plans:
- [ ] 07-01: TBD

## Progress

**Execution Order:** Phase 4 → Phase 5 → Phase 6 → Phase 7

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. CLI Global Binary | v1.0 | 1/1 | Complete | 2026-03-18 |
| 2. Agent Key Management | v1.0 | 2/2 | Complete | 2026-03-18 |
| 3. Skill Document | v1.0 | 1/1 | Complete | 2026-03-18 |
| 4. Foundation | v2.0 | 2/2 | Complete | 2026-03-19 |
| 5. Command Port | v2.0 | 1/1 | Complete | 2026-03-19 |
| 6. Output, Shell & Distribution | v2.0 | 0/? | Not started | - |
| 7. Cleanup & Docs | v2.0 | 0/? | Not started | - |
