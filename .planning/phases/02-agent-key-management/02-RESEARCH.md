# Phase 2: Agent Key Management - Research

**Researched:** 2026-03-18
**Domain:** Ethereum wallet keystore management, CLI non-interactive operation, multi-agent file management
**Confidence:** HIGH

## Summary

Phase 2 transforms the existing interactive `init` command and `loadWallet()` flow into a fully non-interactive, multi-agent keystore system. The current codebase already uses `ethers.Wallet.encrypt()` and `ethers.Wallet.fromEncryptedJson()` (ethers.js v6.16.0), writes to `~/.clawduel/keyfile.json`, and reads `CLAW_KEY_PASSWORD` for non-interactive decryption. The changes are primarily about (1) adding a `--non-interactive` flag to `init` that reads both key and password from env vars, (2) moving keystore storage from a single `keyfile.json` to `~/.clawduel/keystores/<address>.json`, and (3) adding `--agent <address>` / `CLAW_AGENT_ADDRESS` selection logic with auto-select when only one keystore exists.

This is a well-scoped refactor of existing patterns. The ethers.js APIs are already in use and verified working. The main complexity is in the `loadWallet()` function which must gain keystore discovery/selection logic while maintaining backward compatibility.

**Primary recommendation:** Implement in a single plan with 3-4 tasks: (1) non-interactive init with keystores directory, (2) keystore discovery and selection in loadWallet, (3) --agent flag parsing and help text updates.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| KEYS-01 | `claw-cli init --non-interactive` reads `AGENT_PRIVATE_KEY` and `CLAW_KEY_PASSWORD` from env vars to create keystore without prompts | ethers.js `Wallet.encrypt()` already used in `cmdInit()` (line 232); just need to skip `promptLine()` calls when `--non-interactive` flag is present and env vars are set |
| KEYS-02 | When `CLAW_KEY_PASSWORD` is set, keystore decryption is fully non-interactive across all commands | `loadWallet()` already reads `CLAW_KEY_PASSWORD` at line 252: `process.env.CLAW_KEY_PASSWORD \|\| await promptLine(...)` -- this already works for single keyfile, needs extension for keystores directory |
| MAGT-01 | Keystores stored in `~/.clawduel/keystores/` directory, one file per agent named by address | New directory `~/.clawduel/keystores/`; files named `<address>.json` where address is lowercase hex without 0x prefix (matches ethers keystore format) or with 0x prefix for readability |
| MAGT-02 | CLI accepts `--agent <address>` flag or `CLAW_AGENT_ADDRESS` env var to select which keystore | New flag parsing in `main()` before `loadWallet()` call; pass selected address to loadWallet |
| MAGT-03 | When only one keystore exists, it is used automatically without requiring `--agent` | Directory listing in loadWallet: if `keystores/` has exactly 1 `.json` file, use it |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| ethers | 6.16.0 | Wallet encryption/decryption, key validation | Already in use; `Wallet.encrypt()` and `Wallet.fromEncryptedJson()` are the standard ethers keystore APIs |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| fs (built-in) | Node.js | File I/O for keystores | Already imported in claw-cli.ts |
| path (built-in) | Node.js | Path construction | Already imported |
| os (built-in) | Node.js | Home directory resolution | Already imported |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| ethers Wallet.encrypt | web3.js accounts.encrypt | No reason to switch; ethers already in stack |
| fs.readdirSync | glob patterns | Overkill for listing .json files in a single directory |

**Installation:**
No new dependencies required. Everything needed is already in the project.

## Architecture Patterns

### Current Keystore Flow (Before)
```
~/.clawduel/
└── keyfile.json          # Single keyfile, path from CLAW_KEYFILE or default
```

### Target Keystore Flow (After)
```
~/.clawduel/
├── keyfile.json          # Legacy fallback (NOT part of Phase 2 scope, but don't break it)
└── keystores/            # MAGT-01: New multi-agent directory
    ├── 0x19e7e376e7c213b7e7e7e46cc70a5dd086daff2a.json
    └── 0xabcdef1234567890abcdef1234567890abcdef12.json
```

