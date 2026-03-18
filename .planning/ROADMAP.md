# Roadmap: ClawDuel Agent Skill

## Overview

Transform the existing ClawDuel CLI into a self-installing agent skill. Phase 1 makes the CLI globally installable and adds the queue timeout flag. Phase 2 adds non-interactive key management and multi-agent keystore support so agents can set up and operate without TTY access. Phase 3 authors the skill.md document that gives any Claude Code agent everything it needs to bootstrap, configure keys, and compete autonomously.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: CLI Global Binary** - Package CLI as installable global command with queue timeout support
- [ ] **Phase 2: Agent Key Management** - Non-interactive keystore creation and multi-agent keystore selection
- [ ] **Phase 3: Skill Document** - Author skill.md with bootstrap, fight loop, key setup docs, and strategy guidance

## Phase Details

### Phase 1: CLI Global Binary
**Goal**: An agent (or human) can install the CLI from a git clone and use every command via a global `claw-cli` binary
**Depends on**: Nothing (first phase)
**Requirements**: CLIP-01, CLIP-02, CLIP-03, CLIP-04, QUES-01, QUES-02
**Success Criteria** (what must be TRUE):
  1. After `git clone && npm install && npm link`, running `claw-cli help` prints usage and exits 0
  2. All existing commands (register, deposit, balance, queue, dequeue, poll, submit, status, matches, match) work via the global `claw-cli` binary identically to the local invocation
  3. `claw-cli queue --timeout 1800` sets a 30-minute attestation deadline; omitting `--timeout` defaults to 3600 seconds
**Plans**: 1 plan

Plans:
- [ ] 01-01-PLAN.md — Package CLI as global binary (tsconfig, package.json bin, shebang, help text) and add --timeout flag to queue command

### Phase 2: Agent Key Management
**Goal**: An AI agent can create and use encrypted keystores without any TTY interaction or human prompts
**Depends on**: Phase 1
**Requirements**: KEYS-01, KEYS-02, MAGT-01, MAGT-02, MAGT-03
**Success Criteria** (what must be TRUE):
  1. Running `AGENT_PRIVATE_KEY=<key> CLAW_KEY_PASSWORD=<pw> claw-cli init --non-interactive` creates an encrypted keystore in `~/.clawduel/keystores/` with no prompts
  2. With `CLAW_KEY_PASSWORD` set, all CLI commands that need the private key decrypt the keystore without TTY prompts
  3. When only one keystore exists in `~/.clawduel/keystores/`, it is auto-selected; with multiple, `--agent <address>` or `CLAW_AGENT_ADDRESS` selects the correct one
**Plans**: 2 plans

Plans:
- [ ] 02-01-PLAN.md — Non-interactive init command and keystores directory structure
- [ ] 02-02-PLAN.md — Keystore discovery, --agent selection, and loadWallet refactor

### Phase 3: Skill Document
**Goal**: A Claude Code agent can fetch skill.md and have complete instructions to go from zero to competing in a ClawDuel match
**Depends on**: Phase 1, Phase 2
**Requirements**: SKIL-01, SKIL-02, SKIL-03, SKIL-04, SKIL-05, SKIL-06, SKIL-07, KEYS-03, KEYS-04, KEYS-05
**Success Criteria** (what must be TRUE):
  1. skill.md has valid YAML frontmatter and a Claude Code agent can parse it as a skill
  2. Following only skill.md instructions, an agent can clone the repo, install, and have a working `claw-cli` global binary
  3. skill.md documents both key management paths (encrypted keystore via ethers.js `Wallet.encrypt()` and direct `AGENT_PRIVATE_KEY` env var) with security tradeoff explanation
  4. skill.md contains a complete fight loop with exact CLI commands for every step: queue, poll, research, submit
  5. skill.md documents all env vars with defaults, all prediction type format rules, and deadline behavior (absolute, no revisions, no-submit = loss)
**Plans**: 1 plan

Plans:
- [ ] 03-01-PLAN.md — Rewrite skill.md with spec-compliant frontmatter, bootstrap, key management, fight loop, env vars, prediction rules, and strategy tips

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. CLI Global Binary | 1/1 | Complete | 2026-03-18 |
| 2. Agent Key Management | 0/2 | Not started | - |
| 3. Skill Document | 0/1 | Not started | - |
