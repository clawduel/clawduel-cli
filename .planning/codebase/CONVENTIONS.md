# Coding Conventions

**Analysis Date:** 2026-03-18

## Naming Patterns

**Files:**
- kebab-case with `.ts` extension
- Examples: `clawduel-cli.ts`, `register-agent.ts`
- Main SDK export: `src/index.ts`

**Functions:**
- camelCase
- Async functions with `async` keyword: `async function loadWallet()`, `async function apiPost()`
- Command handlers prefixed with `cmd`: `cmdInit()`, `cmdDeposit()`, `cmdBalance()`, `cmdStatus()`
- Helper functions with descriptive names: `detectSecretLeak()`, `validateBackendUrl()`, `redactSecrets()`

**Variables:**
- camelCase for local variables: `privateKey`, `responseBody`, `requestTimeoutMs`
- camelCase for object properties: `{ wallet: ethers.Wallet; privateKey: string }`

**Constants:**
- UPPER_SNAKE_CASE for module-level constants: `SECRET_PATTERNS`, `DEFAULT_REQUEST_TIMEOUT_MS`, `REQUEST_TIMEOUT_MS`, `MAX_TIMESTAMP_DRIFT_MS`, `KEYFILE_DIR`, `BACKEND`, `RPC`
- UPPER_SNAKE_CASE for compiled constants: `BANNER`, `PRIVATE_KEY_FOR_REDACTION`, `PENDING_NONCES_PATH`

**Types/Interfaces:**
- PascalCase
- Examples: `ClientOptions`, `SecretLeakError`, `PendingNonces`
- Custom error classes extend `Error` and set `this.name`

## Code Style

**Formatting:**
- TypeScript with strict mode enabled
- Target: ES2020, Module: CommonJS
- No ESLint or Prettier config found - code follows ad-hoc formatting
- Consistent 2-space indentation observed throughout

**Linting:**
- No formal linting configuration present
- Manual validation via `npm run build` (TypeScript compilation)

**Line Length:**
- Lines typically kept under 100 characters
- Long strings and comments wrapped appropriately

## Import Organization

**Order:**
1. External dependencies (ethers, chalk, dotenv)
2. Node.js built-in modules (fs, path, os, readline)
3. No local imports between files in codebase (single-file modules)

**Path Aliases:**
- Relative imports used within `src/`
- External dependencies imported directly by package name: `import { ethers } from 'ethers'`, `import chalk from 'chalk'`

**Examples:**
```typescript
// src/index.ts
import { ethers } from 'ethers';
import chalk from 'chalk';
import dotenv from 'dotenv';

// clawduel-cli.ts
import { ethers } from 'ethers';
import chalk from 'chalk';
import * as fs from 'fs';
import * as path from 'path';
import * as os from 'os';
import * as readline from 'readline';
```

## Error Handling

**Pattern:**
- All API calls wrapped in try-catch blocks
- Custom error class `SecretLeakError` for security violations
- Specific error type checking: `if (err.name === 'AbortError')` for timeout detection
- Errors re-thrown with redacted messages to prevent secret leakage

**Security-First Approach:**
- Secret leak detection runs BEFORE outgoing requests via `assertNoSecretLeak(body, privateKey)`
- Response error messages redacted via `redactSecrets()` to prevent secret reflection
- Timeout handling with explicit error message: `Request to ${path} timed out after ${timeout}ms`
- Custom `SecretLeakError` class distinguishable from generic errors for proper handling

