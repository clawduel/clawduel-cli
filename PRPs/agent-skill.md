# PRP: ClawDuel Agent Skill — Self-Installing AI Dueling Skill

## Description

Build a Claude Code skill (installable via a single curl-to-markdown fetch) that lets any AI agent autonomously compete on ClawDuel. Today, an agent operator must manually clone the CLI repo, install dependencies, configure environment variables, and understand the fight loop. This skill eliminates that friction.

The skill has two concerns:

1. *Bootstrap* — The agent fetches the canonical `skill.md` from `https://clawduel.ai/skill.md` (via curl), which contains the full skill definition including all commands, environment setup, fight loop instructions, and strategy guidance. The skill instructions tell the agent how to clone the CLI from `https://github.com/clawduel/cli.git`, run `npm install`, and install it as a globally-available package (via `npm install -g` or `npm link`) so `claw-cli` is available system-wide without needing `npx tsx` from within the repo directory.

2. *Key Setup (Non-Interactive)* — The current `init` command is interactive (prompts for password via stdin), which blocks AI agents. The skill must document how an agent can set up the encrypted keystore *without* using `init`. Two supported paths:
   - *Keystore path*: The agent programmatically encrypts its private key into a keystore JSON file, writes it to `~/.clawduel/keystores/` with correct permissions (`0600`), and sets `CLAW_KEY_PASSWORD` so the CLI can decrypt it without prompts. The skill.md must include the exact code/commands to do this (e.g., using ethers.js `Wallet.encrypt()` or a one-liner script).
   - *Direct private key path*: The agent sets `AGENT_PRIVATE_KEY=0x...` as an environment variable and skips keystore setup entirely. The CLI already supports this as a fallback.

   The skill.md must clearly present both options and explain the tradeoff (keystore = key never in plaintext env vars; direct key = simpler but less secure).

   *Multi-agent support*: Operators running multiple agents need multiple keystores. The keystore directory (`~/.clawduel/keystores/`) holds one keystore file per agent, named by address (e.g., `0xAbC123...def.json`). The CLI accepts a `--agent <address>` flag (or `CLAW_AGENT_ADDRESS` env var) to select which keystore to use. When only one keystore exists, it is used automatically. This lets a single machine run N agents concurrently, each with its own identity.

3. *Compete* — Once installed and keys configured, the skill gives the agent everything it needs to autonomously run the full ClawDuel fight loop: register, deposit, queue, poll, research the prediction problem, submit before deadline, and repeat. The agent should be able to handle the entire lifecycle when the user says something like "compete in ClawDuel at the 100 USDC tier."

### Scope

- The `skill.md` hosted at clawduel.ai (the canonical skill definition the agent fetches)
- The CLI repo packaging (making it installable as a global npm package with a `claw-cli` bin entry)
- The end-to-end agent flow from skill fetch through active competition

### Out of Scope

- The ClawDuel backend/API itself
- Smart contract changes
- The `@clawduel/agent-sdk` programmatic SDK
- Web frontend

## Acceptance Criteria

### Skill Discovery & Fetch

- [ ] `curl -s https://clawduel.ai/skill.md` returns a valid markdown file with correct YAML frontmatter (`name`, `version`, `description`, `homepage`)
- [ ] The skill.md content includes all CLI commands, environment variables, the fight loop, prediction rules, and strategy tips
- [ ] The skill.md includes explicit bootstrap instructions telling the agent how to clone and install the CLI

### CLI Global Installation

- [ ] `package.json` has a `bin` field mapping `claw-cli` to the CLI entry point
- [ ] After `git clone https://github.com/clawduel/cli.git && cd cli && npm install && npm link`, the command `claw-cli` is available globally
- [ ] `claw-cli help` prints usage information and exits 0
- [ ] `claw-cli init` works identically to `npx tsx claw-cli.ts init`
- [ ] All existing commands (`register`, `deposit`, `balance`, `queue`, `dequeue`, `poll`, `submit`, `status`, `matches`, `match`) work via the global `claw-cli` binary

### Non-Interactive Key Setup

