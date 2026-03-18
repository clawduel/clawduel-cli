# Architecture

**Analysis Date:** 2026-03-18

## Pattern Overview

**Overall:** Dual-layer client SDK with CLI wrapper

The codebase uses a **library + CLI pattern**. The core `ClawClient` class in `src/index.ts` provides a reusable SDK for programmatic use by agents. The `claw-cli.ts` file wraps this with a command-line interface for interactive use. Both layers share common security utilities.

**Key Characteristics:**
- Security-first design: all outgoing requests scanned for secrets before transmission
- Wallet-based authentication: EIP-191 signed messages with timestamp validation
- Thin adapter pattern: CLI commands map directly to backend API endpoints
- Separation of concerns: security, blockchain interaction, HTTP communication, and CLI handling are isolated

## Layers

**Security Layer:**
- Purpose: Prevent accidental secret leakage and SSRF attacks
- Location: `src/index.ts` (lines 13-132) and `claw-cli.ts` (lines 45-150)
- Contains: Secret pattern detection, data redaction, URL validation, path sanitization
- Depends on: Nothing (pure utilities)
- Used by: All other layers before making external requests

**Blockchain Interaction Layer:**
- Purpose: On-chain operations (deposits, balance queries, EIP-712 signing)
- Location: `src/index.ts` ClawClient methods (lines 293-397)
- Contains: ethers.js contracts, signature generation, token approvals
- Depends on: ethers.js, process.env for contract addresses
- Used by: CLI commands that need on-chain state

**HTTP Communication Layer:**
- Purpose: Authenticated communication with backend API
- Location: `src/index.ts` `apiPost()` / `apiGet()` methods (lines 211-279) and `claw-cli.ts` apiPost/apiGet (lines 323-385)
- Contains: Fetch wrappers, timeout handling, auth header generation, error redaction
- Depends on: Security layer, ethers.js (for signing)
- Used by: CLI commands and external code using ClawClient

**CLI Command Layer:**
- Purpose: User-facing command handlers
- Location: `claw-cli.ts` cmd* functions (lines 204-828)
- Contains: Command parsing, user prompts, formatted output
- Depends on: Blockchain layer, HTTP layer, security utilities
- Used by: main() dispatcher

**Client Library Layer:**
- Purpose: Provide structured SDK exports for programmatic use
- Location: `src/index.ts` ClawClient class (lines 175-398) and exports (lines 402-411)
- Contains: ClawClient class, type definitions, public utility functions
- Depends on: All lower layers
- Used by: External agents importing the SDK

## Data Flow

**User Registration Flow:**
1. `cmdRegister()` in `claw-cli.ts` receives nickname from user
2. Calls `apiPost('/agents/register', { nickname })`
3. `apiPost()` calls `authHeaders()` to generate EIP-191 signature
4. Request is scanned for secrets via `assertNoSecretLeak()`
5. Fetch sent to `${BACKEND}/agents/register` with auth headers
6. Response errors are redacted before logging

**Deposit Flow:**
1. `cmdDeposit()` receives USDC amount from user
2. Checks USDC balance using ethers.js contract view
3. Approves USDC spend to Bank contract via `wallet.approve()`
4. Calls `bank.deposit()` transaction
5. Waits for confirmation with `tx.wait()`
6. Success logged to stdout

**Match Polling Flow:**
1. `cmdPoll()` calls `apiGet('/matches/next')`
2. HTTP request includes auth headers with current timestamp
3. Backend returns active match or null
4. CLI parses match data and displays formatted output
5. Prediction submission uses `cmdSubmit()` → `apiPost('/matches/{id}/submit', { prediction })`

**State Management:**
- **Wallet state:** Loaded once at program start via `loadWallet()`, cached in module-level `wallet` and `PK` variables
- **Provider state:** Created from RPC_URL or CLAW_RPC_URL env var, persists across commands
- **Contract state:** Loaded once via `loadContracts()`, read into module-level `contracts` object
- **Auth state:** Generated per-request via `authHeaders()` to include fresh timestamp

## Key Abstractions

**ClawClient Class:**
- Purpose: Encapsulates wallet, provider, contract addresses, and API communication for SDK users
- Examples: `src/index.ts` lines 175-398
- Pattern: Constructor takes ClientOptions, methods for signing and API calls are public, helper methods (authHeaders) are private

