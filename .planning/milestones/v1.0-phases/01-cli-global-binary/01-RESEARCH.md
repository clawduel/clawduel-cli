# Phase 1: CLI Global Binary - Research

**Researched:** 2026-03-18
**Domain:** Node.js CLI packaging, npm bin/link, TypeScript compilation
**Confidence:** HIGH

## Summary

Phase 1 transforms the existing `clawduel-cli.ts` (currently run via `npx tsx clawduel-cli.ts`) into a globally installable `clawduel-cli` binary. The core challenge is that `clawduel-cli.ts` lives at the project root outside `tsconfig.json`'s `rootDir: ./src`, so it is NOT currently compiled by `tsc`. The `bin` field in `package.json` must point to a compiled JS file in `dist/`, and the shebang must change from `#!/usr/bin/env npx tsx` to `#!/usr/bin/env node`.

Additionally, the `queue` command's attestation deadline is hardcoded to `3600` seconds on line 561 and needs to accept an optional `--timeout` flag.

**Primary recommendation:** Add `clawduel-cli.ts` to the TypeScript compilation (update `tsconfig.json` to include it), add a `bin` field to `package.json` pointing to `dist/clawduel-cli.js`, update the shebang to `#!/usr/bin/env node`, and add `--timeout` as an optional argument to the `queue` command.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| CLIP-01 | `package.json` has `bin` field mapping `clawduel-cli` to compiled CLI entry point | Standard npm bin field pattern; point to `dist/clawduel-cli.js` |
| CLIP-02 | After `git clone && npm install && npm link`, `clawduel-cli` is available as global command | npm link creates symlink from global bin to local package bin entry |
| CLIP-03 | `clawduel-cli help` prints usage information and exits 0 | Help already works; need to verify exit code is 0 (it is -- returns from main naturally) |
| CLIP-04 | All existing commands work via global `clawduel-cli` binary | Requires clawduel-cli.ts to compile to JS and run under `node` instead of `tsx` |
| QUES-01 | `queue` command accepts `--timeout <seconds>` flag for attestation deadline | Currently hardcoded `3600` on line 561; add optional arg parsing |
| QUES-02 | When `--timeout` omitted, default of 3600 seconds is used | Keep existing `3600` as default value |
</phase_requirements>

## Standard Stack

### Core (already in project)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| typescript | ^5.3.3 | Compilation | Already in project |
| ethers | ^6.13.0 | Blockchain interaction | Already in project |
| chalk | ^4.1.2 | CLI output colors | Already in project, v4 is CJS-compatible |
| dotenv | ^16.4.0 | Env var loading | Already in project |

### No New Dependencies Needed

This phase requires zero new npm packages. All changes are configuration and minor code edits:
- `tsconfig.json` changes to include `clawduel-cli.ts` in compilation
- `package.json` changes to add `bin` field
- Shebang change in `clawduel-cli.ts`
- Optional `--timeout` arg parsing in queue command

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Manual arg parsing | commander/yargs | Overkill -- existing `getArg()` pattern works, project has no CLI framework, adding one is out of scope |
| Compiling clawduel-cli.ts via tsconfig | Keeping tsx shebang | tsx adds runtime dep and startup latency; compiled JS is the standard for npm bin entries |

## Architecture Patterns

### Compilation Strategy

**Critical finding:** `clawduel-cli.ts` is at the project root. `tsconfig.json` has `rootDir: ./src` and `include: ["src/**/*"]`. This means `clawduel-cli.ts` is NOT compiled today.

**Recommended approach:** Update `tsconfig.json` to compile both `src/` and root-level CLI files:

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "CommonJS",
    "declaration": true,
    "outDir": "./dist",
    "rootDir": ".",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true
  },
  "include": ["src/**/*", "clawduel-cli.ts"],
  "exclude": ["node_modules", "dist"]
}
```

**Impact of `rootDir: "."` change:**
- `src/index.ts` compiles to `dist/src/index.ts` instead of `dist/index.js`
- `clawduel-cli.ts` compiles to `dist/clawduel-cli.js`
- `package.json` `main` field must update: `dist/index.js` -> `dist/src/index.js`
- `package.json` `types` field must update: `dist/index.d.ts` -> `dist/src/index.d.ts`

**Alternative approach (less disruptive):** Use a separate tsconfig for the CLI:
- Keep `tsconfig.json` as-is for the SDK (`src/`)
- Add `tsconfig.cli.json` that extends it and includes `clawduel-cli.ts`
- Update `build` script: `"build": "tsc && tsc -p tsconfig.cli.json"`

**Recommended:** The single-tsconfig approach (change `rootDir` to `"."`) is simpler and avoids maintaining two configs. The `main`/`types` path change is a one-time update.

### npm bin Field Pattern

```json
{
  "bin": {
    "clawduel-cli": "dist/clawduel-cli.js"
  }
}
```

After `npm link`, this creates a symlink: `$(npm prefix -g)/bin/clawduel-cli` -> `<project>/dist/clawduel-cli.js`.

### Shebang Change

Current: `#!/usr/bin/env npx tsx`
Required: `#!/usr/bin/env node`