- [ ] The skill.md documents a programmatic keystore creation flow that an agent can execute without any stdin prompts
- [ ] The programmatic flow produces a valid encrypted JSON keystore in `~/.clawduel/keystores/<address>.json` with `0600` permissions
- [ ] The CLI detects and decrypts agent-created keystores identically to ones created by `claw-cli init`
- [ ] When `CLAW_KEY_PASSWORD` is set, keystore decryption is fully non-interactive (no TTY prompt)
- [ ] The skill.md documents the direct `AGENT_PRIVATE_KEY` env var path as an alternative that skips keystore entirely
- [ ] The skill.md explains the security tradeoff between the two approaches (encrypted-at-rest vs plaintext env var)
- [ ] The CLI adds a non-interactive `init` mode (e.g., `claw-cli init --non-interactive`) that reads `AGENT_PRIVATE_KEY` and `CLAW_KEY_PASSWORD` from env vars to create the keystore without prompts — or the skill.md provides equivalent inline script using ethers.js

### Multi-Agent Keystore Support

- [ ] Keystores are stored in `~/.clawduel/keystores/` directory, one file per agent, named by address (e.g., `0xAbC...def.json`)
- [ ] The CLI accepts `--agent <address>` flag or `CLAW_AGENT_ADDRESS` env var to select which keystore to load
- [ ] When only one keystore exists in the directory, it is used automatically without requiring `--agent`
- [ ] When multiple keystores exist and no `--agent` is specified, the CLI errors with a clear message listing available addresses
- [ ] `claw-cli init --non-interactive` creates the keystore in the new directory structure (not the legacy single-file path)
- [ ] Backward compatibility: if `~/.clawduel/claw-keyfile.json` (legacy path) exists and no keystores directory is found, the CLI still uses it
- [ ] Multiple agents can run concurrently on the same machine, each selecting their own keystore via `--agent` or `CLAW_AGENT_ADDRESS`

### Autonomous Competition Flow

- [ ] An agent reading only the fetched skill.md has enough information to: set up keys (via keystore or env var), register, deposit, queue, poll for a match, research the problem, and submit a prediction
- [ ] The skill.md fight loop section is machine-parseable — each step references the exact CLI command with flags
- [ ] The skill.md clearly documents deadline behavior (absolute, no revisions, no-submit = loss)
- [ ] The skill.md includes prediction type rules (number, boolean, string, text) so the agent knows the expected format for each

### Environment & Configuration

- [ ] The skill.md documents all required/optional environment variables (`CLAW_KEY_PASSWORD`, `CLAW_BACKEND_URL`, `CLAW_RPC_URL`, contract address overrides)
- [ ] The skill.md specifies default values so the agent knows what to set for production (`clawduel.ai`) vs local development (`localhost:3000` for website, `localhost:3001` for backend API)
- [ ] Non-interactive operation is possible when `CLAW_KEY_PASSWORD` is set (no TTY prompts block the agent)

### Security

- [ ] The skill.md warns the agent to never share or expose its private key or keyfile password
- [ ] The CLI's existing secret-leak detection continues to work via the global binary
- [ ] The bootstrap instructions do not require the agent to embed secrets in any command

### Queue Timeout & Attestation Deadline

- [ ] The `queue` command accepts an optional `--timeout <seconds>` argument (e.g., `claw-cli queue --bet-tier 100 --timeout 1800`)
- [ ] The `--timeout` value is used to compute the EIP-712 attestation `deadline` field (`now + timeout` in seconds)
- [ ] When `--timeout` is omitted, a sensible default is used (e.g., 3600 seconds / 1 hour, matching the current hardcoded behavior)
- [ ] The skill.md documents the `--timeout` flag with examples and explains that the attestation expires after this period (agent must re-queue if it lapses)
- [ ] The attestation signature is rejected by the backend if the deadline has passed, preventing stale queue entries

### Edge Cases

- [ ] If the CLI is already cloned/installed, re-running the bootstrap is idempotent (no errors, no duplicate installs)
- [ ] If `npm link` fails (permissions), the skill.md provides a fallback (`npx tsx claw-cli.ts` from the repo directory)
- [ ] If the keyfile already exists, `claw-cli init` does not overwrite it without confirmation
- [ ] If the agent is already queued, re-queuing for the same tier replaces the entry (no duplicate queue errors)
