# Codebase Concerns

**Analysis Date:** 2026-03-18

## Tech Debt

### Single-File Monolith
- **Issue:** CLI implementation is entirely in `clawduel-cli.ts` (967 lines) with no modular separation
- **Files:** `clawduel-cli.ts`
- **Impact:**
  - Hard to test individual features (no unit tests possible without mocking global state)
  - Future maintenance burden as feature count grows
- **Fix approach:** Extract command handlers into separate modules (`commands/`, `lib/` directories), create shared utilities module

### Global Mutable State
- **Issue:** Global variables `PK`, `provider`, `wallet`, `contracts` are initialized lazily and modified during execution
- **Files:** `clawduel-cli.ts` (lines 287-296)
- **Impact:**
  - Makes concurrent execution or testing difficult
  - State is shared across commands in ways that aren't obvious
  - No clear contract for when these are available
- **Fix approach:** Create a `Session` class that encapsulates these initialization details, pass it to commands

### Hardcoded Contract Addresses
- **Issue:** Default contract addresses are hardcoded as fallback when env vars not set
- **Files:** `clawduel-cli.ts` (lines 300-302)
- **Impact:**
  - Using wrong addresses silently if env var misconfiguration
  - No validation that addresses match expected contracts
  - Easy to accidentally use testnet addresses in production
- **Fix approach:** Make addresses required (no defaults), or add strict validation/confirmation prompt

### Environment Variable Validation Gap
- **Issue:** No validation that required env vars are set or valid before operations
- **Files:** `clawduel-cli.ts` (lines 244-245, 284-285)
- **Impact:**
  - RPC_URL and BACKEND_URL have localhost defaults that may be wrong
  - Error messages only appear when first used, not at startup
  - Agent may queue/deposit before realizing connection is wrong
- **Fix approach:** Add startup validation in `main()`, require explicit --rpc and --backend flags or early env check

## Known Bugs

### Auth Timestamp Validation Not Enforced
- **Issue:** `validateTimestamp()` is defined but only called during `cmdQueue()` execution when creating auth headers
- **Files:** `clawduel-cli.ts` (lines 140-150, 308-310)
- **Impact:**
  - Clock drift is only validated once per queue, not for all API calls
  - If system clock jumps between queue and submit, submit would fail with auth error
  - No retry mechanism to handle transient timestamp misalignment
- **Workaround:** Ensure system time stays synchronized while CLI is running
- **Fix approach:** Move validation to `authHeaders()` function so it's enforced for all requests

### Ready Acknowledgement Loop Doesn't Handle Race Condition
- **Issue:** After sending ready signal, CLI waits for `startsAt` time, but doesn't handle case where both agents are already ready and waiting
- **Files:** `clawduel-cli.ts` (lines 629-651)
- **Impact:**
  - If opponent was ready first, startsAt may be immediate or past, but no explicit sync
  - Match could start while poll response is being processed
- **Workaround:** Repolling after ready signal usually catches updated state
- **Fix approach:** Add explicit state machine tracking (waiting_ready → polling_for_problem → waiting_start)

### JSON Response Parsing Assumes Valid JSON
- **Issue:** `await res.json()` is called without try-catch, will throw if response isn't valid JSON
- **Files:** `clawduel-cli.ts` (lines 340, 369)
- **Impact:**
  - If backend returns HTML error (e.g., 500 with stack trace), JSON parse fails with unclear error
  - Network timeout or 502 response could leave uncaught promise rejection
  - Error message doesn't indicate parse failure vs logic error
- **Fix approach:** Wrap JSON parse in try-catch, return { error: "Invalid response format" }

### Match Start Synchronization Based on Client Time
- **Issue:** `cmdPoll()` uses `Date.now()` to calculate wait time, doesn't account for server time skew
- **Files:** `clawduel-cli.ts` (lines 636, 656)
- **Impact:**
  - If client clock is ahead of server, will wait less than intended
  - If client clock is behind, will wait longer
  - Could cause match to start before poll finishes processing
