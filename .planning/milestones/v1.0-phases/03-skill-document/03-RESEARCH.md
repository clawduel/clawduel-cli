# Phase 3: Skill Document - Research

**Researched:** 2026-03-18
**Domain:** Claude Code Agent Skills / Technical Documentation
**Confidence:** HIGH

## Summary

Phase 3 is a pure documentation phase: authoring a `skill.md` file that enables a Claude Code agent to autonomously bootstrap, configure, and compete in ClawDuel matches. No code changes are needed. The existing `skill.md` already has partial content but needs significant revision to meet all SKIL-* and KEYS-03/04/05 requirements.

The key finding is that the existing `skill.md` has non-standard YAML frontmatter (uses top-level `version` and `homepage` fields which are not part of the Agent Skills specification). The agentskills.io spec only supports `name`, `description`, `license`, `compatibility`, `metadata`, and `allowed-tools` as frontmatter fields. The `version` field belongs inside a `metadata` block, and `homepage` can go there too. Additionally, the existing skill.md still uses `npx tsx clawduel-cli.ts` command format instead of the global `clawduel-cli` binary established in Phase 1, and is missing key sections required by the phase requirements.

**Primary recommendation:** Rewrite skill.md from scratch using the agentskills.io frontmatter spec, the global `clawduel-cli` binary commands, comprehensive env var documentation, both key management paths with security tradeoffs, and a step-by-step fight loop with exact CLI commands.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| SKIL-01 | skill.md has valid YAML frontmatter (name, version, description, homepage) | Frontmatter format research: `name` and `description` are top-level required fields; `version` and `homepage` go inside `metadata` block per agentskills.io spec |
| SKIL-02 | Bootstrap instructions (clone, install, npm link, fallback for permission errors) | Codebase has `prepare` script running `npm run build`; `npm link` may need `sudo` on some systems |
| SKIL-03 | Complete fight loop with exact CLI commands per step | All CLI commands documented from clawduel-cli.ts: init, register, deposit, queue, poll, submit, status, matches, match |
| SKIL-04 | All env vars with defaults (prod: clawduel.ai, local: localhost) | 8 env vars identified from CLI source with their defaults |
| SKIL-05 | Prediction type rules (number, boolean, string, text) with expected formats | Existing skill.md has basic rules; need to verify exact formats from backend expectations |
| SKIL-06 | Deadline behavior (absolute, no revisions, no-submit = loss) | Existing skill.md covers this; needs refinement |
| SKIL-07 | Strategy tips and research guidance | Existing skill.md has basic tips; needs expansion |
| KEYS-03 | Programmatic keystore creation flow using ethers.js Wallet.encrypt() | Code in cmdInit() shows exact pattern: `new ethers.Wallet(key)` then `wallet.encrypt(password)` |
| KEYS-04 | Direct AGENT_PRIVATE_KEY env var path as alternative | loadWallet() fallback path clearly shows this flow |
| KEYS-05 | Security tradeoff between encrypted-at-rest keystore and plaintext env var | Need to document: keystore = encrypted at rest but needs password; env var = plaintext in memory/process table |
</phase_requirements>

## Standard Stack

This phase is documentation-only. No libraries needed. The skill.md file is a Markdown document with YAML frontmatter.

### Relevant Specifications
| Spec | Source | Purpose |
|------|--------|---------|
| Agent Skills Spec | agentskills.io/specification | YAML frontmatter format and file structure |
| Claude Code Skills Docs | code.claude.com/docs/en/skills | How Claude Code discovers and uses skills |

## Architecture Patterns

### skill.md File Structure

Per the agentskills.io specification, the skill.md must follow this structure:

```markdown
---
name: clawduel
description: [max 1024 chars describing what the skill does and when to use it]
metadata:
  version: "2.0.0"
  homepage: https://clawduel.ai
---

[Markdown body with instructions]
```

### YAML Frontmatter Fields (agentskills.io spec)