**Examples from `src/index.ts`:**
```typescript
// Pattern 1: Secure request error handling with secret scanning
async apiPost(path: string, body: unknown): Promise<{ status: number; body: any }> {
  // Scan outgoing body for secrets BEFORE sending
  assertNoSecretLeak(body, this.privateKey);

  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), this.requestTimeoutMs);

  try {
    const res = await fetch(`${this.backendUrl}${sanitizedPath}`, {
      method: 'POST',
      headers,
      body: JSON.stringify(body),
      signal: controller.signal,
    });

    const responseBody = await res.json();

    // Redact any reflected secrets in error responses
    if (res.status >= 400 && responseBody?.error) {
      responseBody.error = redactSecrets(String(responseBody.error));
    }

    return { status: res.status, body: responseBody };
  } catch (err: any) {
    if (err.name === 'AbortError') {
      throw new Error(`Request to ${sanitizedPath} timed out after ${this.requestTimeoutMs}ms`);
    }
    if (err instanceof SecretLeakError) {
      throw err;
    }
    throw new Error(`Request to ${sanitizedPath} failed: ${redactSecrets(err.message || String(err))}`);
  } finally {
    clearTimeout(timeout);
  }
}

// Pattern 2: Custom error class for security violations
export class SecretLeakError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'SecretLeakError';
  }
}
```

**Examples from `clawduel-cli.ts`:**
```typescript
// Pattern 3: Graceful error handling with user feedback
async function cmdDeposit(amountUsdc: number) {
  const balance = await usdc.balanceOf(wallet.address);
  if (balance < amount) {
    log.error(`Insufficient USDC. Have ${ethers.formatUnits(balance, 6)}, need ${amountUsdc}`);
    console.log(JSON.stringify({ ok: false, error: ... }));
    return;
  }
}

// Pattern 4: Validation errors thrown with descriptive messages
function validateBackendUrl(url: string): void {
  let parsed: URL;
  try {
    parsed = new URL(url);
  } catch {
    throw new Error(`Invalid backend URL: ${redactSecrets(url)}`);
  }

  if (!['http:', 'https:'].includes(parsed.protocol)) {
    throw new Error(`Backend URL must use http or https protocol, got: ${parsed.protocol}`);
  }
}
```

## Logging

**Framework:** Native console (no logging library)

**Patterns:**
- Structured logging with color-coded prefixes via chalk
- Log object with methods: `log.info()`, `log.success()`, `log.error()`, `log.warn()`, `log.dim()`, `log.header()`, `log.field()`, `log.json()`

**When/How to Log:**
- Info: Setup steps, operations starting: `log.info('Decrypting keyfile...')`
- Success: Operations completed: `log.success('Keyfile saved to ' + KEYFILE_PATH)`
- Error: User-facing errors with context: `log.error('Invalid private key format. Aborting.')`
- Warn: Informational warnings: `log.warn()`
- Dim: Secondary information: `log.dim('Found keyfile at ' + KEYFILE_PATH)`
- Header: Section headers with formatting
- Field: Structured key-value display: `log.field('Address', wallet.address)`
- JSON: Machine-readable output: `log.json(data)`

**Redaction:**
- Private keys NEVER logged directly
- All error messages passed through `redactSecrets()` before logging
- Secrets redacted to placeholders: `0x[REDACTED_KEY]`, `sk-[REDACTED]`, `[REDACTED_HEX]`

**Example from `clawduel-cli.ts`:**
```typescript
const log = {
  info: (msg: string) => console.log(chalk.cyan('  INFO ') + chalk.white(msg)),
  success: (msg: string) => console.log(chalk.green('    OK ') + chalk.white(msg)),
  warn: (msg: string) => console.log(chalk.yellow('  WARN ') + chalk.white(msg)),
  error: (msg: string) => console.error(chalk.red(' ERROR ') + chalk.white(redactSecrets(msg, PRIVATE_KEY_FOR_REDACTION))),
  dim: (msg: string) => console.log(chalk.gray('       ' + msg)),
  header: (msg: string) => {
    console.log('');
    console.log(chalk.cyan.bold('  ' + msg));
    console.log(chalk.gray('  ' + '-'.repeat(44)));
  },
  field: (label: string, value: string) => {
    const padded = label.padEnd(14);
    console.log(chalk.white('  ' + padded) + chalk.yellow(value));
  },
  json: (data: any) => console.log(JSON.stringify(data, null, 2)),
};
```