- **Fix approach:** Use server-provided startsAt as authoritative, and client time only for minimum epsilon check

## Security Considerations

### Private Key Exposure in Error Stacks
- **Risk:** Uncaught errors in async chains could expose private key in stack trace
- **Files:** `clawduel-cli.ts` (lines 961-967)
- **Current mitigation:** Top-level catch redacts errors before logging, but only message is redacted
- **Recommendations:**
  - Also redact stack traces in error objects
  - Add explicit checks to never pass PK directly to error constructors

### Prediction Submission Sanitization Not Consistent
- **Risk:** `sanitizePrediction()` removes control chars but doesn't validate prediction is numeric when needed
- **Files:** `clawduel-cli.ts` (lines 393-403, 669-701)
- **Current mitigation:** Backend presumably validates prediction format
- **Recommendations:**
  - Add client-side validation that matches backend expectations
  - Document what formats are accepted (numeric? text? ranges?)

### Keyfile Encryption Password Visibility
- **Risk:** Password is prompted and passed in plaintext through memory, could be visible in /proc on Linux
- **Files:** `clawduel-cli.ts` (lines 225-232)
- **Current mitigation:** Password is not logged, file is written with mode 0o600
- **Recommendations:**
  - Consider using `read-secret` library to hide password input
  - Document that password should be unique and not reused

### Backend URL SSRF Protection Incomplete
- **Risk:** Validation allows localhost for development but doesn't warn when production URL is localhost
- **Files:** `clawduel-cli.ts` (lines 118-134)
- **Current mitigation:** Blocks 169.254.169.254 (AWS metadata), rejects non-HTTP(S)
- **Recommendations:**
  - Add warning if BACKEND is localhost or 127.0.0.1 in production mode
  - Validate that contract addresses are sensible (not zero address)

### Auth Header Signature Not Scoped to Request
- **Risk:** Signature is only on address + timestamp, not on request method/body/path
- **Files:** `clawduel-cli.ts` (lines 313-320)
- **Current mitigation:** Backend presumably validates message format and checks against request context
- **Recommendations:**
  - Include request path in signed message for DELETE operations (to distinguish from POST)
  - Document that replay protection is time-based only (5 minute window)

### Secret Pattern Detection Has False Negatives
- **Risk:** 64-char hex patterns could match non-secret data (contract ABIs, transaction hashes, etc.)
- **Files:** `clawduel-cli.ts` (lines 50-58)
- **Current mitigation:** Patterns require boundary chars (not surrounded by hex chars), exact key match
- **Recommendations:**
  - Add configuration mode to disable pattern checking if too aggressive
  - Log what triggered pattern match (for debugging)

## Performance Bottlenecks

### Poll Loop Timing Not Optimized
- **Issue:** `cmdPoll()` waits using `setTimeout`, which is coarse-grained and doesn't account for request processing time
- **Files:** `clawduel-cli.ts` (lines 638-639, 658-659)
- **Impact:** If startsAt is 30 seconds away and polling takes 1 second, agent waits 31 seconds instead of 30
- **Improvement path:** Track elapsed time, subtract from remaining wait before setTimeout

### No Connection Pooling
- **Issue:** Each API call creates new fetch connection without keep-alive
- **Files:** `clawduel-cli.ts` (lines 333, 364)
- **Impact:** Negligible for interactive CLI, but would add latency if called 100s of times per day
- **Improvement path:** Add keep-alive headers in production, consider HTTP agent pooling

## Fragile Areas

### Nonce Generation
- **Files:** `clawduel-cli.ts`
- **Design:** Nonces are random 256-bit values checked against on-chain `usedNonces(address, uint256)`. This makes the system stateless -- no local file tracking, no distributed lock needed, no corruption risk. Collision probability is negligible for 256-bit random values.
- **Test coverage:** None - generateNonce() collision resistance not tested

### Ready Acknowledgement Flow
- **Files:** `clawduel-cli.ts` (lines 628-651)
- **Why fragile:**
  - Mixes poll-and-wait logic with ready signal handling
  - Three different endpoints (active/{address}, readyUrl, active/{address} again)
  - Timing-dependent (if ready takes long, might miss startsAt)
  - No explicit timeout for waiting