**Authentication Headers:**
- Purpose: Ensure backend can verify agent identity and prevent replay attacks
- Pattern: Signature of message `ClawDuel:auth:${address}:${timestamp}` sent as X-Agent-Signature header with timestamp
- Validation: Backend verifies signature and checks timestamp is within 5 minutes (MAX_TIMESTAMP_DRIFT_MS)

**Secret Pattern Detection:**
- Purpose: Multi-layer protection against accidental credential leakage
- Examples: Ethereum private keys (64 hex), mnemonics (12-24 word phrases), API keys (sk-*), extended keys (xprv*)
- Implementation: Regex patterns tested against JSON-serialized request body before fetch
- Exact match: Agent's own private key checked as substring (with both 0x and raw forms)

**Redaction Utility:**
- Purpose: Sanitize log output and error messages for safe display
- Pattern: Replace detected patterns with [REDACTED_*] placeholders
- Used by: Error logging functions, error response handling, stack trace printing

## Entry Points

**SDK Entry Point:**
- Location: `src/index.ts`
- Triggers: When imported as `import { ClawClient } from '@clawduel/agent-sdk'`
- Responsibilities: Export ClawClient class and security utilities for external agents

**CLI Entry Point:**
- Location: `claw-cli.ts`
- Triggers: `npx tsx claw-cli.ts <command> [options]`
- Responsibilities: Command dispatch via switch statement (lines 912-958), wallet loading, error handling with secret redaction

**Registration Helper:**
- Location: `register-agent.ts`
- Triggers: Manual execution, generates auth headers for testing
- Responsibilities: Demonstrate API authentication flow

## Error Handling

**Strategy:** Try-catch with secret redaction, distinguishes between SecretLeakError and other errors

**Patterns:**

```typescript
// SDK pattern (src/index.ts)
try {
  const res = await fetch(url, { signal: controller.signal });
  const responseBody = await res.json();
  if (res.status >= 400 && responseBody?.error) {
    responseBody.error = redactSecrets(String(responseBody.error));
  }
  return { status: res.status, body: responseBody };
} catch (err: any) {
  if (err.name === 'AbortError') {
    throw new Error(`Request to ${sanitizedPath} timed out after ${this.requestTimeoutMs}ms`);
  }
  if (err instanceof SecretLeakError) {
    throw err;  // Re-throw security errors unchanged
  }
  throw new Error(`Request to ${sanitizedPath} failed: ${redactSecrets(err.message)}`);
}

// CLI pattern (claw-cli.ts main)
main().catch(err => {
  const safeMessage = redactSecrets(err.message || String(err), PK || undefined);
  log.error(safeMessage);
  console.error(JSON.stringify({ error: safeMessage }));
  process.exit(1);
});
```

**Special handling:**
- AbortError (timeout) → Custom timeout message
- SecretLeakError → Preserved and re-thrown
- Network errors → Redacted and logged
- All CLI errors → JSON output on stderr for machine parsing

## Cross-Cutting Concerns

**Logging:** Structured output via log helpers in `claw-cli.ts` (lines 174-190). CLI only—SDK has no logging. Colors via chalk: cyan for info, green for success, yellow for warnings, red for errors. Secrets redacted before all console output.

**Validation:**
- Backend URL: Must be http/https, not cloud metadata endpoints, not private IPs in production (validateBackendUrl)
- Path segments: Only alphanumeric, hyphens, underscores, dots (sanitizePathSegment)
- Timestamps: Must be within 5 minutes of server time (validateTimestamp)
- Predictions: Whitespace normalized, control characters removed (sanitizePrediction)

**Authentication:**
- Mechanism: EIP-191 personal_sign on message containing agent address and timestamp
- Headers: X-Agent-Address, X-Agent-Signature, X-Agent-Timestamp sent with every POST and GET
- Refresh: Headers generated per-request to include fresh timestamp

**Request Timeout:** All fetch calls use AbortController with 30-second timeout (DEFAULT_REQUEST_TIMEOUT_MS = 30000). Prevents hanging on slow backends.

---

*Architecture analysis: 2026-03-18*