### Pattern 1: Non-Interactive Init (KEYS-01)
**What:** When `--non-interactive` flag is present, read `AGENT_PRIVATE_KEY` and `CLAW_KEY_PASSWORD` from environment, skip all `promptLine()` calls, error if either is missing.
**When to use:** AI agent automation, CI/CD, headless environments.
**Example:**
```typescript
// Source: existing cmdInit() pattern in claw-cli.ts lines 204-241
async function cmdInit(args: string[]) {
  const nonInteractive = args.includes('--non-interactive');

  let privateKey: string;
  let password: string;

  if (nonInteractive) {
    privateKey = process.env.AGENT_PRIVATE_KEY || '';
    password = process.env.CLAW_KEY_PASSWORD || '';
    if (!privateKey) {
      log.error('AGENT_PRIVATE_KEY env var required for --non-interactive mode');
      process.exit(1);
    }
    if (!password) {
      log.error('CLAW_KEY_PASSWORD env var required for --non-interactive mode');
      process.exit(1);
    }
  } else {
    privateKey = process.env.AGENT_PRIVATE_KEY || await promptLine('Paste your private key: ');
    password = await promptLine('Enter encryption password (will not be hidden): ');
  }

  // Validate key
  let tempWallet: ethers.Wallet;
  try {
    tempWallet = new ethers.Wallet(privateKey.trim());
  } catch {
    log.error('Invalid private key format. Aborting.');
    process.exit(1);
  }

  // Encrypt and save to keystores directory
  const encrypted = await tempWallet.encrypt(password);
  const keystoresDir = path.join(os.homedir(), '.clawduel', 'keystores');
  fs.mkdirSync(keystoresDir, { recursive: true });

  const filename = `${tempWallet.address.toLowerCase()}.json`;
  const keystorePath = path.join(keystoresDir, filename);
  fs.writeFileSync(keystorePath, encrypted, { mode: 0o600 });

  log.success('Keystore saved to ' + keystorePath);
  log.field('Address', tempWallet.address);
  console.log(JSON.stringify({ ok: true, address: tempWallet.address, keystore: keystorePath }));
}
```

### Pattern 2: Keystore Discovery and Selection (MAGT-01, MAGT-02, MAGT-03)
**What:** `loadWallet()` discovers keystores in `~/.clawduel/keystores/`, applies selection logic.
**When to use:** Every command that requires a wallet (all commands except `init` and `help`).
**Example:**
```typescript
// Source: derived from existing loadWallet() at claw-cli.ts lines 243-280
const KEYSTORES_DIR = path.join(os.homedir(), '.clawduel', 'keystores');

function discoverKeystores(): string[] {
  if (!fs.existsSync(KEYSTORES_DIR)) return [];
  return fs.readdirSync(KEYSTORES_DIR)
    .filter(f => f.endsWith('.json'))
    .map(f => path.join(KEYSTORES_DIR, f));
}

function selectKeystore(agentAddress?: string): string {
  const keystores = discoverKeystores();

  if (keystores.length === 0) {
    // No keystores found -- fall through to legacy keyfile.json or env var
    throw new Error('NO_KEYSTORES');
  }

  if (agentAddress) {
    // MAGT-02: Explicit selection by address
    const normalized = agentAddress.toLowerCase();
    const match = keystores.find(k => path.basename(k, '.json').toLowerCase() === normalized);
    if (!match) {
      log.error(`No keystore found for agent ${agentAddress}`);
      log.dim(`Available: ${keystores.map(k => path.basename(k, '.json')).join(', ')}`);
      process.exit(1);
    }
    return match;
  }

  if (keystores.length === 1) {
    // MAGT-03: Auto-select when only one exists
    return keystores[0];
  }

  // Multiple keystores, no --agent specified
  log.error('Multiple keystores found. Specify which agent to use:');
  for (const k of keystores) {
    log.dim(`  --agent ${path.basename(k, '.json')}`);
  }
  process.exit(1);
}
```

### Pattern 3: Flag Parsing for --agent (MAGT-02)
**What:** Parse `--agent <address>` from args or read `CLAW_AGENT_ADDRESS` env var before loading wallet.
**When to use:** In `main()` before the `loadWallet()` call.
**Example:**
```typescript
// In main(), before loadWallet():
const agentIdx = args.indexOf('--agent');
const agentAddress = (agentIdx !== -1 && agentIdx + 1 < args.length)
  ? args[agentIdx + 1]
  : process.env.CLAW_AGENT_ADDRESS;

const loaded = await loadWallet(agentAddress);
```

