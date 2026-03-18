# Codebase Concerns

**Analysis Date:** 2026-03-18

## Tech Debt

### Single-File Monolith
- **Issue:** CLI implementation is entirely in `clawduel-cli.ts` (967 lines) with no modular separation
- **Files:** `clawduel-cli.ts`
- **Impact:**
  - Hard to test individual features (no unit tests possible without mocking global state)
  - Difficult to reuse components between CLI and SDK
  - Code duplication between `clawduel-cli.ts` and `src/index.ts` (secret detection, URL validation, auth headers)
  - Future maintenance burden as feature count grows
- **Fix approach:** Extract command handlers into separate modules (`commands/`, `lib/` directories), create shared utilities module

### Code Duplication Between CLI and SDK
- **Issue:** Secret leak detection, URL validation, redaction, and auth header logic are duplicated across `clawduel-cli.ts` and `src/index.ts`
- **Files:** `clawduel-cli.ts` (lines 45-114), `src/index.ts` (lines 7-132)
- **Impact:**
  - Bug fixes must be applied in two places
  - Secret pattern additions require dual updates
  - Risk of divergence between CLI and SDK security guarantees
- **Fix approach:** Extract shared security utilities to `src/security.ts`, export from `src/index.ts`, import in both CLI and SDK

### Global Mutable State
- **Issue:** Global variables `PK`, `provider`, `wallet`, `contracts` are initialized lazily and modified during execution
- **Files:** `clawduel-cli.ts` (lines 287-296)
- **Impact:**
  - Makes concurrent execution or testing difficult
  - State is shared across commands in ways that aren't obvious
  - No clear contract for when these are available
- **Fix approach:** Create a `Session` class that encapsulates these initialization details, pass it to commands

### Silent JSON Parse Failure in Nonce Tracking
- **Issue:** `loadPendingNonces()` silently ignores corrupt files with empty catch block
- **Files:** `clawduel-cli.ts` (lines 496-502)
- **Impact:**
  - Operator doesn't know if nonce file is corrupted
  - Could lead to nonce reuse if file gets corrupted between queue and submission
  - No logging, no error visibility
- **Fix approach:** Log warnings when JSON parse fails, add diagnostic mode to inspect/repair nonce files

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

### Nonce Tracking Doesn't Survive Unsigned Submissions
- **Issue:** Pending nonce is only saved after successful queue; if signature is rejected, nonce is lost but next queue increments it
- **Files:** `clawduel-cli.ts` (lines 548-599, 518-546)
- **Impact:**
  - Operator queues, signature fails at backend, then requeues with different nonce (wastes on-chain nonce)
  - No way to recover if backend says "invalid signature" but CLI doesn't know why
- **Fix approach:** Save pending nonce immediately after signing (before sending), not after success

### JSON Response Parsing Assumes Valid JSON
- **Issue:** `await res.json()` is called without try-catch, will throw if response isn't valid JSON
- **Files:** `clawduel-cli.ts` (lines 340, 369), `src/index.ts` (lines 229, 263)
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
- **Files:** `clawduel-cli.ts` (lines 961-967), `src/index.ts` (lines 237-244)
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
- **Files:** `clawduel-cli.ts` (lines 118-134), `src/index.ts` (lines 76-107)
- **Current mitigation:** Blocks 169.254.169.254 (AWS metadata), rejects non-HTTP(S)
- **Recommendations:**
  - Add warning if BACKEND is localhost or 127.0.0.1 in production mode
  - Validate that contract addresses are sensible (not zero address)

### Auth Header Signature Not Scoped to Request
- **Risk:** Signature is only on address + timestamp, not on request method/body/path
- **Files:** `clawduel-cli.ts` (lines 313-320), `src/index.ts` (lines 281-291)
- **Current mitigation:** Backend presumably validates message format and checks against request context
- **Recommendations:**
  - Include request path in signed message for DELETE operations (to distinguish from POST)
  - Document that replay protection is time-based only (5 minute window)

### Secret Pattern Detection Has False Negatives
- **Risk:** 64-char hex patterns could match non-secret data (contract ABIs, transaction hashes, etc.)
- **Files:** `clawduel-cli.ts` (lines 50-58), `src/index.ts` (lines 13-26)
- **Current mitigation:** Patterns require boundary chars (not surrounded by hex chars), exact key match
- **Recommendations:**
  - Add configuration mode to disable pattern checking if too aggressive
  - Log what triggered pattern match (for debugging)

## Performance Bottlenecks

### Synchronous File I/O on Hot Path
- **Issue:** `loadPendingNonces()` does synchronous JSON parse for every queue operation
- **Files:** `clawduel-cli.ts` (lines 496-507)
- **Impact:** Negligible for single agent, but if CLI is used in automated loops, blocks main thread
- **Improvement path:** Cache nonce state in memory after first load, only persist on success

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

### Nonce Management System
- **Files:** `clawduel-cli.ts` (lines 481-546)
- **Why fragile:**
  - Relies on local JSON file to track pending nonces across invocations
  - No distributed lock mechanism (two agents with same key could corrupt file)
  - File can be deleted/corrupted without agent knowing
  - Per-tier nonce reuse logic is complex and could have off-by-one errors
- **Safe modification:**
  - Add comprehensive tests (currently none exist)
  - Add --show-nonces flag to inspect current state
  - Add --reset-nonces flag to rebuild from on-chain state
  - Lock file during read-modify-write cycle
- **Test coverage:** None - nonce logic is untested

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

### Single-Instance Nonce Tracking
- **Current capacity:** Works reliably for single agent instance only
- **Limit:** Breaks if agent runs on multiple machines with same key (nonce file corruption)
- **Scaling path:**
  - Add distributed lock (Redis, DynamoDB)
  - Or track pending nonces on-chain
  - Or require server to assign nonce (less secure)

### Request Timeout Fixed at 30 Seconds
- **Current capacity:** 30s timeout suitable for typical queue/submit operations
- **Limit:** May timeout on slow networks or overloaded backends
- **Scaling path:** Make timeout configurable via --timeout flag

## Dependencies at Risk

### Ethers.js v6 API Surface
- **Risk:** Large API surface (contracts, signers, providers, utilities), any version bump could break functionality
- **Files:** `clawduel-cli.ts` (lines 38, 219, 256, 426, etc.), `src/index.ts` (entire SDK)
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

### Nonce Management System
- **What's not tested:**
  - `getNextNonce()` function with various edge cases (empty file, pruning, reuse)
  - Concurrent access (though CLI is single-threaded)
  - File corruption recovery
- **Files:** `clawduel-cli.ts` (lines 481-546)
- **Risk:** Nonce reuse or skipping could silently corrupt match state
- **Priority:** HIGH

### Secret Leak Detection
- **What's not tested:**
  - All secret patterns against real examples
  - False positive rate (do contract ABIs trigger pattern?)
  - Exact key matching
- **Files:** `clawduel-cli.ts` (lines 45-91), `src/index.ts` (lines 7-74)
- **Risk:** Secrets leak or legitimate data gets blocked
- **Priority:** HIGH

### API Response Error Handling
- **What's not tested:**
  - Invalid JSON responses
  - Network timeouts
  - Backend error messages with secrets
- **Files:** `clawduel-cli.ts` (lines 323-385), `src/index.ts` (lines 211-278)
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
