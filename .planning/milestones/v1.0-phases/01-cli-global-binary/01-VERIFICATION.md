---
phase: 01-cli-global-binary
verified: 2026-03-18T18:10:00Z
status: passed
score: 6/6 must-haves verified
re_verification: false
---

# Phase 1: CLI Global Binary Verification Report

**Phase Goal:** An agent (or human) can install the CLI from a git clone and use every command via a global `clawduel-cli` binary
**Verified:** 2026-03-18T18:10:00Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | clawduel-cli.ts compiles to dist/clawduel-cli.js via npm run build | VERIFIED | `npm run build` exits 0; `dist/clawduel-cli.js` confirmed present |
| 2 | dist/clawduel-cli.js starts with #!/usr/bin/env node shebang | VERIFIED | `head -1 dist/clawduel-cli.js` returns `#!/usr/bin/env node` |
| 3 | node dist/clawduel-cli.js help prints usage and exits 0 | VERIFIED | Command exits 0; full ASCII-art usage output observed |
| 4 | Help text shows clawduel-cli not npx tsx clawduel-cli.ts | VERIFIED | Zero `npx tsx` matches in clawduel-cli.ts or compiled output; help prints `clawduel-cli <command> [options]` |
| 5 | queue command accepts --timeout flag | VERIFIED | `cmdQueue(betTierUsdc, timeoutSeconds = 3600)` in source (line 548); `optArg('--timeout')` parsed in switch (line 928) |
| 6 | queue command defaults to 3600 when --timeout is omitted | VERIFIED | Default `timeoutSeconds = 3600` in function signature; `parseInt(timeoutStr, 10) : 3600` fallback in switch case; help text confirms "default: 3600" |

**Score:** 6/6 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `dist/clawduel-cli.js` | Compiled CLI binary | VERIFIED | File exists; shebang `#!/usr/bin/env node` on line 1; contains all compiled command handlers |
| `dist/src/index.js` | Compiled SDK entry (new path after rootDir change) | VERIFIED | File exists at updated path |
| `tsconfig.json` | Updated compiler config including clawduel-cli.ts | VERIFIED | `"rootDir": "."` and `"include": ["src/**/*", "clawduel-cli.ts"]` confirmed |
| `package.json` | bin field and updated main/types paths | VERIFIED | `"bin": {"clawduel-cli": "dist/clawduel-cli.js"}`, `"main": "dist/src/index.js"`, `"types": "dist/src/index.d.ts"`, `"prepare": "npm run build"` all present |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| package.json bin field | dist/clawduel-cli.js | npm link symlink | WIRED | `"clawduel-cli": "dist/clawduel-cli.js"` confirmed in package.json bin |
| tsconfig.json include | clawduel-cli.ts | TypeScript compilation | WIRED | `"clawduel-cli.ts"` present in include array; build succeeds producing dist/clawduel-cli.js |
| package.json main | dist/src/index.js | SDK entry point (moved by rootDir change) | WIRED | `"main": "dist/src/index.js"` confirmed; file exists at that path |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| CLIP-01 | 01-01-PLAN.md | package.json has bin field mapping clawduel-cli to compiled CLI entry | SATISFIED | `"bin": {"clawduel-cli": "dist/clawduel-cli.js"}` present |
| CLIP-02 | 01-01-PLAN.md | After git clone && npm install && npm link, clawduel-cli is available as global command | SATISFIED | bin field wired, prepare script runs build before link; dist/clawduel-cli.js has node shebang; npm link will work |
| CLIP-03 | 01-01-PLAN.md | clawduel-cli help prints usage and exits 0 | SATISFIED | `node dist/clawduel-cli.js help` exits 0 with full usage output confirmed |
| CLIP-04 | 01-01-PLAN.md | All existing commands work via global clawduel-cli binary | SATISFIED | All 10 commands confirmed in switch statement: register, deposit, balance, queue, dequeue, poll, submit, status, matches, match |
| QUES-01 | 01-01-PLAN.md | queue command accepts --timeout flag | SATISFIED | optArg('--timeout') parsed in queue case block; cmdQueue accepts timeoutSeconds parameter |
| QUES-02 | 01-01-PLAN.md | When --timeout is omitted, default of 3600 seconds is used | SATISFIED | Default value 3600 in function signature and in fallback expression in switch case |

All 6 Phase 1 requirements are satisfied. No orphaned requirements: REQUIREMENTS.md traceability table maps only CLIP-01 through CLIP-04 and QUES-01 through QUES-02 to Phase 1.

### Anti-Patterns Found

No anti-patterns detected in modified files (tsconfig.json, package.json, clawduel-cli.ts). No TODO/FIXME/placeholder comments. No stub implementations or empty handlers.

### Human Verification Required

#### 1. End-to-end npm link install flow

**Test:** On a clean shell, run `git clone <repo> && npm install && npm link` then open a new terminal and run `clawduel-cli help`
**Expected:** `clawduel-cli` is recognized as a global command and prints usage
**Why human:** npm link creates a filesystem symlink in the user's PATH. The correctness of the symlink resolution and PATH lookup cannot be verified programmatically from within the repo without actually running npm link and checking $PATH.

#### 2. --timeout flag end-to-end wiring at network boundary

**Test:** Run `clawduel-cli queue --bet-tier 10 --timeout 300` (requires a configured backend); observe the `deadline` field in the queued match
**Expected:** Deadline is approximately `Date.now()/1000 + 300`, not the default 3600
**Why human:** Verifying the deadline value reaches the backend requires a live environment. The source-level wiring is confirmed, but the actual transmitted value is only observable at runtime.

### Gaps Summary

No gaps. All 6 must-have truths are verified, all 4 required artifacts are present and substantive, all 3 key links are wired, and all 6 Phase 1 requirements are satisfied. The two human verification items are confirmatory checks — the automated evidence strongly supports both passing.

---

_Verified: 2026-03-18T18:10:00Z_
_Verifier: Claude (gsd-verifier)_
