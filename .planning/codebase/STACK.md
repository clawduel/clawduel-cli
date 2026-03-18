# Technology Stack

**Analysis Date:** 2026-03-18

## Languages

**Primary:**
- TypeScript 5.3.3 - CLI application and SDK library

## Runtime

**Environment:**
- Node.js (no explicit version lock, see npm install)

**Package Manager:**
- npm 10.x (implied by npm 16.4.0 in lockfile)
- Lockfile: `package-lock.json` present

## Frameworks

**Core:**
- ethers.js 6.13.0 - Ethereum wallet, contract interaction, RPC communication, EIP-712 signing

**CLI:**
- chalk 4.1.2 - Terminal output formatting and colors

**Configuration:**
- dotenv 16.4.0 - Environment variable loading from `.env` files

**Build/Dev:**
- TypeScript 5.3.3 - Compilation and type checking

## Key Dependencies

**Critical:**
- ethers.js 6.13.0 - Why it matters: Core to all blockchain interaction (wallet creation, contract calls, signing, RPC communication)

**Dev:**
- @types/node 20.11.0 - TypeScript definitions for Node.js APIs (fs, path, os, readline, crypto)

## Configuration

**Environment:**
- Loaded via `dotenv.config()` in `src/index.ts`
- `.env` file location: Project root (added to `.gitignore`)
- Environment variables are NOT committed to git

**Build:**
- `tsconfig.json`: Targets ES2020, compiles to CommonJS, output to `./dist/`
- TypeScript strict mode enabled
- Declaration files generated (`*.d.ts`)

## Platform Requirements

**Development:**
- Node.js (no version pinned in package.json)
- npm with access to ethers.js and dependencies
- TypeScript compiler available via npm

**Production:**
- Node.js runtime (ES2020 compatibility)
- Access to Ethereum RPC endpoint (configured via `CLAW_RPC_URL` env var)
- Access to ClawDuel backend HTTP endpoint (configured via `CLAW_BACKEND_URL` env var)

## Entry Points

**CLI:**
- `clawduel-cli.ts` - Executable CLI script (shebang: `#!/usr/bin/env npx tsx`)
- Run with: `npx tsx clawduel-cli.ts <command>`
- Supports: init, register, deposit, balance, queue, dequeue, poll, submit, status, matches, match, help

**SDK/Library:**
- `src/index.ts` - Exports `ClawClient` class and security utilities
- Main export: `ClawClient` - Programmatic blockchain interaction
- Published as npm package: `@clawduel/agent-sdk` (version 2.0.0)
- Main entry: `dist/index.js`, Types: `dist/index.d.ts`

**Register Script:**
- `register-agent.ts` - One-off agent registration utility (uses ethers directly)

## Build Output

- TypeScript compiles to: `./dist/`
- Source root: `./src/`
- Declaration files generated for library consumers
- No build minification or optimization configured

---

*Stack analysis: 2026-03-18*
