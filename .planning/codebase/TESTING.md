# Testing Patterns

**Analysis Date:** 2026-03-18

## Test Framework

**Runner:**
- Not detected - No test framework configured (Jest, Vitest, Mocha, etc.)

**Assertion Library:**
- Not applicable

**Run Commands:**
```bash
npm run build              # TypeScript compilation (only automated check)
# No test runner configured
```

## Testing Strategy

**Current Approach:**
- Manual testing via battle scripts and CLI commands
- Compilation validation via `npm run build` (TypeScript strict mode)
- Per CLAUDE.md: "Manual testing via battle scripts"

**Where Tests Would Go:**
If a testing framework is added, test files should be co-located with source code:
- `clawduel-cli.test.ts` alongside `clawduel-cli.ts`

## Manual Testing Patterns

**CLI Commands Tested Manually:**
- `npm run build` - Validates TypeScript compilation
- `npx tsx clawduel-cli.ts init` - Encrypted keyfile setup
- `npx tsx clawduel-cli.ts deposit --amount <usdc>` - USDC deposit
- `npx tsx clawduel-cli.ts balance` - Check balance
- `npx tsx clawduel-cli.ts queue --bet-tier <amount>` - Queue for match
- `npx tsx clawduel-cli.ts dequeue --bet-tier <amount>` - Leave queue
- `npx tsx clawduel-cli.ts poll` - Check current match
- `npx tsx clawduel-cli.ts submit --match-id <id> --prediction <value>` - Submit prediction
- `npx tsx clawduel-cli.ts status` - Agent info
- `npx tsx clawduel-cli.ts matches [--status <filter>] [--page <n>] [--category <cat>] [--from <ISO>] [--to <ISO>]` - List matches
- `npx tsx clawduel-cli.ts match --id <matchId>` - Get specific match

## Code Coverage

**Requirements:** None enforced

**Manual Coverage Areas (if testing were added):**

### Critical Security Functions (HIGH PRIORITY)
Files: `clawduel-cli.ts`

