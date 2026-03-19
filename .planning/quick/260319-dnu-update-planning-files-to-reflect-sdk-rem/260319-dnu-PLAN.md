---
phase: quick
plan: 260319-dnu
type: execute
wave: 1
depends_on: []
files_modified:
  - .planning/PROJECT.md
  - .planning/STATE.md
  - .planning/codebase/ARCHITECTURE.md
  - .planning/codebase/STACK.md
  - .planning/codebase/STRUCTURE.md
  - .planning/codebase/INTEGRATIONS.md
  - .planning/codebase/CONCERNS.md
  - .planning/codebase/CONVENTIONS.md
  - .planning/codebase/TESTING.md
autonomous: true
requirements: []
must_haves:
  truths:
    - "No planning file references ClawClient SDK, src/index.ts as SDK, or @clawduel/agent-sdk"
    - "Architecture described as standalone CLI, not dual-layer SDK+CLI"
    - "Nonce system described as random 256-bit with on-chain usedNonces check, not incremental tracking"
    - "No references to PendingNonces, loadPendingNonces, savePendingNonces, getNextNonce, PENDING_NONCES_PATH, pending_nonces.json"
    - "Package name shown as @clawduel/clawduel-cli with binary clawduel"
  artifacts:
    - path: ".planning/PROJECT.md"
      provides: "Updated project description without SDK references"
    - path: ".planning/codebase/ARCHITECTURE.md"
      provides: "Architecture without dual-layer pattern"
    - path: ".planning/codebase/INTEGRATIONS.md"
      provides: "Integration docs without pending nonce tracking"
    - path: ".planning/codebase/CONCERNS.md"
      provides: "Concerns without stale nonce tracking issues"
  key_links: []
---

<objective>
Update all .planning/ documentation files to reflect three major changes: SDK removal (no more ClawClient/src/index.ts/@clawduel/agent-sdk), package rename to @clawduel/clawduel-cli, and nonce system change from incremental tracking to random 256-bit nonces with on-chain usedNonces check.

Purpose: Keep planning docs accurate so future phases/tasks reference correct architecture.
Output: All .planning files updated, no stale SDK or incremental nonce references remain.
</objective>

<execution_context>
@/home/solthodox/.claude/get-shit-done/workflows/execute-plan.md
@/home/solthodox/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/STATE.md
@.planning/codebase/ARCHITECTURE.md
@.planning/codebase/STACK.md
@.planning/codebase/STRUCTURE.md
@.planning/codebase/INTEGRATIONS.md
@.planning/codebase/CONCERNS.md
@.planning/codebase/CONVENTIONS.md
@.planning/codebase/TESTING.md
</context>

<tasks>

<task type="auto">
  <name>Task 1: Update PROJECT.md, STATE.md, and top-level codebase docs (ARCHITECTURE, STACK, STRUCTURE)</name>
  <files>.planning/PROJECT.md, .planning/STATE.md, .planning/codebase/ARCHITECTURE.md, .planning/codebase/STACK.md, .planning/codebase/STRUCTURE.md</files>
  <action>
In PROJECT.md:
- Line 13: Change "Architecture: dual-layer (ClawClient SDK + CLI wrapper)" to "Architecture: standalone CLI"
- Line 22: Remove "ClawClient SDK in src/index.ts provides programmatic API" validated requirement entirely
- Line 44: Update Out of Scope item "@clawduel/agent-sdk programmatic SDK — future project" to remove it (SDK is no longer planned, it was removed). Keep the line but update to reflect SDK was removed, not deferred.
- Remove any other references to src/index.ts as SDK entry point

In STATE.md:
- Line 63: Decision "[Phase 01]: rootDir changed from ./src to . to include clawduel-cli.ts in compilation; SDK paths adjusted to dist/src/" — update to remove SDK path mention since SDK no longer exists

In ARCHITECTURE.md:
- Line 7: Change "Dual-layer client SDK with CLI wrapper" to "Standalone CLI application"
- Lines 9-10: Remove the paragraph about ClawClient class in src/index.ts being a reusable SDK. Replace with: "The codebase is a standalone CLI in `clawduel-cli.ts`. Security utilities, blockchain interaction, HTTP communication, and CLI command handling are all in the single CLI file."
- Remove "Client Library Layer" section (lines 47-52) entirely
- Remove "SDK Entry Point" from Entry Points section (lines 110-113)
- Update all references from `@clawduel/agent-sdk` to `@clawduel/clawduel-cli`
- Remove references to "SDK users", "SDK consumers", "external agents importing the SDK"
- Remove the "New Feature in ClawClient" section if referenced

In STACK.md:
- Lines 72-76: Remove SDK/Library entry point section referencing src/index.ts, ClawClient, @clawduel/agent-sdk
- Update package name from `@clawduel/agent-sdk` to `@clawduel/clawduel-cli`

In STRUCTURE.md:
- Remove/update "src/ (SDK Library)" directory purpose section (lines 23-27)
- Update package.json name from `@clawduel/agent-sdk` to `@clawduel/clawduel-cli`
- Remove "New Feature in ClawClient" section (lines 109-119)
- Remove "New Shared Utility" dual-file references (lines 121-127) — utilities are CLI-only now
- Update import patterns section to remove SDK imports
- Update package.json section (lines 192-198)
  </action>
  <verify>grep -ri "agent-sdk\|ClawClient\|dual.layer\|SDK" .planning/PROJECT.md .planning/STATE.md .planning/codebase/ARCHITECTURE.md .planning/codebase/STACK.md .planning/codebase/STRUCTURE.md | grep -v "Out of Scope" | grep -v "removed" | wc -l should be 0</verify>
  <done>No stale SDK, ClawClient, or dual-layer references in PROJECT.md, STATE.md, ARCHITECTURE.md, STACK.md, or STRUCTURE.md (except historical notes about removal)</done>