### Anti-Patterns to Avoid
- **Do NOT rename existing `KEYFILE_PATH` constant or delete legacy keyfile support:** Other code may reference `~/.clawduel/keyfile.json`. Legacy path should continue working as fallback.
- **Do NOT use address from keystore JSON `address` field for matching:** The `address` field in ethers encrypted JSON is lowercase without 0x prefix. Use the filename as the canonical identifier instead, but normalize comparison.
- **Do NOT prompt in non-interactive mode:** If `--non-interactive` is set, any missing env var should be a hard error, never a prompt.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Wallet encryption | Custom AES encryption | `ethers.Wallet.encrypt()` | Uses scrypt KDF + AES-128-CTR, industry standard |
| Wallet decryption | Custom JSON parsing | `ethers.Wallet.fromEncryptedJson()` | Handles all keystore versions (v1, v3) |
| Key validation | Hex regex | `new ethers.Wallet(key)` | Validates curve point, not just format |
| Address checksumming | Manual EIP-55 | `ethers.getAddress()` | Handles mixed-case properly |

**Key insight:** ethers.js `Wallet.encrypt()` produces a standard Ethereum V3 keystore (scrypt KDF). The encrypted JSON includes the `address` field (lowercase, no 0x prefix) which can be used to derive the filename.

## Common Pitfalls

### Pitfall 1: Wallet.encrypt() is Slow
**What goes wrong:** `Wallet.encrypt()` uses scrypt with N=131072 by default. Takes 2-5 seconds.
**Why it happens:** Intentional -- scrypt is designed to be CPU/memory intensive to resist brute force.
**How to avoid:** This is expected behavior. Log a message ("Encrypting keyfile, this may take a moment...") as the current code already does.
**Warning signs:** If it takes > 10 seconds, something is wrong.

### Pitfall 2: Address Case Sensitivity in Keystore Files
**What goes wrong:** ethers encrypted JSON stores address as lowercase without 0x. `wallet.address` returns checksummed (mixed-case with 0x). File lookup breaks if comparing directly.
**Why it happens:** Different conventions for different contexts.
**How to avoid:** Always normalize to lowercase for file operations. Use `address.toLowerCase()` for filename. When matching `--agent` input, normalize both sides.
**Warning signs:** "No keystore found" when keystore clearly exists.

### Pitfall 3: --non-interactive Must Not Call promptLine
**What goes wrong:** In non-interactive mode, `promptLine()` hangs forever because there's no TTY.
**Why it happens:** readline waits for user input that never comes.
**How to avoid:** Check `--non-interactive` flag early, error immediately if env vars missing. Never reach promptLine code path.
**Warning signs:** Process hangs indefinitely during CI/agent automation.

### Pitfall 4: File Permissions on Keystores
**What goes wrong:** Keystore files created without restricted permissions are readable by other users.
**Why it happens:** Default file creation mode is 0o644.
**How to avoid:** Always use `{ mode: 0o600 }` in `writeFileSync`. Create directory with `0o700`.
**Warning signs:** `ls -la ~/.clawduel/keystores/` shows group/other read permissions.

### Pitfall 5: --agent Flag Consumed Before Command Args
**What goes wrong:** `--agent 0xABC --bet-tier 100` -- the `--agent` flag gets confused with command-specific args.
**Why it happens:** Current arg parsing is positional within the switch cases.
**How to avoid:** Parse `--agent` in `main()` before dispatching to command handlers. Remove it from args passed to handlers, or use a global pre-parse step.
**Warning signs:** "Missing required argument" errors when --agent is used with other flags.

## Code Examples

### Verified: Wallet.encrypt() Output Format
```typescript
// Verified against ethers.js v6.16.0 in this project
const wallet = new ethers.Wallet(privateKey);
const encrypted = await wallet.encrypt(password);
const parsed = JSON.parse(encrypted);
// parsed.address = "19e7e376e7c213b7e7e7e46cc70a5dd086daff2a" (no 0x, lowercase)
// parsed.id = UUID string
// parsed.version = 3
// parsed.Crypto = { cipher, cipherparams, ciphertext, kdf, kdfparams, mac }
```

### Verified: Wallet.fromEncryptedJson() Decryption
```typescript
// Source: already in claw-cli.ts line 256
const encryptedJson = fs.readFileSync(keystorePath, 'utf-8');
const decryptedWallet = await ethers.Wallet.fromEncryptedJson(encryptedJson, password);
const connectedWallet = decryptedWallet.connect(provider);
// decryptedWallet.address = "0x19E7E376E7C213B7E7e7e46cc70A5dD086DAff2A" (checksummed)
// decryptedWallet.privateKey = "0x1111..." (with 0x prefix)
```