1. **Secret Leak Detection:**
   - `detectSecretLeak()` - Regex patterns for detecting secrets
   - `assertNoSecretLeak()` - Blocking requests containing secrets
   - Test cases:
     - Ethereum private keys (with and without 0x prefix)
     - BIP-39 mnemonic seed phrases (12 and 24 word variants)
     - Extended private keys (xprv format)
     - API keys (sk-, sk-ant- prefixes)
     - AWS secret keys (40-char base64)
     - False positives (random 64-char hex that isn't private key context)
     - Exact match against agent's own private key

2. **Secret Redaction:**
   - `redactSecrets()` - Proper masking of secrets in logs
   - Test cases:
     - Various secret formats properly redacted
     - Non-secret text untouched
     - Multiple secrets in single string
     - Case sensitivity for patterns

3. **URL Validation:**
   - `validateBackendUrl()` - SSRF prevention
   - Test cases:
     - Valid HTTP/HTTPS URLs accepted
     - Non-HTTP(S) schemes rejected
     - Private IP ranges rejected (10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16)
     - Localhost and 127.0.0.1 allowed (development)
     - AWS metadata endpoint (169.254.169.254) rejected
     - Malformed URLs properly caught

4. **Path Sanitization:**
   - `sanitizePathSegment()` - Path traversal prevention
   - Test cases:
     - Alphanumeric, hyphens, underscores, dots pass through
     - Control characters removed
     - Special characters stripped

### API Request Handling (HIGH PRIORITY)
Files: `clawduel-cli.ts` (apiPost, apiGet)

1. **Request Timeout:**
   - Default 30 seconds enforced
   - AbortController properly aborts hanging requests
   - Error message includes timeout duration
   - Cleanup happens in finally block

2. **Authentication Headers:**
   - `authHeaders()` generates valid headers
   - Includes X-Agent-Address, X-Agent-Signature, X-Agent-Timestamp
   - Message properly formatted: `ClawDuel:auth:${address}:${timestamp}`
   - Timestamp within acceptable drift (5 minutes)

3. **Response Handling:**
   - Non-200 status responses properly captured
   - Error messages in responses redacted
   - JSON parsing errors handled gracefully
   - Network errors converted to descriptive messages

### Key Management (HIGH PRIORITY)
Files: `clawduel-cli.ts`

1. **Keyfile Encryption:**
   - `cmdInit()` - Private key encryption via ethers.Wallet
   - Test cases:
     - Valid private keys accepted
     - Invalid keys rejected with helpful message
     - Password-protected encryption works
     - File permissions set to 0o600 (owner read/write only)
     - Keyfile location respected

2. **Keyfile Loading:**
   - `loadWallet()` - Decryption and setup
   - Test cases:
     - Correct password decrypts successfully
     - Wrong password fails gracefully
     - Fallback to AGENT_PRIVATE_KEY env var works
     - Proper error when neither available

### Contract Interaction (MEDIUM PRIORITY)
Files: `clawduel-cli.ts`

1. **Balance Queries:**
   - `cmdBalance()` - Queries Bank contract
   - Test cases:
     - Correct address queried
     - Units properly converted (6 decimals for USDC)
     - Available + locked = total

2. **USDC Approval & Deposit:**
   - `cmdDeposit()` - Approve + deposit flow
   - Test cases:
     - Insufficient balance detected and reported
     - Approval transaction awaited
     - Deposit transaction awaited
     - Proper error if blockchain fails

3. **EIP-712 Attestation Signing:**
   - `signDuelAttestation()` - Typed data signing
   - Test cases:
     - Proper domain name and version
     - Correct chain ID
     - Nonce incremented from on-chain value
     - Deadline properly set (default 3600s)
     - Signature verified off-chain

### Data Sanitization (MEDIUM PRIORITY)
Files: `clawduel-cli.ts`

1. **Prediction Sanitization:**
   - `sanitizePrediction()` - Text cleanup before submission
   - Test cases:
     - Control characters removed
     - Line endings normalized (\r\n → \n, \r → \n)
     - Tabs converted to spaces
     - Multiple spaces collapsed
     - Multiple newlines reduced to 2 max
     - Leading/trailing whitespace trimmed

### Command Logic (MEDIUM PRIORITY)
Files: `clawduel-cli.ts`

Commands with side effects that need testing:
- `cmdQueue()` - Nonce tracking, match queueing
- `cmdDequeue()` - Match removal
- `cmdSubmit()` - Prediction submission
- `cmdMatches()- URLSearchParams query building
- `cmdMatch()` - Single match retrieval

Test cases for each:
- Proper request body constructed
- Response properly parsed
- User feedback matches reality
- Error cases handled gracefully
- JSON output format consistent

## Gaps and Recommendations

**Critical Testing Gaps:**
1. **No automated secret detection tests** - Secret patterns could regress
2. **No API request tests** - Network timeouts, malformed responses not validated
3. **No key management tests** - Encryption/decryption flow not verified
4. **No contract interaction tests** - Blockchain calls not mocked/tested

**Recommended Testing Approach:**
1. Add Jest or Vitest with supertest for HTTP mocking
2. Mock ethers.js contracts and wallet operations
3. Mock filesystem for keyfile tests
4. Create test fixtures for responses
5. Aim for 80%+ coverage on security-critical functions

**Testing Command Recommendations:**
```bash
npm run test               # Run all tests
npm run test:watch        # Watch mode
npm run test:coverage     # Coverage report
npm run build              # Must succeed before publishing
```

## Test Structure (If Implemented)

**Recommended Suite Organization:**

```typescript
// clawduel-cli.test.ts
describe('CLI', () => {
  describe('Secret Leak Detection', () => {
    it('should detect Ethereum private keys (0x-prefixed)', () => { ... })
    it('should detect raw hex private keys', () => { ... })
    it('should detect mnemonic seed phrases', () => { ... })
    it('should block request containing agent\'s own key', () => { ... })
    it('should allow safe request bodies', () => { ... })
  })

  describe('URL Validation', () => {
    it('should accept valid HTTPS URLs', () => { ... })
    it('should reject non-HTTP protocols', () => { ... })
    it('should reject AWS metadata endpoint', () => { ... })
  })

  describe('API Methods', () => {
    it('should timeout requests after 30 seconds', () => { ... })
    it('should include auth headers', () => { ... })
    it('should redact secrets in error responses', () => { ... })
  })

  describe('Contract Interactions', () => {
    it('should query balance with correct decimals', () => { ... })
    it('should generate valid EIP-712 attestations', () => { ... })
  })
})

describe('CLI Commands', () => {
  describe('cmdInit', () => {
    it('should create encrypted keyfile', () => { ... })
    it('should reject invalid private keys', () => { ... })
  })

  describe('cmdDeposit', () => {
    it('should detect insufficient balance', () => { ... })
    it('should approve and deposit USDC', () => { ... })
  })

  describe('Prediction Sanitization', () => {
    it('should remove control characters', () => { ... })
    it('should normalize whitespace', () => { ... })
  })
})
```

## Notes on Current State

- Build passes with `npm run build` (TypeScript compilation with strict mode)
- No runtime test execution configured
- Manual testing workflow documented in README and skill.md
- Security is validated through code review and careful error handling
- Would benefit significantly from automated secret detection regression tests

---

*Testing analysis: 2026-03-18*
