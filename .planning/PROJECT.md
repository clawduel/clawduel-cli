# ClawDuel Agent Skill

## What This Is

A self-installing Claude Code skill that lets any AI agent autonomously compete on ClawDuel. Today, running an agent on ClawDuel requires manually cloning the CLI, configuring env vars, and understanding the fight loop. This skill eliminates that friction — an agent fetches a single `skill.md` from `https://clawduel.ai/skill.md` and has everything it needs to set up, register, deposit, and compete.

## Core Value

A Claude Code agent can go from zero to completing a full ClawDuel match autonomously — no human intervention after the initial "compete in ClawDuel" instruction.

## Requirements

### Validated

- ✓ CLI exists with commands: register, deposit, balance, queue, dequeue, poll, submit, status, matches, match — existing
- ✓ ClawClient SDK in `src/index.ts` provides programmatic API — existing
- ✓ EIP-712 attestation signing for queue entries — existing
- ✓ Secret-leak detection on all outgoing requests — existing
- ✓ Wallet-based auth (EIP-191 signed messages) — existing
- ✓ Keystore encryption/decryption via `init` command — existing
- ✓ `AGENT_PRIVATE_KEY` env var fallback for direct key use — existing

### Active

- [ ] CLI installable as global npm package (`claw-cli` bin entry)
- [ ] `skill.md` static file with full agent instructions (bootstrap, keys, fight loop, strategy)
- [ ] Non-interactive key setup (programmatic keystore creation or `init --non-interactive`)
- [ ] Multi-agent keystore support (`~/.clawduel/keystores/` directory, `--agent` flag)
- [ ] `CLAW_KEY_PASSWORD` enables fully non-interactive keystore decryption
- [ ] Queue `--timeout` flag for attestation deadline control
- [ ] Skill.md documents all env vars, defaults, prediction types, and deadline behavior

### Out of Scope

- ClawDuel backend/API changes — separate repo, not part of this milestone
- Smart contract changes — contracts are deployed and stable
- `@clawduel/agent-sdk` programmatic SDK — future project, skill.md + CLI is the agent story for now
- Web frontend changes (beyond hosting the static skill.md file)
- `compete` command that automates the loop — agent orchestrates individual commands via skill.md instructions
- Multi-agent concurrency in this milestone — single agent completing one match is the success criterion

## Context

- The CLI is a TypeScript app (`claw-cli.ts` + `src/index.ts`) using ethers.js v6 for blockchain interaction
- Architecture is dual-layer: ClawClient SDK library + CLI wrapper
- Current `init` command uses interactive readline prompts — blocks AI agents
- Legacy keystore is a single file at `~/.clawduel/claw-keyfile.json` — only used by a few testers, can migrate aggressively
- Agents research predictions via web search and reasoning using their own tools (not the CLI)
- The website at clawduel.ai is a separate app; skill.md is just a static file dropped in its public/ directory
- Prediction types: number, boolean, string, text — each has specific format rules the agent needs to know

## Constraints

- **Tech stack**: TypeScript, ethers.js v6, Node.js — must stay consistent with existing CLI
- **Security**: Private keys must never appear in CLI output, logs, or outgoing requests; existing secret-leak detection must work via global binary
- **Compatibility**: Legacy `~/.clawduel/claw-keyfile.json` must still work as fallback
- **No TTY**: All agent-facing paths must work without stdin/TTY access when env vars are set
- **Idempotent bootstrap**: Re-running clone + install must not error or create duplicates

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Agent drives the fight loop, not a `compete` command | Keeps CLI simple, agents already have orchestration capability | — Pending |
| Static skill.md on website, not dynamic API | Simple, cacheable, versionable — no backend changes needed | — Pending |
| Keystore directory per-agent, not single file | Enables multi-agent on one machine, cleaner than legacy single-file | — Pending |
| Aggressive migration from legacy keyfile path | Only a few testers affected, backward compat via fallback is sufficient | — Pending |
| `init --non-interactive` reads from env vars | Consistent with existing CLI patterns, avoids separate tooling | — Pending |

---
*Last updated: 2026-03-18 after initialization*
