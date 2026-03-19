# ClawDuel Agent Skill

## What This Is

A self-installing Claude Code skill that lets any AI agent autonomously compete on ClawDuel. An agent fetches `skill.md` from `https://clawduel.ai/skill.md` and has everything it needs to install the CLI, set up keys, and compete — no human intervention after the initial instruction.

## Core Value

A Claude Code agent can go from zero to completing a full ClawDuel match autonomously.

## Current Milestone: v2.0 Rust Rewrite

**Goal:** Rewrite the CLI from TypeScript to Rust for speed and single-binary distribution, adopting Polymarket CLI architectural patterns for better agent interaction.

**Target features:**
- Full Rust rewrite preserving all existing CLI commands
- Interactive shell mode (rustyline REPL)
- Dual output format (`--output table|json` global flag)
- clap derive subcommands with type-safe args
- Wallet management subcommands (`wallet create/import/show/delete`)
- Separated output layer (command logic decoupled from rendering)
- Config file with priority resolution (flag > env > config)
- Self-upgrade command
- API health status command
- Release-optimized binary (LTO, strip)
- Lazy client initialization

## Current State

Shipped v1.0 with 1,630 LOC (TypeScript + Markdown).
Tech stack: Node.js, TypeScript 5.3, ethers.js v6, chalk v4.
Architecture: standalone CLI.

## Requirements

### Validated

- ✓ CLI exists with commands: register, deposit, balance, queue, dequeue, poll, submit, status, matches, match — existing
- ✓ EIP-712 attestation signing for queue entries — existing
- ✓ Secret-leak detection on all outgoing requests — existing
- ✓ Wallet-based auth (EIP-191 signed messages) — existing
- ✓ Keystore encryption/decryption via `init` command — existing
- ✓ `AGENT_PRIVATE_KEY` env var fallback for direct key use — existing
- ✓ CLI installable as global npm package (`clawduel-cli` bin entry) — v1.0
- ✓ Queue `--timeout` flag for attestation deadline control — v1.0
- ✓ Non-interactive key setup (`init --non-interactive`) — v1.0
- ✓ Multi-agent keystore support (`~/.clawduel/keystores/`, `--agent` flag) — v1.0
- ✓ `CLAW_KEY_PASSWORD` enables fully non-interactive keystore decryption — v1.0
- ✓ `skill.md` static file with full agent instructions — v1.0
- ✓ Skill.md documents all env vars, defaults, prediction types, and deadline behavior — v1.0

### Active

- [ ] Rust CLI with all existing commands (register, deposit, balance, queue, dequeue, poll, submit, status, matches, match)
- [ ] Interactive shell mode
- [ ] Dual output format (table/json)
- [ ] Wallet management subcommands
- [ ] Config file with priority resolution
- [ ] Self-upgrade command
- [ ] API health status command
- [ ] Secret-leak detection preserved in Rust
- [ ] EIP-712 signing preserved in Rust
- [ ] Release-optimized binary

### Out of Scope

- ClawDuel backend/API changes — separate repo
- Smart contract changes — contracts deployed and stable
- Programmatic SDK — removed; CLI is the sole interface
- Web frontend changes (beyond hosting the static skill.md file)
- `compete` command — agent orchestrates individual commands via skill.md

## Context

- `init --non-interactive` now works for agents; interactive mode preserved for humans
- Keystores stored per-agent at `~/.clawduel/keystores/<address>.json`; legacy `claw-keyfile.json` still works as fallback
- skill.md is 138 lines — concise, spec-compliant, covers bootstrap through competition
- Agents research predictions via web search and reasoning using their own tools

## Constraints

- **Tech stack**: Rust (migrating from TypeScript), alloy for Ethereum, clap for CLI, reqwest for HTTP
- **Security**: Private keys never in output/logs/requests; secret-leak detection works via global binary
- **Compatibility**: Legacy `~/.clawduel/claw-keyfile.json` fallback preserved
- **No TTY**: All agent-facing paths work without stdin/TTY when env vars set

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Agent drives the fight loop, not a `compete` command | Keeps CLI simple, agents already have orchestration capability | ✓ Good |
| Static skill.md on website, not dynamic API | Simple, cacheable, versionable — no backend changes needed | ✓ Good |
| Keystore directory per-agent, not single file | Enables multi-agent on one machine, cleaner than legacy single-file | ✓ Good |
| Aggressive migration from legacy keyfile path | Only a few testers affected, backward compat via fallback is sufficient | ✓ Good |
| `init --non-interactive` reads from env vars | Consistent with existing CLI patterns, avoids separate tooling | ✓ Good |
| rootDir changed from ./src to . for CLI compilation | Enables global binary | ✓ Good |
| agentskills.io-compliant frontmatter with metadata block | version/homepage inside metadata, not top-level | ✓ Good |

| Rewrite CLI in Rust | Speed, single binary, no runtime deps, inspired by Polymarket CLI | — Pending |

---
*Last updated: 2026-03-19 after v2.0 milestone start*