- **Safe modification:**
  - Extract to separate function `cmdReady()`
  - Add explicit timeout parameter
  - Document expected state transitions
- **Test coverage:** None

### EIP-712 Signature Generation
- **Files:** `clawduel-cli.ts` (lines 554-579)
- **Why fragile:**
  - Hard-coded domain name, version, chainId - any change breaks compatibility
  - No validation that signature format matches backend expectations
  - Deadline is set to 1 hour, no configurability
  - Uses on-chain nonce that may be out of sync with local state
- **Safe modification:**
  - Extract domain config to constants
  - Validate signature before sending (check it's valid format)
  - Make deadline configurable or at least log it
- **Test coverage:** None

## Scaling Limits

### Request Timeout Fixed at 30 Seconds
- **Current capacity:** 30s timeout suitable for typical queue/submit operations
- **Limit:** May timeout on slow networks or overloaded backends
- **Scaling path:** Make timeout configurable via --timeout flag

## Dependencies at Risk

### Ethers.js v6 API Surface
- **Risk:** Large API surface (contracts, signers, providers, utilities), any version bump could break functionality
- **Files:** `clawduel-cli.ts`
- **Impact:**
  - Breaking change in ethers v7 would require major rewrite
  - EIP-712 signing API is stable but wallet encryption API is less common
  - JsonRpcProvider constructor signature has changed between versions
- **Migration plan:**
  - Pin ethers to ^6.13.0 in package.json (already done)
  - Add integration tests that verify contract interaction still works
  - Monitor ethers.js releases for security patches

### Node.js Built-in APIs (fs, path, os)
- **Risk:** Generally stable, but readline API is deprecated in favor of alternatives
- **Files:** `clawduel-cli.ts` (lines 40-42)
- **Impact:** Node.js may drop readline in future version, password prompting would break
- **Migration plan:**
  - Consider using `enquirer` or `prompts` library instead of readline
  - Or accept that password prompting will need update in distant future

## Missing Critical Features

### No Test Suite
- **Problem:** Zero test coverage - commands are untested
- **Blocks:**
  - Confidence in refactoring
  - Detection of regressions
  - Ability to add features safely
- **Recommendation Priority:** HIGH

### No Dry-Run Mode
- **Problem:** No way to preview what will happen without executing (e.g., `queue --dry-run`)
- **Blocks:** Operators can't verify arguments are correct before spending money
- **Recommendation Priority:** HIGH

### No Transaction Receipts Tracking
- **Problem:** Commands like `deposit()` don't return tx hash or block confirmation
- **Blocks:** Operators can't verify blockchain receipt
- **Recommendation Priority:** MEDIUM

### No Retry Logic
- **Problem:** Transient network errors cause immediate failure
- **Blocks:** Reliability in flaky network environments
- **Recommendation Priority:** MEDIUM

## Test Coverage Gaps

### Secret Leak Detection
- **What's not tested:**
  - All secret patterns against real examples
  - False positive rate (do contract ABIs trigger pattern?)
  - Exact key matching
- **Files:** `clawduel-cli.ts` (lines 45-91)
- **Risk:** Secrets leak or legitimate data gets blocked
- **Priority:** HIGH

### API Response Error Handling
- **What's not tested:**
  - Invalid JSON responses
  - Network timeouts
  - Backend error messages with secrets
- **Files:** `clawduel-cli.ts` (lines 323-385)
- **Risk:** Unhandled exceptions or secret exposure in errors
- **Priority:** MEDIUM

### End-to-End Flow
- **What's not tested:**
  - Full flow: init → register → deposit → queue → poll → submit
  - Match lifecycle (ready acknowledgement, start sync, submission)
  - Nonce persistence across commands
- **Files:** `clawduel-cli.ts` (all command functions)
- **Risk:** Subtle integration bugs that only appear in real scenarios
- **Priority:** MEDIUM

---

*Concerns audit: 2026-03-18*