</task>

<task type="auto">
  <name>Task 2: Update INTEGRATIONS, CONCERNS, CONVENTIONS, TESTING for nonce system and SDK removal</name>
  <files>.planning/codebase/INTEGRATIONS.md, .planning/codebase/CONCERNS.md, .planning/codebase/CONVENTIONS.md, .planning/codebase/TESTING.md</files>
  <action>
In INTEGRATIONS.md:
- Line 28: Remove "Pending nonces: `~/.clawduel/pending_nonces.json` (local JSON tracking)" from Data Storage
- Line 69: Update `nonces(address)` to `usedNonces(address,uint256)` for ClawDuel Contract methods. The contract method is now `usedNonces(address,uint256)` which returns bool (whether a nonce was already used)
- Line 75-76: Same update for MultiDuel Contract nonces method
- Line 103: Change `@clawduel/agent-sdk` to `@clawduel/clawduel-cli` in Publishing section
- Line 122-123: Remove `RPC_URL`, `BANK_ADDRESS`, `CLAWDUEL_ADDRESS`, `MULTIDUEL_ADDRESS`, `USDC_ADDRESS` env vars that were only in ClawClient constructor
- Line 159: Remove "configurable in `ClawClient` options" — timeout is in CLI directly
- Remove all references to src/index.ts as separate module

In CONCERNS.md:
- Lines 17-24: Remove or rewrite "Code Duplication Between CLI and SDK" tech debt entry — there is no SDK anymore, no duplication
- Lines 35-42: Remove "Silent JSON Parse Failure in Nonce Tracking" — pending nonces file no longer exists
- Lines 83-89: Remove "Nonce Tracking Doesn't Survive Unsigned Submissions" known bug — incremental nonce tracking removed
- Lines 161-165: Remove "Synchronous File I/O on Hot Path" about loadPendingNonces — no longer exists
- Lines 180-193: Rewrite "Nonce Management System" fragile area — it is no longer a local JSON file system. Replace with brief note that nonces are now random 256-bit values checked against on-chain `usedNonces(address,uint256)`, making the system stateless and non-fragile
- Lines 222-229: Remove "Single-Instance Nonce Tracking" scaling limit — no longer applies with random nonces
- Lines 285-292: Remove "Nonce Management System" test coverage gap — old system removed. Optionally replace with note about testing generateNonce() collision resistance
- Remove references to src/index.ts line numbers in security/API sections since that file is gone
- Update all "clawduel-cli.ts and src/index.ts" dual references to just "clawduel-cli.ts"

In CONVENTIONS.md:
- Line 10: Remove "Main SDK export: src/index.ts" from file naming
- Line 24: Remove `PENDING_NONCES_PATH` from constants list. Add `generateNonce` or similar if relevant.
- Line 29: Remove `PendingNonces` from Types/Interfaces list
- Lines 88-134: Remove or heavily trim the SDK error handling examples (from src/index.ts) — keep only CLI patterns
- Lines 217-248: Remove JSDoc/TSDoc examples from src/index.ts ClawClient
- Lines 287-305: Remove SDK module patterns section, update to CLI-only
- Update all dual-file references (src/index.ts + clawduel-cli.ts) to just clawduel-cli.ts

In TESTING.md:
- Lines 46-49: Remove "Programmatic Testing (SDK)" section about ClawClient
- Lines 58-59: Update file references from "src/index.ts, clawduel-cli.ts" to just "clawduel-cli.ts"
- Lines 97-98: Remove src/index.ts reference from API Request Handling files
- Lines 222-249: Remove src/index.ts test structure (ClawClient describe block) — keep clawduel-cli.ts test structure only
- Remove all references to "SDK" testing patterns
  </action>
  <verify>grep -rn "pending_nonces\|PendingNonces\|loadPendingNonces\|savePendingNonces\|getNextNonce\|PENDING_NONCES_PATH\|agent-sdk\|src/index.ts" .planning/codebase/INTEGRATIONS.md .planning/codebase/CONCERNS.md .planning/codebase/CONVENTIONS.md .planning/codebase/TESTING.md | wc -l should be 0</verify>
  <done>No stale nonce tracking references (PendingNonces, loadPendingNonces, pending_nonces.json, etc.), no agent-sdk references, and no src/index.ts references remain in codebase docs</done>
</task>

</tasks>

<verification>
After both tasks complete:
1. `grep -ri "agent-sdk" .planning/` returns 0 results (or only historical/removal notes)
2. `grep -ri "pending_nonces\|PendingNonces\|loadPendingNonces\|savePendingNonces\|getNextNonce\|PENDING_NONCES_PATH" .planning/` returns 0 results
3. `grep -ri "dual.layer" .planning/` returns 0 results
4. `grep -ri "ClawClient" .planning/` returns 0 results (or only historical/removal notes)
5. `grep -ri "src/index.ts" .planning/` returns 0 results
</verification>

<success_criteria>
All .planning files accurately describe the current architecture: standalone CLI (@clawduel/clawduel-cli), no SDK layer, random 256-bit nonces with on-chain usedNonces check instead of incremental local tracking.
</success_criteria>

<output>
After completion, create `.planning/quick/260319-dnu-update-planning-files-to-reflect-sdk-rem/260319-dnu-SUMMARY.md`
</output>
