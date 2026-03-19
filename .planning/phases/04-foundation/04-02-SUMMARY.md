---
phase: 04-foundation
plan: 02
subsystem: auth, http, security
tags: [eip-191, reqwest, regex, secret-detection, ssrf, http-client]

requires:
  - phase: 04-foundation-01
    provides: "CLI skeleton with clap, config resolution, wallet management"
provides:
  - "Secret leak detection with 7 patterns (Ethereum keys, mnemonics, API keys, xprv, AWS)"
  - "EIP-191 auth header generation for backend requests"
  - "Authenticated HttpClient with 30s timeout, pre-request secret scanning, SSRF protection"
  - "Backend URL validation, path sanitization, timestamp validation"
  - "Error response redaction to prevent secret reflection"
affects: [05-commands, 06-shell]

tech-stack:
  added: [regex, url]
  patterns: [LazyLock compiled regex, pre-request security scanning, EIP-191 personal_sign]

key-files:
  created:
    - src/security.rs
    - src/auth.rs
    - src/http.rs
    - tests/security_test.rs
    - tests/http_test.rs
  modified:
    - src/lib.rs
    - Cargo.toml

key-decisions:
  - "Used LazyLock<Regex> for compiled patterns to avoid recompilation per call"
  - "Rust regex crate lacks lookbehind - used boundary-aware replacement with captures for raw hex redaction"
  - "HttpClient validates backend URL on construction, not per-request"

patterns-established:
  - "Security scanning: all POST/DELETE bodies pass through assert_no_secret_leak before network send"
  - "Auth injection: auth_headers() called per-request to generate fresh timestamp and signature"
  - "Error redaction: any 4xx/5xx response body with 'error' field is redacted before returning to caller"

requirements-completed: [CORE-04, CORE-05, CORE-06, CORE-08]

duration: 7min
completed: 2026-03-19
---

# Phase 4 Plan 2: Security, Auth, and HTTP Client Summary

**Secret leak detection with 7 regex patterns, EIP-191 auth headers via alloy signing, and authenticated HttpClient with 30s timeout and SSRF protection**

## Performance

- **Duration:** 7 min
- **Started:** 2026-03-19T22:18:56Z
- **Completed:** 2026-03-19T22:26:18Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- All 7 secret detection patterns from the TypeScript CLI ported to Rust with identical regex semantics
- EIP-191 auth header generation producing verifiable signatures (tested with address recovery)
- HttpClient with pre-request body scanning, SSRF URL validation, 30s timeout, and error redaction
- 35 tests covering security patterns, timestamp validation, auth headers, and HTTP client behavior

## Task Commits

Each task was committed atomically:

1. **Task 1: Security module** - `3fe0da7` (feat) - secret detection, redaction, SSRF validation, path sanitization
2. **Task 2: Auth + HTTP client** - `4c9d9b5` (feat) - EIP-191 signing, authenticated HTTP client with secret scanning

## Files Created/Modified
- `src/security.rs` - 7 secret patterns, assert_no_secret_leak, redact_secrets, validate_backend_url, sanitize_path_segment, validate_timestamp
- `src/auth.rs` - EIP-191 auth_headers() generating Content-Type, X-Agent-Address, X-Agent-Signature, X-Agent-Timestamp
- `src/http.rs` - HttpClient struct with get(), post(), delete(), request() methods, all authenticated and security-scanned
- `tests/security_test.rs` - 25 tests for all security functions
- `tests/http_test.rs` - 10 tests for timestamp, auth headers, and HTTP client
- `src/lib.rs` - Added auth, http, security module declarations
- `Cargo.toml` - Added regex and url dependencies

## Decisions Made
- Used LazyLock<Regex> statics for zero-cost pattern compilation after first use
- Rust regex crate does not support lookbehind assertions; used boundary-character-aware replacement with regex::Captures to preserve surrounding characters during raw hex redaction
- HttpClient validates backend URL once at construction time (not per-request) since URL doesn't change

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
- Rust regex crate lacks lookbehind/lookahead support needed for the raw hex redaction pattern from TypeScript. Solved by matching boundary characters in the regex and preserving them during replacement using Captures.
- LazyLock<Regex> inside a static slice reference triggers interior mutability error in Rust 2024. Solved by using individual static LazyLock items with a parallel names array.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- HttpClient is ready for Phase 5 commands to use via `HttpClient::new(backend_url, signer, address, pk_hex)`
- All security primitives exported from the library crate for command-level use
- 55 total tests passing across config, wallet, security, and HTTP modules

---
*Phase: 04-foundation*
*Completed: 2026-03-19*