| Field | Required | Notes |
|-------|----------|-------|
| `name` | Yes | Max 64 chars, lowercase letters/numbers/hyphens only, must match parent directory name |
| `description` | Yes | Max 1024 chars, should describe what skill does AND when to use it |
| `license` | No | License name or reference |
| `compatibility` | No | Max 500 chars, environment requirements |
| `metadata` | No | Arbitrary key-value mapping -- put `version` and `homepage` here |
| `allowed-tools` | No | Space-delimited list of pre-approved tools |

**Critical finding:** The requirement SKIL-01 says "YAML frontmatter (name, version, description, homepage)" but the agentskills.io spec does NOT support top-level `version` or `homepage` fields. They belong in `metadata`. The planner should use the spec-compliant approach: `name` and `description` at top level, `version` and `homepage` inside `metadata`.

### Recommended Sections for skill.md Body

Based on requirements analysis, the skill.md body should contain these sections in order:

1. **Title and Overview** -- What ClawDuel is (concise, Claude already knows general concepts)
2. **Bootstrap** (SKIL-02) -- Clone, install, npm link, permission error fallback
3. **Key Setup** (KEYS-03, KEYS-04, KEYS-05) -- Both paths with security tradeoff
4. **Environment Variables** (SKIL-04) -- All env vars with defaults for both prod and local
5. **Fight Loop** (SKIL-03) -- Step-by-step with exact CLI commands
6. **Prediction Rules** (SKIL-05) -- Format rules per type
7. **Deadline Behavior** (SKIL-06) -- Critical rules about timing
8. **Strategy Tips** (SKIL-07) -- Research guidance and tips

### Anti-Patterns to Avoid
- **Over-explaining things Claude already knows:** Don't explain what USDC is or how Ethereum works. Be concise per skill authoring best practices.
- **Using old command format:** Must use `clawduel-cli` not `npx tsx clawduel-cli.ts` -- Phase 1 established the global binary.
- **Non-standard frontmatter:** Don't put `version`/`homepage` at the top level.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Frontmatter validation | Custom YAML parser | agentskills.io spec compliance | Spec is well-defined, just follow it |

## Common Pitfalls

### Pitfall 1: Non-Standard YAML Frontmatter
**What goes wrong:** Existing skill.md uses top-level `version` and `homepage` which are not in the agentskills.io spec. Some validators or tools may reject this.
**Why it happens:** The original skill.md was written before the spec was finalized or without checking the spec.
**How to avoid:** Use `metadata` block for non-standard fields. Keep only `name` and `description` at top level.
**Warning signs:** YAML validation errors, skill not loading in Claude Code.

### Pitfall 2: Stale Command Format
**What goes wrong:** Existing skill.md uses `npx tsx clawduel-cli.ts` instead of `clawduel-cli` global binary.
**Why it happens:** skill.md was written before Phase 1 made the CLI globally installable.
**How to avoid:** Use `clawduel-cli <command>` throughout. This is what agents will use after bootstrap.
**Warning signs:** Agent tries to run `npx tsx clawduel-cli.ts` and fails because tsx is not installed.

### Pitfall 3: Missing Production Defaults
**What goes wrong:** Env var defaults point to localhost only, agent doesn't know production URLs.
**Why it happens:** CLI code defaults to localhost for dev. Production URL (clawduel.ai) is not in the CLI source.
**How to avoid:** Document both sets of defaults explicitly in skill.md. The requirement says "prod: clawduel.ai, local: localhost".
**Warning signs:** Agent connects to localhost in production environment.

### Pitfall 4: npm link Permission Errors
**What goes wrong:** `npm link` fails on some systems without sudo/admin.
**Why it happens:** Global npm installs require elevated permissions on some setups.
**How to avoid:** Document both `npm link` and a fallback (`sudo npm link` or `npx` alternative).
**Warning signs:** Permission denied errors during bootstrap.

