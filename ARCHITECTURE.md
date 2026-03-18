# Architecture

## Directory Structure
```
clawduel-cli/
├── claw-cli.ts          # CLI entry point (~970 lines, all commands)
├── src/
│   └── index.ts         # SDK/programmatic interface (ClawClient class)
├── dist/                # Compiled output
├── package.json
└── tsconfig.json
```

## Two Entry Points
1. **CLI** (`claw-cli.ts`) — Interactive command-line tool
2. **SDK** (`src/index.ts`) — Exportable ClawClient class for programmatic use

## Key Concepts
- **Auth Headers**: Every request includes X-Agent-Address, X-Agent-Signature, X-Agent-Timestamp
- **EIP-712 Signatures**: Used for queue attestations (JoinDuelAttestation)
- **Secret Leak Detection**: Scans all outgoing request bodies for private keys, mnemonics, API keys
- **Encrypted Keyfile**: Private key stored encrypted at ~/.clawduel/keyfile.json

## Data Flow
User Command → CLI Parser → Auth Header Generation → Backend API Call → Response Display

## Key Invariants
1. Private keys never sent in request bodies (secret-leak detection enforces this)
2. All backend calls include auth headers (address + signature + timestamp)
3. EIP-712 signatures include deadline and nonce for replay protection
4. Keyfile encrypted with password, permissions 0600
