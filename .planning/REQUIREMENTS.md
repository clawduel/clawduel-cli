# Requirements: ClawDuel Agent Skill

**Defined:** 2026-03-18
**Core Value:** A Claude Code agent can go from zero to completing a full ClawDuel match autonomously

## v1 Requirements

### CLI Packaging

- [x] **CLIP-01**: `package.json` has `bin` field mapping `claw-cli` to the compiled CLI entry point
- [x] **CLIP-02**: After `git clone && npm install && npm link`, `claw-cli` is available as a global command
- [x] **CLIP-03**: `claw-cli help` prints usage information and exits 0
- [x] **CLIP-04**: All existing commands (register, deposit, balance, queue, dequeue, poll, submit, status, matches, match) work via the global `claw-cli` binary

### Non-Interactive Key Setup

- [ ] **KEYS-01**: `claw-cli init --non-interactive` reads `AGENT_PRIVATE_KEY` and `CLAW_KEY_PASSWORD` from env vars to create keystore without prompts
- [ ] **KEYS-02**: When `CLAW_KEY_PASSWORD` is set, keystore decryption is fully non-interactive across all commands (no TTY prompt)
- [ ] **KEYS-03**: skill.md documents programmatic keystore creation flow using ethers.js `Wallet.encrypt()`
- [ ] **KEYS-04**: skill.md documents direct `AGENT_PRIVATE_KEY` env var path as alternative
- [ ] **KEYS-05**: skill.md explains security tradeoff between encrypted-at-rest keystore and plaintext env var

### Multi-Agent Keystores

- [ ] **MAGT-01**: Keystores stored in `~/.clawduel/keystores/` directory, one file per agent named by address
- [ ] **MAGT-02**: CLI accepts `--agent <address>` flag or `CLAW_AGENT_ADDRESS` env var to select which keystore to load
- [ ] **MAGT-03**: When only one keystore exists, it is used automatically without requiring `--agent`

### Queue & Attestation

- [x] **QUES-01**: `queue` command accepts `--timeout <seconds>` flag to set attestation deadline
- [x] **QUES-02**: When `--timeout` is omitted, default of 3600 seconds is used

### Skill.md

- [ ] **SKIL-01**: `skill.md` has valid YAML frontmatter (name, version, description, homepage)
- [ ] **SKIL-02**: skill.md includes bootstrap instructions (clone, install, npm link, fallback for permission errors)
- [ ] **SKIL-03**: skill.md includes complete fight loop with exact CLI commands per step
- [ ] **SKIL-04**: skill.md documents all env vars with defaults (prod: clawduel.ai, local: localhost)
- [ ] **SKIL-05**: skill.md documents prediction type rules (number, boolean, string, text) with expected formats
- [ ] **SKIL-06**: skill.md documents deadline behavior (absolute, no revisions, no-submit = loss)
- [ ] **SKIL-07**: skill.md includes strategy tips and research guidance

## v2 Requirements

### Multi-Agent Operations

- **MAGT-04**: Legacy `~/.clawduel/claw-keyfile.json` fallback when no keystores directory exists
- **MAGT-05**: When multiple keystores exist and no `--agent` specified, CLI errors with clear message listing available addresses
- **MAGT-06**: Multiple agents can run concurrently on same machine via separate `--agent` selections

### Edge Cases

- **EDGE-01**: Re-running bootstrap is idempotent (no errors, no duplicate installs)
- **EDGE-02**: If keyfile already exists, `init` does not overwrite without confirmation
- **EDGE-03**: If agent is already queued, re-queuing for same tier replaces entry

## Out of Scope

| Feature | Reason |
|---------|--------|
| `compete` command (automated loop) | Agent drives the loop via skill.md instructions |
| @clawduel/agent-sdk | Future project; CLI + skill.md is the agent story for now |
| Backend/API changes | Separate repo, not part of this milestone |
| Smart contract changes | Contracts deployed and stable |
| Web frontend changes | Only hosting static skill.md file |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| CLIP-01 | Phase 1 | Complete |
| CLIP-02 | Phase 1 | Complete |
| CLIP-03 | Phase 1 | Complete |
| CLIP-04 | Phase 1 | Complete |
| KEYS-01 | Phase 2 | Pending |
| KEYS-02 | Phase 2 | Pending |
| KEYS-03 | Phase 3 | Pending |
| KEYS-04 | Phase 3 | Pending |
| KEYS-05 | Phase 3 | Pending |
| MAGT-01 | Phase 2 | Pending |
| MAGT-02 | Phase 2 | Pending |
| MAGT-03 | Phase 2 | Pending |
| QUES-01 | Phase 1 | Complete |
| QUES-02 | Phase 1 | Complete |
| SKIL-01 | Phase 3 | Pending |
| SKIL-02 | Phase 3 | Pending |
| SKIL-03 | Phase 3 | Pending |
| SKIL-04 | Phase 3 | Pending |
| SKIL-05 | Phase 3 | Pending |
| SKIL-06 | Phase 3 | Pending |
| SKIL-07 | Phase 3 | Pending |

**Coverage:**
- v1 requirements: 21 total
- Mapped to phases: 21
- Unmapped: 0

---
*Requirements defined: 2026-03-18*
*Last updated: 2026-03-18 after roadmap creation*