### Pitfall 5: Verbose Skill Content
**What goes wrong:** Skill.md exceeds 500 lines, consuming too much context window.
**Why it happens:** Over-explaining concepts Claude already knows.
**How to avoid:** Be concise. Focus on project-specific information only. Per best practices: "Only add context Claude doesn't already have."
**Warning signs:** Skill.md approaching or exceeding 500 lines.

## Code Examples

### Current CLI Commands (from clawduel-cli.ts source)

All commands use the `clawduel-cli` global binary after Phase 1 bootstrap:

```bash
# No-wallet commands
clawduel-cli help
clawduel-cli init [--non-interactive]

# Wallet-required commands (need keystore or AGENT_PRIVATE_KEY)
clawduel-cli register --nickname <name>
clawduel-cli deposit --amount <usdc_amount>
clawduel-cli balance
clawduel-cli queue --bet-tier <10|100|1000|10000|100000> [--timeout <seconds>]
clawduel-cli dequeue --bet-tier <10|100|1000|10000|100000>
clawduel-cli poll
clawduel-cli submit --match-id <id> --prediction <value>
clawduel-cli status
clawduel-cli matches [--status <filter>] [--page <n>] [--category <cat>] [--from <ISO>] [--to <ISO>]
clawduel-cli match --id <matchId>

# Global option for multi-agent setups
--agent <address>   (or CLAW_AGENT_ADDRESS env var)
```

### All Environment Variables (from clawduel-cli.ts source)

| Variable | Purpose | Default (local) | Default (prod) |
|----------|---------|-----------------|----------------|
| `AGENT_PRIVATE_KEY` | Fallback private key (if no keystore) | none | none |
| `CLAW_KEY_PASSWORD` | Keystore decryption password | none (prompts) | none (prompts) |
| `CLAW_AGENT_ADDRESS` | Select keystore by address | none | none |
| `CLAW_BACKEND_URL` | Backend API URL | `http://localhost:3001` | `https://clawduel.ai` |
| `CLAW_RPC_URL` | Ethereum RPC URL | `http://localhost:8545` | (chain-specific) |
| `CLAW_BANK_ADDRESS` | Bank contract address | hardcoded default | (deployment-specific) |
| `CLAW_CLAWDUEL_ADDRESS` | ClawDuel contract address | hardcoded default | (deployment-specific) |
| `CLAW_USDC_ADDRESS` | USDC contract address | hardcoded default | (deployment-specific) |
| `CLAW_KEYFILE` | Legacy keyfile path override | `~/.clawduel/keyfile.json` | same |

**Note:** The production default for `CLAW_BACKEND_URL` is specified in the requirements as `clawduel.ai` but the CLI source code defaults to `http://localhost:3001`. The skill.md must explicitly document the production URL.

### Programmatic Keystore Creation (KEYS-03)

From clawduel-cli.ts `cmdInit()` (lines 209-265):

```typescript
// 1. Create wallet from private key
const tempWallet = new ethers.Wallet(privateKey.trim());

// 2. Encrypt with password (this is ethers.js Wallet.encrypt())
const encrypted = await tempWallet.encrypt(password);

// 3. Write to keystores directory
fs.mkdirSync(KEYSTORES_DIR, { recursive: true, mode: 0o700 });
const filename = `${tempWallet.address.toLowerCase()}.json`;
const keystorePath = path.join(KEYSTORES_DIR, filename);
fs.writeFileSync(keystorePath, encrypted, { mode: 0o600 });
```

The non-interactive flow uses env vars:
```bash
AGENT_PRIVATE_KEY=0x... CLAW_KEY_PASSWORD=mypassword clawduel-cli init --non-interactive
```

### Key Management Security Tradeoffs (KEYS-05)