### Keystore File Naming Convention
```typescript
// Use wallet.address (checksummed, with 0x) as filename for human readability
const filename = `${wallet.address.toLowerCase()}.json`;
// Result: "0x19e7e376e7c213b7e7e7e46cc70a5dd086daff2a.json"
```

### Directory Listing for Discovery
```typescript
// List all .json files in keystores directory
const files = fs.readdirSync(KEYSTORES_DIR).filter(f => f.endsWith('.json'));
// files = ["0x19e7e37...json", "0xabcdef...json"]
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Single keyfile.json | Multi-keystore directory | This phase | Enables multi-agent operation |
| Interactive-only init | --non-interactive flag | This phase | Enables agent automation |
| No agent selection | --agent flag + auto-select | This phase | Enables specific agent targeting |

**No deprecated APIs involved:** `Wallet.encrypt()` and `Wallet.fromEncryptedJson()` are stable ethers.js v6 APIs. No changes expected.

## Open Questions

1. **Filename convention: with or without 0x prefix?**
   - What we know: ethers keystore JSON stores address without 0x. `wallet.address` includes 0x.
   - Recommendation: Use `wallet.address.toLowerCase()` (with 0x) as filename for human readability. The 0x makes it clear this is an Ethereum address when browsing the directory. Normalize for matching.

2. **Should `init --non-interactive` overwrite an existing keystore for the same address?**
   - What we know: Requirements say EDGE-02 (overwrite protection) is v2. Phase 2 doesn't include it.
   - Recommendation: For now, overwrite silently (simplest). EDGE-02 is deferred to v2.

3. **Should the legacy `~/.clawduel/keyfile.json` path still work?**
   - What we know: MAGT-04 (legacy fallback) is explicitly v2. Phase 2 requirements don't include it.
   - Recommendation: Don't break it, but don't add explicit fallback logic either. The existing `CLAW_KEYFILE` env var and `KEYFILE_PATH` constant stay. `loadWallet()` should check keystores directory first, then fall through to existing keyfile logic.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Manual testing via CLI invocation |
| Config file | none -- no test framework configured |
| Quick run command | `npm run build && claw-cli help` |
| Full suite command | `npm run build` (build success is the gate) |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| KEYS-01 | `init --non-interactive` creates keystore from env vars | smoke | `AGENT_PRIVATE_KEY=0x$(openssl rand -hex 32) CLAW_KEY_PASSWORD=test claw-cli init --non-interactive && ls ~/.clawduel/keystores/` | N/A (manual) |
| KEYS-02 | `CLAW_KEY_PASSWORD` enables non-interactive decryption | smoke | `CLAW_KEY_PASSWORD=test claw-cli balance` (should not prompt) | N/A (manual) |
| MAGT-01 | Keystores stored in `~/.clawduel/keystores/` named by address | smoke | `ls ~/.clawduel/keystores/*.json` after init | N/A (manual) |
| MAGT-02 | `--agent <address>` selects keystore | smoke | `claw-cli balance --agent 0x...` | N/A (manual) |
| MAGT-03 | Auto-select when only one keystore exists | smoke | Remove all but one keystore, run `claw-cli balance` without --agent | N/A (manual) |

### Sampling Rate
- **Per task commit:** `npm run build` must succeed
- **Per wave merge:** Build + manual smoke test of init --non-interactive
- **Phase gate:** All 5 requirements manually verified

### Wave 0 Gaps
- No automated test infrastructure exists for this project (no test framework, no test files)
- Manual smoke testing is the established pattern per CLAUDE.md: "Manual testing via battle scripts"
- Adding a test framework is out of scope for this phase

## Sources

### Primary (HIGH confidence)
- **Project source code** (`claw-cli.ts` lines 204-280): Existing init and loadWallet implementation
- **ethers.js v6.16.0** (verified locally): `Wallet.encrypt()`, `Wallet.fromEncryptedJson()` API confirmed working
- **Encrypted keystore format** (verified locally): `{ address: "hex_no_0x", id: "uuid", version: 3, Crypto: {...} }`

### Secondary (MEDIUM confidence)
- ethers.js documentation on wallet encryption: APIs are stable across v6.x releases

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - ethers.js v6.16.0 already in use, APIs verified locally
- Architecture: HIGH - straightforward file system operations with well-understood patterns
- Pitfalls: HIGH - identified from direct code analysis and API testing

**Research date:** 2026-03-18
**Valid until:** 2026-04-18 (stable domain, no fast-moving dependencies)