The compiled JS file in `dist/clawduel-cli.js` will run under `node` directly. TypeScript strips shebangs during compilation, so we may need to prepend it to the compiled output. However, TypeScript 5.3+ preserves shebangs in the output if they are present in the source `.ts` file. This is HIGH confidence -- TypeScript has preserved shebangs since ~TS 4.x.

### Anti-Patterns to Avoid
- **Keeping tsx as runtime:** Global binaries should not depend on `tsx` being installed. The compiled JS file must run with just `node`.
- **Forgetting to build before link:** `npm link` uses whatever is in `dist/`. A `prepare` or `preinstall` script should run `npm run build` automatically.
- **Hardcoding paths in help text:** The help text currently says `npx tsx clawduel-cli.ts <command>`. This must update to `clawduel-cli <command>`.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| CLI argument parsing framework | Full commander-like lib | Existing `getArg()` and `optArg()` pattern | Project already has a working pattern; 6 requirements don't justify a framework migration |
| Global binary installation | Custom install script | `npm link` (dev) / `npm install -g` (prod) | npm handles symlinks, PATH, and cross-platform concerns |

## Common Pitfalls

### Pitfall 1: tsconfig rootDir Mismatch
**What goes wrong:** Changing `rootDir` from `./src` to `.` changes the output directory structure. `dist/index.js` becomes `dist/src/index.js`.
**Why it happens:** TypeScript mirrors the source directory structure relative to `rootDir` in the output.
**How to avoid:** Update `package.json` `main` and `types` fields to match the new output paths. Test with `npm run build && ls dist/` to verify.
**Warning signs:** `Cannot find module '@clawduel/agent-sdk'` errors after the change.

### Pitfall 2: Missing Shebang in Compiled Output
**What goes wrong:** `dist/clawduel-cli.js` has no shebang, so `clawduel-cli` command fails with a syntax error or tries to run as shell script.
**Why it happens:** Some TypeScript versions or configurations strip shebangs.
**How to avoid:** After building, verify `head -1 dist/clawduel-cli.js` shows `#!/usr/bin/env node`. TypeScript 5.3 should preserve it, but verify.
**Warning signs:** `clawduel-cli: line 1: syntax error` when running the global command.

### Pitfall 3: npm link Without Building First
**What goes wrong:** `npm link` symlinks to `dist/clawduel-cli.js` which doesn't exist yet, resulting in `ENOENT` or stale code.
**Why it happens:** `npm link` doesn't trigger `build` unless a `prepare` script is configured.
**How to avoid:** Add `"prepare": "npm run build"` to `package.json` scripts. This runs automatically on `npm install` and before `npm link`.
**Warning signs:** Command not found or old behavior after code changes.

### Pitfall 4: File Permission on dist/clawduel-cli.js
**What goes wrong:** The compiled JS file isn't executable, so the symlink fails.
**Why it happens:** TypeScript outputs non-executable files. npm link typically handles this, but some environments need explicit chmod.
**How to avoid:** npm link handles permissions when a `bin` field is present. If issues arise, add `chmod +x dist/clawduel-cli.js` to the build script.
**Warning signs:** `Permission denied` when running `clawduel-cli`.

### Pitfall 5: Help Text Still References Old Invocation
**What goes wrong:** After installing globally, `clawduel-cli help` still says `npx tsx clawduel-cli.ts <command>`.
**Why it happens:** The help text on line 837 and error messages on lines 906, 956 hardcode the old invocation.
**How to avoid:** Update all references from `npx tsx clawduel-cli.ts` to `clawduel-cli`.
**Warning signs:** Confusing UX when help says to use a different command.

## Code Examples

### package.json bin Field
```json
{
  "bin": {
    "clawduel-cli": "dist/clawduel-cli.js"
  },
  "scripts": {
    "build": "tsc",
    "prepare": "npm run build",
    "prepublishOnly": "npm run build"
  }
}
```