| Path | Security | Convenience | Risk |
|------|----------|-------------|------|
| Encrypted keystore (`clawduel-cli init`) | Encrypted at rest with password | Must set `CLAW_KEY_PASSWORD` env var for non-interactive use | Password in env is still readable by same-user processes |
| `AGENT_PRIVATE_KEY` env var | Plaintext in process environment | No init step needed, just set env var | Key visible in process table (`/proc/PID/environ`), shell history, CI logs |

**Recommendation for skill.md:** Present keystore as the preferred path. Present AGENT_PRIVATE_KEY as the quick-start alternative with clear security warnings.

### Fight Loop Steps (SKIL-03)

The complete fight loop extracted from the codebase:

1. **Bootstrap** (once): `git clone`, `npm install`, `npm link`
2. **Init keystore** (once): `clawduel-cli init --non-interactive`
3. **Register** (once): `clawduel-cli register --nickname "AgentName"`
4. **Deposit USDC**: `clawduel-cli deposit --amount 100`
5. **Queue for match**: `clawduel-cli queue --bet-tier 10 [--timeout 3600]`
6. **Poll for match**: `clawduel-cli poll` (repeat until match object is non-null)
   - Poll handles the ready acknowledgement flow automatically (waiting_ready -> sends ready signal)
   - Poll handles waiting_start state (waits for synchronized start time)
7. **Read problem**: Parse the poll JSON response for `category`, `title`, `prompt`, `valueType`, `deadline`
8. **Research and reason**: Use available tools (web search, fetch, etc.)
9. **Submit prediction**: `clawduel-cli submit --match-id <id> --prediction "<value>"`
10. **Review results**: `clawduel-cli match --id <matchId>` or `clawduel-cli matches --status resolved`
11. **Repeat from step 5**

### Prediction Types (SKIL-05)

| Type | Format | Scoring |
|------|--------|---------|
| `number` | Numeric value, e.g. `67432.50` | Absolute error -- closest to actual wins |
| `boolean` | `yes` or `no` | Exact match wins |
| `string` | Exact text | Case-insensitive exact match |
| `text` | Free text | Scored by semantic similarity |

**Note:** The prediction is sanitized by `sanitizePrediction()` before submission: control chars removed, whitespace normalized, trimmed.

### Poll Response Structure

The poll command hits `/matches/active/<address>` and returns match data including:
- `match.status` -- `waiting_ready`, `waiting_start`, `active`, etc.
- `match.problem` -- null until match starts, then contains the prediction problem
- `match.readyUrl` -- URL to acknowledge readiness
- `match.startsAt` -- synchronized start time
- Fields in problem: `category`, `title`, `prompt`, `valueType`, `deadline`

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `npx tsx clawduel-cli.ts` | `clawduel-cli` global binary | Phase 1 (2026-03-18) | All commands use global binary |
| `~/.clawduel/keyfile.json` single file | `~/.clawduel/keystores/<address>.json` | Phase 2 (2026-03-18) | Multi-agent support |
| Interactive-only init | `clawduel-cli init --non-interactive` | Phase 2 (2026-03-18) | Agent can bootstrap without TTY |
| No `--agent` flag | `--agent <address>` / `CLAW_AGENT_ADDRESS` | Phase 2 (2026-03-18) | Multi-agent keystore selection |
| No `--timeout` flag on queue | `--timeout <seconds>` (default 3600) | Phase 1 (2026-03-18) | Configurable attestation deadline |

**Deprecated/outdated in existing skill.md:**
- Uses `npx tsx clawduel-cli.ts` commands throughout (should be `clawduel-cli`)
- References `~/.clawduel/keyfile.json` (should be `~/.clawduel/keystores/`)
- Missing `--non-interactive` init documentation
- Missing `--agent` and `--timeout` flags
- Missing programmatic keystore creation documentation (KEYS-03)

## Open Questions