## Comments

**When to Comment:**
- Section headers to organize code logically: `// --- Security: Secret Leak Detection ---`, `// --- Types ---`, `// --- Commands ---`
- Complex security logic with explanations
- Regex patterns with their purpose: `// Ethereum private keys: 64 hex chars, with or without 0x prefix`
- Important invariants or constraints
- TODO or FIXME when applicable

**JSDoc/TSDoc:**
- Used sparingly for public exports and class/interface documentation
- Method-level docstrings for SDK consumers in `src/index.ts`
- Example from `src/index.ts`:
```typescript
/**
 * Scans a string for embedded secrets. Returns the name of the first match found, or null.
 */
function detectSecretLeak(data: string): string | null { ... }

/**
 * Scans an object (to be JSON-serialized) for secrets. Throws if a secret is detected.
 * Also accepts the agent's own private key to do an exact-match check.
 */
function assertNoSecretLeak(body: unknown, privateKey?: string): void { ... }

/**
 * Custom error class for secret leak detection so callers can distinguish it.
 */
export class SecretLeakError extends Error { ... }

/**
 * ClawClient
 *
 * Handles on-chain interactions: deposits, balance queries, and EIP-712 attestation signing.
 * Used programmatically by agents that import the SDK directly.
 *
 * Includes built-in secret leak detection: all outgoing request bodies are scanned for
 * private keys, mnemonics, and other secrets before being sent to the backend.
 *
 * For CLI usage, run `npx tsx clawduel-cli.ts help` instead.
 */
export class ClawClient { ... }
```

## Function Design

**Size:**
- Most functions 10-50 lines
- Longer functions dedicated to specific operations: `cmdQueue()`, `cmdMatches()` handle complex logic with multiple steps
- Focused responsibility per function

**Parameters:**
- Single object parameter for configuration: `ClientOptions`, filter objects
- Path and body for API methods: `apiPost(path: string, body: unknown)`
- Avoid excessive parameter lists; use objects when >2 parameters needed

**Return Values:**
- Async functions return Promises with meaningful types: `Promise<{ status: number; body: any }>`
- Simple functions return specific types: `string | null`, `boolean`, `void`
- API methods return structured responses: `{ status: number; body: responseBody }`
- No implicit `any` return types - always explicit

**Examples:**
```typescript
// Simple utility with clear return
function detectSecretLeak(data: string): string | null { ... }

// API method with structured return
async apiPost(path: string, body: unknown): Promise<{ status: number; body: any }> { ... }

// Command with optional object parameter
async cmdMatches(filters: { status?: string; page?: string; category?: string; from?: string; to?: string } = {}) { ... }

// Wallet loader with tuple return
async function loadWallet(): Promise<{ wallet: ethers.Wallet; privateKey: string }> { ... }
```

## Module Design

**Exports:**
- `src/index.ts` exports public API for SDK consumers: class `ClawClient`, utility functions, error class
- Security utilities exported for programmatic use: `assertNoSecretLeak`, `detectSecretLeak`, `redactSecrets`, `validateBackendUrl`, `sanitizePathSegment`, `SECRET_PATTERNS`, `DEFAULT_REQUEST_TIMEOUT_MS`
- CLI in `clawduel-cli.ts` operates independently with `main()` entry point

**Barrel Files:**
- No barrel files (index.ts with re-exports from multiple modules)
- Single SDK file (`src/index.ts`) or standalone CLI (`clawduel-cli.ts`)

**Module Patterns:**
```typescript
// src/index.ts: Export class + utilities
export class ClawClient { ... }
export class SecretLeakError extends Error { ... }
export { assertNoSecretLeak, detectSecretLeak, redactSecrets, validateBackendUrl, ... }

// clawduel-cli.ts: Standalone with main()
async function main() { ... }
// Entry point at bottom
```

---

*Convention analysis: 2026-03-18*
