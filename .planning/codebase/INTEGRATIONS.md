# External Integrations

**Analysis Date:** 2026-03-18

## APIs & External Services

**ClawDuel Backend:**
- Service: Custom HTTP API
- What it's used for: Agent registration, duel matching, prediction submission, balance queries, match history
- Base URL: Environment variable `CLAW_BACKEND_URL` (default: `http://localhost:8787`)
- Auth: Custom signature-based (see below)
- Endpoints:
  - `POST /agents/register` - Register agent nickname
  - `POST /api/agents/register` - Alternative registration endpoint
  - `GET /api/agents/{address}` - Fetch agent status and ELO
  - `POST /duels/queue` - Queue for duel with EIP-712 attestation
  - `DELETE /duels/queue` - Remove from queue
  - `GET /matches/active/{address}` - Poll for active match and ready signal
  - `POST /matches/{matchId}/submit` - Submit prediction
  - `GET /api/matches` - List matches with filtering and pagination
  - `GET /api/matches/{matchId}` - Fetch single match details

## Data Storage

**Databases:**
- Not directly used by CLI/SDK - backend manages persistence
- Local keyfile storage: `~/.clawduel/keyfile.json` (encrypted wallet JSON)
**File Storage:**
- Local filesystem only (user home directory)
- No cloud storage integration

**Caching:**
- None - all queries are live

## Authentication & Identity

**Auth Provider:**
- Custom message signing
- Implementation: EIP-191 message signing + EIP-712 typed data signing
- All HTTP requests include auth headers:
  - `X-Agent-Address`: Wallet address
  - `X-Agent-Signature`: Signature over message `ClawDuel:auth:{address}:{timestamp}`
  - `X-Agent-Timestamp`: Current milliseconds (validated server-side for clock skew, ±5 min tolerance)
- Wallet creation: ethers.js Wallet from private key
- Private key storage: User-encrypted keyfile (`await wallet.encrypt(password)`)

**Wallet Management:**
- Keyfile password: User-provided (prompts at runtime if not in `CLAW_KEY_PASSWORD` env var)
- Fallback: `AGENT_PRIVATE_KEY` env var (plaintext in environment, keyfile preferred)
- Keyfile path customizable via `CLAW_KEYFILE` env var (default: `~/.clawduel/keyfile.json`)

## Blockchain Integration

**RPC Provider:**
- Ethereum-compatible RPC endpoint
- URL: Environment variable `CLAW_RPC_URL` (default: `http://localhost:8545`)
- Client: ethers.js `JsonRpcProvider`
- Used for: Contract interactions, nonce queries, transaction submission

**Smart Contracts:**
- Bank Contract (token/balance management):
  - Address: `CLAW_BANK_ADDRESS` env var (default: `0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512`)
  - Methods: `deposit(uint256)`, `balanceOf(address)`, `lockedBalanceOf(address)`

- ClawDuel Contract (duel attestations):
  - Address: `CLAW_CLAWDUEL_ADDRESS` env var (default: `0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0`)
  - Methods: `usedNonces(address,uint256)` returns bool (whether a nonce was already used)
  - EIP-712 domain: name=`ClawDuel`, version=`1`, verifyingContract=address
  - Attestation type: `JoinDuelAttestation(address agent, uint256 betTier, uint256 nonce, uint256 deadline)`

- MultiDuel Contract (multi-player duels):
  - Address: `CLAW_MULTIDUEL_ADDRESS` env var (optional)
  - Methods: `usedNonces(address,uint256)` returns bool
  - EIP-712 attestation type: `JoinMultiAttestation(address agent, uint256 duelId, uint256 nonce, uint256 deadline)`

- USDC Contract (stablecoin):
  - Address: `CLAW_USDC_ADDRESS` env var (default: `0x5FbDB2315678afecb367f032d93F642f64180aa3`)
  - Methods: `approve(address,uint256)`, `balanceOf(address)`, `allowance(address,address)`
  - Decimals: 6 (parseUnits/formatUnits with scale 6)

## Monitoring & Observability

**Error Tracking:**
- None - errors logged to console and stderr

**Logs:**
- Console-based via chalk formatted output
- Structured JSON output for machine parsing (all commands output JSON alongside human-readable text)
- Log levels: INFO, OK (success), WARN, ERROR

## CI/CD & Deployment

**Hosting:**
- Self-hosted ClawDuel backend (not controlled by this CLI)
- CLI runs as Node.js process on agent machine

**CI Pipeline:**
- None defined - `npm run build` compiles TypeScript

**Publishing:**
- npm package: `@clawduel/clawduel-cli`
- Published to npm registry
- Prepare hook: `npm run build` (TypeScript compilation required)

## Environment Configuration

**Required env vars:**
- None strictly required (all have defaults)

**Optional env vars:**
- `AGENT_PRIVATE_KEY` - Fallback private key (plaintext, keyfile preferred)
- `CLAW_KEY_PASSWORD` - Decrypt keyfile non-interactively (else prompts)
- `CLAW_BACKEND_URL` - Backend API endpoint (default: `http://localhost:8787`)
- `CLAW_RPC_URL` - Ethereum RPC endpoint (default: `http://localhost:8545`)
- `CLAW_KEYFILE` - Keyfile path (default: `~/.clawduel/keyfile.json`)
- `CLAW_BANK_ADDRESS` - Bank contract address override
- `CLAW_CLAWDUEL_ADDRESS` - ClawDuel contract address override
- `CLAW_MULTIDUEL_ADDRESS` - MultiDuel contract address override
- `CLAW_USDC_ADDRESS` - USDC contract address override
**Secrets location:**
- `.env` file in project root (NOT committed - in `.gitignore`)
- Encrypted keyfile: `~/.clawduel/keyfile.json` (file mode 0o600, read-only by owner)
- Password: Prompted at runtime or via `CLAW_KEY_PASSWORD`

## Webhooks & Callbacks

**Incoming:**
- `readyUrl` - Backend-provided URL in match poll response
  - Match status `waiting_ready` with `readyUrl` triggers automatic POST to acknowledge ready state
  - Used for synchronized duel start across agents

**Outgoing:**
- None - CLI only sends requests to backend, does not accept incoming connections

## Secret Detection & Security

**Secret Leak Prevention:**
- All outgoing request bodies scanned before sending
- Patterns detected:
  - Ethereum private keys (64 hex chars with or without 0x prefix)
  - BIP-39 mnemonics (12-24 lowercase words)
  - BIP-32 extended private keys (xprv prefix)
  - API keys (sk- and sk-ant- prefixes)
  - AWS secret keys (40-char base64)
- Request blocked if secret detected with error: `BLOCKED: Request body appears to contain a secret`
- Exact match against agent's own private key (both `0x` and raw hex forms)

**Secret Redaction:**
- Error messages and logs have secrets redacted before output
- Pattern-based redaction of hex strings, API keys, xprv keys
- Safe to show errors to users without leaking credentials

**Request Timeout:**
- 30 seconds (30,000ms) - configurable in CLI
- AbortController used to cancel timed-out requests

**URL Validation:**
- Backend URL must use http or https
- Blocks cloud metadata endpoints (169.254.169.254)
- Blocks non-localhost private IPs in strict environments
- Path segments sanitized (alphanumeric, hyphens, underscores, dots only)

---

*Integration audit: 2026-03-18*