1. **Production CLAW_BACKEND_URL**
   - What we know: Requirements say "prod: clawduel.ai", CLI defaults to localhost
   - What's unclear: Is it `https://clawduel.ai` or `https://api.clawduel.ai` or `https://clawduel.ai/api`?
   - Recommendation: Use `https://clawduel.ai` per requirements; planner can parameterize this

2. **Production RPC URL**
   - What we know: SDK defaults to `https://polygon-rpc.com`, CLI defaults to `http://localhost:8545`
   - What's unclear: What chain does production use? What RPC should agents use?
   - Recommendation: Document placeholder, note that production contract addresses will be provided

3. **Poll response exact JSON schema**
   - What we know: CLI code parses `data.match.status`, `data.match.readyUrl`, `data.match.problem`, `data.match.startsAt`
   - What's unclear: Full schema of the problem object (fields like `valueType`, `deadline` format)
   - Recommendation: Document the fields referenced in CLI code; note that actual format comes from backend

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Manual validation (no automated test framework in project) |
| Config file | none |
| Quick run command | `npm run build` (verifies no code was broken) |
| Full suite command | `npm run build` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SKIL-01 | Valid YAML frontmatter | manual | Visually inspect frontmatter against agentskills.io spec | N/A |
| SKIL-02 | Bootstrap instructions present | manual | Verify bootstrap section exists with clone/install/link commands | N/A |
| SKIL-03 | Complete fight loop with exact CLI commands | manual | Verify each CLI command in fight loop matches `clawduel-cli help` output | N/A |
| SKIL-04 | All env vars documented with defaults | manual | Cross-reference env vars in skill.md against clawduel-cli.ts source | N/A |
| SKIL-05 | Prediction type rules documented | manual | Verify all 4 types (number, boolean, string, text) with formats | N/A |
| SKIL-06 | Deadline behavior documented | manual | Verify deadline rules section exists | N/A |
| SKIL-07 | Strategy tips present | manual | Verify strategy section exists | N/A |
| KEYS-03 | Programmatic keystore creation documented | manual | Verify ethers.js Wallet.encrypt() flow is documented | N/A |
| KEYS-04 | AGENT_PRIVATE_KEY path documented | manual | Verify env var alternative path is documented | N/A |
| KEYS-05 | Security tradeoff explained | manual | Verify comparison table or explanation exists | N/A |

### Sampling Rate
- **Per task commit:** `npm run build` (ensure no accidental code changes)
- **Per wave merge:** Manual review of skill.md against all SKIL-* and KEYS-03/04/05 requirements
- **Phase gate:** All requirements checked in skill.md content review

### Wave 0 Gaps
None -- this phase is documentation-only, no test infrastructure needed.

## Sources

### Primary (HIGH confidence)
- agentskills.io/specification -- Complete YAML frontmatter spec with field requirements
- code.claude.com/docs/en/skills -- Claude Code skill discovery, frontmatter fields, best practices
- platform.claude.com/docs/en/agents-and-tools/agent-skills/best-practices -- Skill authoring guidelines (conciseness, structure, anti-patterns)
- clawduel-cli.ts source (local) -- All CLI commands, env vars, flags, help text
- src/index.ts source (local) -- SDK env vars, contract addresses, API methods

### Secondary (MEDIUM confidence)
- Existing skill.md (local) -- Partial content, needs updates for Phase 1/2 changes
- REQUIREMENTS.md (local) -- Requirement definitions including SKIL-01 frontmatter fields

### Tertiary (LOW confidence)
- Production URL (`clawduel.ai`) -- Referenced in requirements but not verified in code; planner should confirm

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - No libraries needed, just Markdown authoring
- Architecture: HIGH - agentskills.io spec is well-documented and verified via official docs
- Pitfalls: HIGH - Identified from direct source code analysis and spec comparison
- Content accuracy: HIGH for CLI commands/env vars (from source), MEDIUM for production defaults (from requirements only)

**Research date:** 2026-03-18
**Valid until:** 2026-04-18 (stable -- documentation phase, no fast-moving dependencies)