### Queue Command with --timeout Support
```typescript
// In main() switch case for 'queue':
case 'queue': {
  const optArg = (flag: string): string | undefined => {
    const idx = args.indexOf(flag);
    return idx !== -1 && idx + 1 < args.length ? args[idx + 1] : undefined;
  };
  const timeoutSec = optArg('--timeout');
  const timeout = timeoutSec ? parseInt(timeoutSec, 10) : 3600;
  await cmdQueue(parseFloat(getArg('--bet-tier')), timeout);
  break;
}

// Updated cmdQueue signature:
async function cmdQueue(betTierUsdc: number, timeoutSeconds: number = 3600) {
  // ...existing code...
  const deadline = Math.floor(Date.now() / 1000) + timeoutSeconds;
  // ...rest unchanged...
}
```

### Updated Shebang
```typescript
#!/usr/bin/env node
/**
 * ClawDuel CLI
 * ...
 */
```

### tsconfig.json Updated
```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "CommonJS",
    "declaration": true,
    "outDir": "./dist",
    "rootDir": ".",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true
  },
  "include": ["src/**/*", "clawduel-cli.ts"],
  "exclude": ["node_modules", "dist"]
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `#!/usr/bin/env npx tsx` shebang | `#!/usr/bin/env node` with compiled JS | Standard practice | No tsx runtime dependency for global install |
| `prepare` script for npm link | Same -- still standard | Long-standing | Ensures build runs before link |

**Deprecated/outdated:**
- Using `tsx` as a global binary runtime: Works for development but adds startup latency and requires tsx installed globally. Standard practice is compiled JS for `bin` entries.

## Open Questions

1. **register-agent.ts compilation**
   - What we know: `register-agent.ts` also lives at root. Including it in compilation is optional since it's a reference script, not a binary.
   - What's unclear: Whether it should also be compiled or left as-is.
   - Recommendation: Exclude it from compilation (it's a dev reference). Add to `exclude` or just don't add to `include`.

2. **SDK import path after rootDir change**
   - What we know: Changing `rootDir` to `"."` moves SDK output from `dist/index.js` to `dist/src/index.js`.
   - What's unclear: Whether any external consumers depend on the current `dist/index.js` path.
   - Recommendation: Update `main` and `types` in package.json. Since this is pre-v1 agent skill work, external consumers can adapt.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Manual testing (no automated test framework in project) |
| Config file | none |
| Quick run command | `npm run build` |
| Full suite command | `npm run build && npm link && clawduel-cli help` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CLIP-01 | package.json has bin field | smoke | `node -e "const p=require('./package.json'); if(!p.bin || !p.bin['clawduel-cli']) process.exit(1)"` | N/A (inline) |
| CLIP-02 | npm link makes clawduel-cli available | smoke | `npm run build && npm link && which clawduel-cli` | N/A (inline) |
| CLIP-03 | clawduel-cli help exits 0 | smoke | `clawduel-cli help; echo $?` | N/A (inline) |
| CLIP-04 | All commands work via binary | manual-only | Manual -- requires wallet, RPC, backend | N/A |
| QUES-01 | queue accepts --timeout flag | smoke | `grep -q "timeout" dist/clawduel-cli.js` | N/A (inline) |
| QUES-02 | Default timeout is 3600 | smoke | `grep -q "3600" dist/clawduel-cli.js` | N/A (inline) |

### Sampling Rate
- **Per task commit:** `npm run build` (must succeed)
- **Per wave merge:** `npm run build && npm link && clawduel-cli help`
- **Phase gate:** Full suite green before verify

### Wave 0 Gaps
- No test framework needed -- this phase is verified via build success and smoke commands
- Existing `npm run build` is the gatekeeper

## Sources

### Primary (HIGH confidence)
- Project source code: `clawduel-cli.ts`, `package.json`, `tsconfig.json` -- direct inspection
- npm docs on `bin` field: standard Node.js packaging pattern, well-documented
- TypeScript compiler behavior with `rootDir` and shebangs: verified in TypeScript 5.x release notes

### Secondary (MEDIUM confidence)
- npm link behavior with `prepare` scripts: standard npm lifecycle, verified through long-standing documentation

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - no new dependencies, all patterns are well-established npm/TypeScript conventions
- Architecture: HIGH - straightforward tsconfig change and bin field addition; two clear approaches identified with recommendation
- Pitfalls: HIGH - all pitfalls derived from direct code inspection and known npm/TypeScript behaviors

**Research date:** 2026-03-18
**Valid until:** 2026-04-18 (stable domain, nothing fast-moving)
