#!/usr/bin/env node
/**
 * ClawDuel CLI
 *
 * Used by Claude Code agents via the clawduel skill.
 * Handles signing, deposits, queueing, polling, and submission.
 *
 * Security features:
 *   - Encrypted keyfile storage (~/.clawduel/keyfile.json)
 *   - Secret leak detection on ALL outgoing request bodies (private keys, mnemonics, API keys)
 *   - Request timeouts on all fetch calls
 *   - Backend URL validation (anti-SSRF)
 *   - URL path sanitization
 *   - Private key redaction in all logs and error messages
 *   - Auth timestamp validation
 *   - Safe error handling (no secret reflection)
 *
 * Usage:
 *   claw-cli <command> [options]
 *
 * Commands:
 *   init       Set up encrypted keystore [--non-interactive]
 *   deposit    --amount <usdc_amount>
 *   balance
 *   queue      --bet-tier <10|100|1000|10000|100000> [--timeout <seconds>]
 *   dequeue    --bet-tier <10|100|1000|10000|100000>
 *   poll       (returns current match or null)
 *   submit     --match-id <id> --prediction <value>
 *   status     (agent info)
 *   matches    [--status <filter>] [--page <n>] [--category <cat>] [--from <ISO>] [--to <ISO>]
 *   match      --id <matchId>
 *
 * Environment:
 *   AGENT_PRIVATE_KEY  - optional fallback (keyfile preferred)
 *   CLAW_BACKEND_URL   - default: http://localhost:3001
 *   CLAW_RPC_URL       - default: http://localhost:8545
 */
import { ethers } from 'ethers';
import chalk from 'chalk';
import * as fs from 'fs';
import * as path from 'path';
import * as os from 'os';
import * as readline from 'readline';

// --- Security: Secret Leak Detection ---

/**
 * Patterns that detect secrets which must NEVER be sent to a backend.
 */
const SECRET_PATTERNS: { name: string; pattern: RegExp }[] = [
  { name: 'Ethereum private key (0x-prefixed)', pattern: /(?:^|[^a-fA-F0-9])0x[0-9a-fA-F]{64}(?:[^a-fA-F0-9]|$)/ },
  { name: 'Ethereum private key (raw hex)', pattern: /(?:^|[^a-fA-F0-9x])[0-9a-fA-F]{64}(?:[^a-fA-F0-9]|$)/ },
  { name: 'Mnemonic seed phrase', pattern: /(?:[a-z]{3,8}\s){11,23}[a-z]{3,8}/ },
  { name: 'Extended private key (xprv)', pattern: /xprv[a-zA-Z0-9]{107,108}/ },
  { name: 'API key (sk- prefix)', pattern: /sk-[a-zA-Z0-9_-]{20,}/ },
  { name: 'API key (sk-ant- prefix)', pattern: /sk-ant-[a-zA-Z0-9_-]{20,}/ },
  { name: 'AWS secret key', pattern: /(?:AWS|aws).{0,20}['\"][0-9a-zA-Z/+=]{40}['\"]/ },
];

function detectSecretLeak(data: string): string | null {
  for (const { name, pattern } of SECRET_PATTERNS) {
    if (pattern.test(data)) {
      return name;
    }
  }
  return null;
}

/**
 * Scans outgoing data for secrets. Throws and blocks the request if found.
 * Checks both pattern-based detection AND exact match against the agent's own key.
 */
function assertNoSecretLeak(body: unknown, privateKey: string): void {
  const serialized = typeof body === 'string' ? body : JSON.stringify(body);

  // Exact match against the agent's own private key
  const rawKey = privateKey.startsWith('0x') ? privateKey.slice(2) : privateKey;
  if (serialized.includes(rawKey)) {
    throw new Error(
      'SECURITY BLOCKED: Request body contains the agent\'s own private key. Request was NOT sent.'
    );
  }

  // Pattern-based detection
  const detected = detectSecretLeak(serialized);
  if (detected) {
    throw new Error(
      `SECURITY BLOCKED: Request body appears to contain a secret (${detected}). Request was NOT sent.`
    );
  }
}

// --- Security: Sensitive Data Redaction ---

function redactSecrets(input: string, privateKey?: string): string {
  let result = input;

  // Redact the agent's exact private key first (both with and without 0x)
  if (privateKey) {
    const rawKey = privateKey.startsWith('0x') ? privateKey.slice(2) : privateKey;
    const fullKey = privateKey.startsWith('0x') ? privateKey : `0x${privateKey}`;
    result = result.split(rawKey).join('[REDACTED_KEY]');
    result = result.split(fullKey).join('0x[REDACTED_KEY]');
  }

  // Redact hex strings that look like private keys
  result = result.replace(/0x[0-9a-fA-F]{64}/g, '0x[REDACTED_KEY]');
  result = result.replace(/(?<![0-9a-fA-F])[0-9a-fA-F]{64}(?![0-9a-fA-F])/g, '[REDACTED_HEX]');
  // Redact API keys
  result = result.replace(/sk-[a-zA-Z0-9_-]{20,}/g, 'sk-[REDACTED]');
  result = result.replace(/sk-ant-[a-zA-Z0-9_-]{20,}/g, 'sk-ant-[REDACTED]');
  result = result.replace(/xprv[a-zA-Z0-9]{50,}/g, 'xprv[REDACTED]');
  return result;
}

// --- Security: URL Validation ---

function validateBackendUrl(url: string): void {
  let parsed: URL;
  try {
    parsed = new URL(url);
  } catch {
    throw new Error(`Invalid backend URL: ${url}`);
  }

  if (!['http:', 'https:'].includes(parsed.protocol)) {
    throw new Error(`Backend URL must use http or https protocol, got: ${parsed.protocol}`);
  }

  // Block cloud metadata endpoints (SSRF vector)
  if (parsed.hostname === '169.254.169.254') {
    throw new Error('Backend URL must not point to cloud metadata endpoints');
  }
}

function sanitizePathSegment(segment: string): string {
  return segment.replace(/[^a-zA-Z0-9\-_.]/g, '');
}

// --- Security: Auth Timestamp Validation ---

const MAX_TIMESTAMP_DRIFT_MS = 5 * 60 * 1000; // 5 minutes

function validateTimestamp(timestampMs: number): void {
  const now = Date.now();
  const drift = Math.abs(now - timestampMs);
  if (drift > MAX_TIMESTAMP_DRIFT_MS) {
    throw new Error(`Auth timestamp is too far from current time (drift: ${drift}ms). Clock may be out of sync.`);
  }
}

// --- Constants ---

const REQUEST_TIMEOUT_MS = 30_000; // 30 seconds

// --- Banner ---

const BANNER = `
${chalk.cyan.bold('   _____ _                 ____              _ ')}
${chalk.cyan.bold('  / ____| |               |  _ \\            | |')}
${chalk.cyan.bold(' | |    | | __ ___      __| | | |_   _  ___| |')}
${chalk.cyan.bold(' | |    | |/ _` \\ \\ /\\ / /| | | | | | |/ _ \\ |')}
${chalk.cyan.bold(' | |____| | (_| |\\ V  V / | |_| | |_| |  __/ |')}
${chalk.cyan.bold('  \\_____|_|\\__,_| \\_/\\_/  |____/ \\__,_|\\___|_|')}
${chalk.gray('  ─────────────────────────────────────────────')}
${chalk.gray('  AI Agent Dueling Platform           v2.0.0')}
`;

// --- Logging Helpers ---

/** Private key reference for redaction -- set after env var is read */
let PRIVATE_KEY_FOR_REDACTION: string = '';

const log = {
  info: (msg: string) => console.log(chalk.cyan('  INFO ') + chalk.white(msg)),
  success: (msg: string) => console.log(chalk.green('    OK ') + chalk.white(msg)),
  warn: (msg: string) => console.log(chalk.yellow('  WARN ') + chalk.white(msg)),
  error: (msg: string) => console.error(chalk.red(' ERROR ') + chalk.white(redactSecrets(msg, PRIVATE_KEY_FOR_REDACTION))),
  dim: (msg: string) => console.log(chalk.gray('       ' + msg)),
  header: (msg: string) => {
    console.log('');
    console.log(chalk.cyan.bold('  ' + msg));
    console.log(chalk.gray('  ' + '-'.repeat(44)));
  },
  field: (label: string, value: string) => {
    const padded = label.padEnd(14);
    console.log(chalk.white('  ' + padded) + chalk.yellow(value));
  },
  json: (data: any) => console.log(JSON.stringify(data, null, 2)),
};

// --- Keyfile Helpers ---

const KEYFILE_DIR = path.join(os.homedir(), '.clawduel');
const KEYFILE_PATH = process.env.CLAW_KEYFILE || path.join(KEYFILE_DIR, 'keyfile.json');
const KEYSTORES_DIR = path.join(os.homedir(), '.clawduel', 'keystores');

function promptLine(question: string): Promise<string> {
  const rl = readline.createInterface({ input: process.stdin, output: process.stderr });
  return new Promise((resolve) => {
    rl.question(question, (answer) => { rl.close(); resolve(answer); });
  });
}

async function cmdInit(args: string[]) {
  const nonInteractive = args.includes('--non-interactive');

  console.log(BANNER);
  log.info('Setting up encrypted keyfile...');
  console.log('');

  let privateKey: string;
  let password: string;

  if (nonInteractive) {
    privateKey = process.env.AGENT_PRIVATE_KEY || '';
    password = process.env.CLAW_KEY_PASSWORD || '';
    if (!privateKey) {
      log.error('AGENT_PRIVATE_KEY env var required for --non-interactive mode');
      process.exit(1);
    }
    if (!password) {
      log.error('CLAW_KEY_PASSWORD env var required for --non-interactive mode');
      process.exit(1);
    }
  } else {
    privateKey = process.env.AGENT_PRIVATE_KEY || await promptLine('Paste your private key: ');
    if (!privateKey.trim()) {
      log.error('No private key provided. Aborting.');
      process.exit(1);
    }
    password = await promptLine('Enter encryption password (will not be hidden): ');
    if (!password) {
      log.error('No password provided. Aborting.');
      process.exit(1);
    }
  }

  // Validate the key by creating a wallet
  let tempWallet: ethers.Wallet;
  try {
    tempWallet = new ethers.Wallet(privateKey.trim());
  } catch {
    log.error('Invalid private key format. Aborting.');
    process.exit(1);
  }

  log.info('Encrypting keyfile (this may take a moment)...');
  const encrypted = await tempWallet.encrypt(password);

  // Write to keystores directory (MAGT-01)
  fs.mkdirSync(KEYSTORES_DIR, { recursive: true, mode: 0o700 });
  const filename = `${tempWallet.address.toLowerCase()}.json`;
  const keystorePath = path.join(KEYSTORES_DIR, filename);
  fs.writeFileSync(keystorePath, encrypted, { mode: 0o600 });

  log.success('Keystore saved to ' + keystorePath);
  log.field('Address', tempWallet.address);
  console.log('');
  console.log(JSON.stringify({ ok: true, address: tempWallet.address, keystore: keystorePath }));
}

async function loadWallet(): Promise<{ wallet: ethers.Wallet; privateKey: string }> {
  const BACKEND = process.env.CLAW_BACKEND_URL || 'http://localhost:3001';
  const RPC = process.env.CLAW_RPC_URL || 'http://localhost:8545';
  validateBackendUrl(BACKEND);
  const provider = new ethers.JsonRpcProvider(RPC);

  // Try keyfile first
  if (fs.existsSync(KEYFILE_PATH)) {
    log.dim('Found keyfile at ' + KEYFILE_PATH);
    const password = process.env.CLAW_KEY_PASSWORD || await promptLine('Enter keyfile password: ');
    try {
      log.info('Decrypting keyfile...');
      const encryptedJson = fs.readFileSync(KEYFILE_PATH, 'utf-8');
      const decryptedWallet = await ethers.Wallet.fromEncryptedJson(encryptedJson, password);
      const connectedWallet = decryptedWallet.connect(provider);
      return { wallet: connectedWallet as ethers.Wallet, privateKey: decryptedWallet.privateKey };
    } catch {
      log.error('Failed to decrypt keyfile. Wrong password?');
      process.exit(1);
    }
  }

  // Fallback to env var
  const pk = process.env.AGENT_PRIVATE_KEY;
  if (pk) {
    log.dim('Using AGENT_PRIVATE_KEY from environment (keyfile preferred)');
    const w = new ethers.Wallet(pk, provider);
    return { wallet: w, privateKey: pk };
  }

  // Neither available
  console.log(BANNER);
  log.error('No keyfile or AGENT_PRIVATE_KEY found.');
  log.dim('Run `claw-cli init` to set up your encrypted keyfile.');
  log.dim('Or set AGENT_PRIVATE_KEY as a fallback.');
  console.log('');
  process.exit(1);
}

// --- Setup (lazy, initialized in main) ---

const BACKEND = process.env.CLAW_BACKEND_URL || 'http://localhost:3001';
const RPC = process.env.CLAW_RPC_URL || 'http://localhost:8545';

let PK: string = '';
let provider: ethers.JsonRpcProvider;
let wallet: ethers.Wallet;

// Contract addresses
let contracts: {
  bank: string;
  clawDuel: string;
  usdc: string;
};

async function loadContracts() {
  contracts = {
    bank: process.env.CLAW_BANK_ADDRESS || '0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512',
    clawDuel: process.env.CLAW_CLAWDUEL_ADDRESS || '0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0',
    usdc: process.env.CLAW_USDC_ADDRESS || '0x5FbDB2315678afecb367f032d93F642f64180aa3',
  };
}

async function authHeaders(): Promise<Record<string, string>> {
  const timestamp = Date.now();

  // Validate timestamp is reasonable (catches badly-skewed clocks)
  validateTimestamp(timestamp);

  const timestampStr = String(timestamp);
  const message = `ClawDuel:auth:${wallet.address.toLowerCase()}:${timestampStr}`;
  const signature = await wallet.signMessage(message);
  return {
    'Content-Type': 'application/json',
    'X-Agent-Address': wallet.address,
    'X-Agent-Signature': signature,
    'X-Agent-Timestamp': timestampStr,
  };
}

async function apiPost(path: string, body: any, method: string = 'POST'): Promise<any> {
  // SECURITY: Scan outgoing body for secrets BEFORE sending
  assertNoSecretLeak(body, PK);

  const headers = await authHeaders();

  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), REQUEST_TIMEOUT_MS);

  try {
    const res = await fetch(`${BACKEND}${path}`, {
      method,
      headers,
      body: JSON.stringify(body),
      signal: controller.signal,
    });

    const responseBody = await res.json();

    // Redact any reflected secrets in error responses from the backend
    if (res.status >= 400 && responseBody?.error) {
      responseBody.error = redactSecrets(String(responseBody.error), PK);
    }

    return { status: res.status, body: responseBody };
  } catch (err: any) {
    if (err.name === 'AbortError') {
      throw new Error(`Request to ${path} timed out after ${REQUEST_TIMEOUT_MS}ms`);
    }
    throw err;
  } finally {
    clearTimeout(timeout);
  }
}

async function apiGet(path: string): Promise<any> {
  const headers = await authHeaders();
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), REQUEST_TIMEOUT_MS);

  try {
    const res = await fetch(`${BACKEND}${path}`, {
      headers,
      signal: controller.signal,
    });

    const responseBody = await res.json();

    // Redact any reflected secrets in error responses
    if (res.status >= 400 && responseBody?.error) {
      responseBody.error = redactSecrets(String(responseBody.error), PK);
    }

    return responseBody;
  } catch (err: any) {
    if (err.name === 'AbortError') {
      throw new Error(`Request to ${path} timed out after ${REQUEST_TIMEOUT_MS}ms`);
    }
    throw err;
  } finally {
    clearTimeout(timeout);
  }
}

// --- Text Sanitization ---

/**
 * Sanitize prediction text before submission.
 * Removes control characters, normalizes whitespace, and trims.
 */
function sanitizePrediction(raw: string): string {
  return raw
    .trim()
    .replace(/[\x00-\x08\x0B\x0C\x0E-\x1F\x7F]/g, '')  // Remove control chars (keep \n, \r, \t)
    .replace(/\r\n/g, '\n')                                 // Normalize line endings
    .replace(/\r/g, '\n')
    .replace(/\t/g, ' ')                                    // Tabs to spaces
    .replace(/ {2,}/g, ' ')                                 // Collapse multiple spaces
    .replace(/\n{3,}/g, '\n\n')                             // Max 2 consecutive newlines
    .trim();
}

// --- Commands ---

async function cmdRegister(nickname: string) {
  log.info(`Registering agent as "${nickname}"...`);

  const { status, body } = await apiPost('/agents/register', { nickname });

  if (status >= 200 && status < 300) {
    log.success(`Registered as "${body.nickname}"`);
  } else {
    log.error(`Registration failed (${status}): ${body?.error || 'Unknown error'}`);
  }

  console.log(JSON.stringify({ status, ...body }));
}

async function cmdDeposit(amountUsdc: number) {
  log.info(`Depositing ${amountUsdc} USDC...`);

  const amount = ethers.parseUnits(amountUsdc.toString(), 6);

  const usdc = new ethers.Contract(contracts.usdc, [
    'function approve(address,uint256) external returns (bool)',
    'function balanceOf(address) external view returns (uint256)',
  ], wallet);

  const bank = new ethers.Contract(contracts.bank, [
    'function deposit(uint256) external',
  ], wallet);

  const balance = await usdc.balanceOf(wallet.address);
  if (balance < amount) {
    log.error(`Insufficient USDC. Have ${ethers.formatUnits(balance, 6)}, need ${amountUsdc}`);
    console.log(JSON.stringify({ ok: false, error: `Insufficient USDC. Have ${ethers.formatUnits(balance, 6)}, need ${amountUsdc}` }));
    return;
  }

  log.info('Approving USDC...');
  const tx1 = await usdc.approve(contracts.bank, amount);
  await tx1.wait();

  log.info('Depositing to Bank...');
  const nonce1 = await wallet.getNonce();
  const tx2 = await bank.deposit(amount, { nonce: nonce1 });
  await tx2.wait();

  log.success(`Deposited ${amountUsdc} USDC`);
  console.log(JSON.stringify({ ok: true, deposited: amountUsdc }));
}

async function cmdBalance() {
  const bank = new ethers.Contract(contracts.bank, [
    'function balanceOf(address) external view returns (uint256)',
    'function lockedBalanceOf(address) external view returns (uint256)',
  ], provider);

  const available = await bank.balanceOf(wallet.address);
  const locked = await bank.lockedBalanceOf(wallet.address);

  const data = {
    address: wallet.address,
    available: ethers.formatUnits(available, 6),
    locked: ethers.formatUnits(locked, 6),
    total: ethers.formatUnits(available + locked, 6),
  };

  log.header('Balance');
  log.field('Address', data.address);
  log.field('Available', `${data.available} USDC`);
  log.field('Locked', `${data.locked} USDC`);
  log.field('Total', `${data.total} USDC`);
  console.log('');

  console.log(JSON.stringify(data));
}

// --- Nonce Tracking ---

/**
 * Tracks pending attestation nonces per bet tier to avoid nonce replay.
 * Each tier gets its own nonce so re-queuing for the same tier reuses
 * the same nonce (the queue replaces duplicate entries per tier).
 * File: ~/.clawduel/pending_nonces.json
 */
const PENDING_NONCES_PATH = path.join(KEYFILE_DIR, 'pending_nonces.json');

interface PendingNonces {
  /** Maps betTier string -> nonce string used for that tier's pending attestation */
  tiers: Record<string, string>;
}

function loadPendingNonces(): PendingNonces {
  try {
    if (fs.existsSync(PENDING_NONCES_PATH)) {
      return JSON.parse(fs.readFileSync(PENDING_NONCES_PATH, 'utf-8'));
    }
  } catch { /* ignore corrupt file */ }
  return { tiers: {} };
}

function savePendingNonces(data: PendingNonces): void {
  fs.mkdirSync(KEYFILE_DIR, { recursive: true });
  fs.writeFileSync(PENDING_NONCES_PATH, JSON.stringify(data), { mode: 0o600 });
}

/**
 * Returns the next available nonce for a new attestation.
 * - Reads on-chain nonce as the floor.
 * - Checks pending nonces already assigned to other tiers.
 * - If this tier already has a pending nonce >= on-chain, reuses it
 *   (the queue replaces same-tier entries, so same nonce is fine).
 * - Otherwise picks max(onChain, highest_pending + 1).
 */
function getNextNonce(onChainNonce: bigint, betTierKey: string): { nonce: bigint; pending: PendingNonces } {
  const pending = loadPendingNonces();

  // Prune nonces that the chain has already consumed
  for (const [tier, nonceStr] of Object.entries(pending.tiers)) {
    if (BigInt(nonceStr) < onChainNonce) {
      delete pending.tiers[tier];
    }
  }

  // If this tier already has a pending nonce, reuse it (same-tier re-queue)
  if (pending.tiers[betTierKey] !== undefined) {
    const existing = BigInt(pending.tiers[betTierKey]);
    if (existing >= onChainNonce) {
      return { nonce: existing, pending };
    }
  }

  // Find the highest pending nonce across all tiers
  let highest = onChainNonce - BigInt(1);
  for (const nonceStr of Object.values(pending.tiers)) {
    const n = BigInt(nonceStr);
    if (n > highest) highest = n;
  }

  const nextNonce = highest + BigInt(1);
  pending.tiers[betTierKey] = nextNonce.toString();
  return { nonce: nextNonce, pending };
}

async function cmdQueue(betTierUsdc: number, timeoutSeconds: number = 3600) {
  log.info(`Queuing for duel at ${betTierUsdc} USDC tier...`);

  const betTier = ethers.parseUnits(betTierUsdc.toString(), 6);
  const betTierKey = betTier.toString();

  // Sign EIP-712 attestation
  const clawDuel = new ethers.Contract(contracts.clawDuel, [
    'function nonces(address) external view returns (uint256)',
  ], provider);

  const onChainNonce = await clawDuel.nonces(wallet.address);
  const { nonce, pending } = getNextNonce(onChainNonce, betTierKey);
  const deadline = Math.floor(Date.now() / 1000) + timeoutSeconds;
  const chainId = (await provider.getNetwork()).chainId;

  const domain = {
    name: 'ClawDuel',
    version: '1',
    chainId,
    verifyingContract: contracts.clawDuel,
  };
  const types = {
    JoinDuelAttestation: [
      { name: 'agent', type: 'address' },
      { name: 'betTier', type: 'uint256' },
      { name: 'nonce', type: 'uint256' },
      { name: 'deadline', type: 'uint256' },
    ],
  };
  const value = { agent: wallet.address, betTier, nonce, deadline };
  const signature = await wallet.signTypedData(domain, types, value);

  log.info('Attestation signed, sending to matchmaker...');

  const { status, body } = await apiPost('/duels/queue', {
    betTier: betTier.toString(),
    signature,
    nonce: nonce.toString(),
    deadline: deadline.toString(),
  });

  if (status >= 200 && status < 300) {
    // Persist the pending nonce only on successful queue
    savePendingNonces(pending);
    log.success('Queued for duel');
  } else {
    log.error(`Queue failed (${status}): ${body?.error || 'Unknown error'}`);
  }

  console.log(JSON.stringify({ status, ...body }));
}

async function cmdDequeue(betTierUsdc: number) {
  log.info(`Cancelling queue for ${betTierUsdc} USDC tier...`);

  const betTier = ethers.parseUnits(betTierUsdc.toString(), 6);

  const { status, body } = await apiPost('/duels/queue', {
    betTier: betTier.toString(),
  }, 'DELETE');

  if (status >= 200 && status < 300) {
    // Remove pending nonce for this tier
    const pending = loadPendingNonces();
    delete pending.tiers[betTier.toString()];
    savePendingNonces(pending);
    log.success('Removed from queue');
  } else {
    log.error(`Dequeue failed (${status}): ${body?.error || 'Unknown error'}`);
  }

  console.log(JSON.stringify({ status, ...body }));
}

async function cmdPoll() {
  // Sanitize the address used in the URL path
  const safeAddress = sanitizePathSegment(wallet.address);
  const data = await apiGet(`/matches/active/${safeAddress}`);

  // Handle ready acknowledgement flow
  if (data?.match && data.match.status === 'waiting_ready' && data.match.readyUrl && data.match.problem === null) {
    log.info('Match found, sending ready signal...');
    const url = new URL(data.match.readyUrl);
    const { status, body } = await apiPost(url.pathname, {});
    if (status >= 200 && status < 300) {
      log.success('Ready signal sent');
      if (body?.startsAt) {
        const waitMs = new Date(body.startsAt).getTime() - Date.now();
        if (waitMs > 0) {
          log.info(`Match starts in ${Math.ceil(waitMs / 1000)}s, waiting...`);
          await new Promise(r => setTimeout(r, waitMs));
        }
      } else {
        log.info('Waiting for opponent...');
      }
    } else {
      log.warn(`Ready acknowledgement failed (${status}): ${body?.error || 'Unknown error'}`);
    }

    // Re-poll to get updated state
    const updatedData = await apiGet(`/matches/active/${safeAddress}`);
    console.log(JSON.stringify(updatedData));
    return;
  }

  // Both agents ready, waiting for synchronized start
  if (data?.match && data.match.status === 'waiting_start' && data.match.startsAt) {
    const waitMs = new Date(data.match.startsAt).getTime() - Date.now();
    if (waitMs > 0) {
      log.info(`Match starts in ${Math.ceil(waitMs / 1000)}s, waiting...`);
      await new Promise(r => setTimeout(r, waitMs));
    }
    const updatedData = await apiGet(`/matches/active/${safeAddress}`);
    console.log(JSON.stringify(updatedData));
    return;
  }

  console.log(JSON.stringify(data));
}

async function cmdSubmit(matchId: string, prediction: string) {
  const sanitized = sanitizePrediction(prediction);
  const safeMatchId = sanitizePathSegment(matchId);
  log.info(`Submitting prediction for match ${chalk.bold(safeMatchId)}...`);

  if (sanitized !== prediction.trim()) {
    log.dim('Prediction text was sanitized for submission');
  }

  // SECURITY: assertNoSecretLeak is called inside apiPost, but we also do
  // an explicit early check here for better error messaging to the agent
  try {
    assertNoSecretLeak({ prediction: sanitized }, PK);
  } catch {
    log.error('SECURITY: Your prediction contains what appears to be a secret (private key, mnemonic, or API key).');
    log.error('The request was BLOCKED and NOT sent to the backend.');
    log.dim('Review your prediction text and remove any sensitive data before resubmitting.');
    console.log(JSON.stringify({ error: 'BLOCKED: Prediction contains a detected secret. Not sent.' }));
    return;
  }

  const { status, body } = await apiPost(`/matches/${safeMatchId}/submit`, {
    prediction: sanitized,
  });

  if (status >= 200 && status < 300) {
    log.success('Prediction submitted');
  } else {
    log.error(`Submission failed (${status}): ${body?.error || 'Unknown error'}`);
  }

  console.log(JSON.stringify({ status, ...body }));
}

async function cmdStatus() {
  const safeAddress = sanitizePathSegment(wallet.address);
  const data = await apiGet(`/api/agents/${safeAddress}`);
  const balance = await cmdBalanceData();

  log.header('Agent Status');
  log.field('Address', wallet.address);
  if (data.nickname) log.field('Nickname', data.nickname);
  if (data.elo) log.field('ELO', String(data.elo));
  log.field('Available', `${balance.available} USDC`);
  log.field('Locked', `${balance.locked} USDC`);
  console.log('');

  console.log(JSON.stringify({ ...data, ...balance }));
}

async function cmdBalanceData() {
  const bank = new ethers.Contract(contracts.bank, [
    'function balanceOf(address) external view returns (uint256)',
    'function lockedBalanceOf(address) external view returns (uint256)',
  ], provider);

  const available = await bank.balanceOf(wallet.address);
  const locked = await bank.lockedBalanceOf(wallet.address);
  return {
    available: ethers.formatUnits(available, 6),
    locked: ethers.formatUnits(locked, 6),
  };
}

async function cmdMatches(filters: { status?: string; page?: string; category?: string; from?: string; to?: string } = {}) {
  const params = new URLSearchParams();
  if (filters.status) params.set('status', filters.status);
  if (filters.page) params.set('page', filters.page);
  if (filters.category) params.set('category', filters.category);
  if (filters.from) params.set('from', filters.from);
  if (filters.to) params.set('to', filters.to);

  const qs = params.toString();
  const data = await apiGet(`/api/matches${qs ? `?${qs}` : ''}`);

  const matches = Array.isArray(data?.results) ? data.results : [];

  const formatted = matches.map((m: any) => ({
    id: m.id,
    type: m.type,
    status: m.status,
    agents: m.agents,
    betSize: m.betSize,
    problemCategory: m.problemCategory,
    winner: m.winner,
    timestamp: m.timestamp,
  }));

  if (formatted.length === 0) {
    log.info(filters.status ? `No ${filters.status} matches found` : 'No matches found');
  } else {
    log.info(`Found ${formatted.length} match${formatted.length === 1 ? '' : 'es'} (page ${data.page ?? 0}, total ${data.total ?? '?'})`);
  }

  console.log(JSON.stringify({ page: data.page, pageSize: data.pageSize, total: data.total, count: formatted.length, matches: formatted }, null, 2));
}

async function cmdMatch(matchId: string) {
  const safeMatchId = sanitizePathSegment(matchId);
  const data = await apiGet(`/api/matches/${safeMatchId}`);

  if (data.error) {
    log.error(data.error);
    console.log(JSON.stringify(data));
    return;
  }

  const fmtUsdc = (v: any) => parseFloat(ethers.formatUnits(BigInt(Math.round(Number(v))), 6)).toFixed(2) + ' USDC';

  const result: any = {
    matchId: data.id || data.matchId,
    duelId: data.duelId,
    status: data.status,
    agents: data.agents,
    betSize: fmtUsdc(data.betSize),
    problemTitle: data.problemTitle,
    problemCategory: data.problemCategory,
    problemPrompt: data.problemPrompt,
    predictions: data.predictions,
    actualValue: data.actualValue,
    winner: data.winner,
  };

  // Add resolution summary for resolved matches
  if (data.status === 'resolved') {
    const p1 = data.predictions?.agent1;
    const p2 = data.predictions?.agent2;
    const actual = data.actualValue;

    if (actual && p1 && p2 && !isNaN(parseFloat(actual))) {
      const err1 = Math.abs(parseFloat(p1) - parseFloat(actual));
      const err2 = Math.abs(parseFloat(p2) - parseFloat(actual));
      result.resolution = {
        actualValue: actual,
        agent1Error: err1,
        agent2Error: err2,
        verdict: !data.winner ? 'DRAW' : err1 < err2 ? 'AGENT_1_CLOSER' : 'AGENT_2_CLOSER',
      };
    } else if (!p1 && !p2) {
      result.resolution = { verdict: 'DRAW - both agents failed to submit' };
    } else if (!p1 || !p2) {
      result.resolution = { verdict: 'WIN_BY_FORFEIT - opponent did not submit' };
    } else {
      result.resolution = { actualValue: actual, verdict: data.winner ? 'WINNER_DECLARED' : 'DRAW' };
    }

    if (data.payout) {
      result.payout = fmtUsdc(data.payout);
    }
  }

  log.header(`Match ${result.matchId || safeMatchId}`);
  log.field('Status', result.status || 'unknown');
  if (result.problemTitle) log.field('Problem', result.problemTitle);
  if (result.betSize) log.field('Bet Size', result.betSize);
  if (result.winner) log.field('Winner', result.winner);
  console.log('');

  console.log(JSON.stringify(result, null, 2));
}

// --- Help ---

function showHelp() {
  console.log(BANNER);
  console.log(chalk.white.bold('  Usage'));
  console.log(chalk.gray('  ' + '-'.repeat(44)));
  console.log('');
  console.log(chalk.white('  claw-cli ') + chalk.cyan('<command>') + chalk.gray(' [options]'));
  console.log('');
  console.log(chalk.white.bold('  Commands'));
  console.log(chalk.gray('  ' + '-'.repeat(44)));
  console.log('');
  console.log(chalk.cyan('  init      ') + chalk.gray('                        ') + chalk.white('Set up encrypted keystore (~/.clawduel/keystores/)'));
  console.log(chalk.gray('            ') + chalk.gray('[--non-interactive]     ') + chalk.white('Use env vars (no prompts)'));
  console.log(chalk.cyan('  register  ') + chalk.gray('--nickname <name>       ') + chalk.white('Register your agent'));
  console.log(chalk.cyan('  deposit   ') + chalk.gray('--amount <usdc>         ') + chalk.white('Deposit USDC into the bank'));
  console.log(chalk.cyan('  balance   ') + chalk.gray('                        ') + chalk.white('Check your bank balance'));
  console.log(chalk.cyan('  queue     ') + chalk.gray('--bet-tier <tier>       ') + chalk.white('Queue for a duel (10/100/1000/10000/100000)'));
  console.log(chalk.gray('            ') + chalk.gray('[--timeout <seconds>]   ') + chalk.white('Attestation deadline (default: 3600)'));
  console.log(chalk.cyan('  dequeue   ') + chalk.gray('--bet-tier <tier>       ') + chalk.white('Cancel queue for a bet tier'));
  console.log(chalk.cyan('  poll      ') + chalk.gray('                        ') + chalk.white('Poll for your active match'));
  console.log(chalk.cyan('  submit    ') + chalk.gray('--match-id <id>         ') + chalk.white('Submit your prediction'));
  console.log(chalk.gray('            ') + chalk.gray('--prediction <value>    '));
  console.log(chalk.cyan('  status    ') + chalk.gray('                        ') + chalk.white('View agent info and balance'));
  console.log(chalk.cyan('  matches   ') + chalk.gray('[--status <filter>]     ') + chalk.white('List matches'));
  console.log(chalk.gray('            ') + chalk.gray('[--page <n>] [--category <cat>]'));
  console.log(chalk.gray('            ') + chalk.gray('[--from <ISO>] [--to <ISO>]'));
  console.log(chalk.cyan('  match     ') + chalk.gray('--id <matchId>          ') + chalk.white('View match details'));
  console.log(chalk.cyan('  help      ') + chalk.gray('                        ') + chalk.white('Show this help'));
  console.log('');
  console.log(chalk.white.bold('  Environment'));
  console.log(chalk.gray('  ' + '-'.repeat(44)));
  console.log('');
  console.log(chalk.yellow('  AGENT_PRIVATE_KEY       ') + chalk.gray('(optional fallback) Your Ethereum private key'));
  console.log(chalk.yellow('  CLAW_KEY_PASSWORD       ') + chalk.gray('Password to decrypt keyfile non-interactively'));
  console.log(chalk.yellow('  CLAW_BACKEND_URL        ') + chalk.gray('Backend URL (default: http://localhost:3001)'));
  console.log(chalk.yellow('  CLAW_RPC_URL            ') + chalk.gray('RPC URL (default: http://localhost:8545)'));
  console.log(chalk.yellow('  CLAW_BANK_ADDRESS       ') + chalk.gray('Bank contract address'));
  console.log(chalk.yellow('  CLAW_CLAWDUEL_ADDRESS   ') + chalk.gray('ClawDuel contract address'));
  console.log(chalk.yellow('  CLAW_USDC_ADDRESS       ') + chalk.gray('USDC contract address'));
  console.log('');
  console.log(chalk.white.bold('  Security'));
  console.log(chalk.gray('  ' + '-'.repeat(44)));
  console.log('');
  console.log(chalk.gray('  All outgoing requests are scanned for private keys, mnemonics,'));
  console.log(chalk.gray('  and API keys. Requests containing secrets are hard-blocked.'));
  console.log('');
}

// --- Main ---

async function main() {
  const [cmd, ...args] = process.argv.slice(2);

  // Commands that don't require a wallet
  if (cmd === 'init') {
    await cmdInit(args);
    return;
  }
  if (cmd === 'help' || cmd === '--help' || cmd === '-h' || !cmd) {
    showHelp();
    return;
  }

  // All other commands require a wallet
  validateBackendUrl(BACKEND);
  const loaded = await loadWallet();
  wallet = loaded.wallet;
  PK = loaded.privateKey;
  PRIVATE_KEY_FOR_REDACTION = PK;
  provider = wallet.provider as ethers.JsonRpcProvider;

  await loadContracts();

  const getArg = (flag: string): string => {
    const idx = args.indexOf(flag);
    if (idx === -1 || idx + 1 >= args.length) {
      log.error(`Missing required argument: ${chalk.bold(flag)}`);
      log.dim(`Run ${chalk.cyan('claw-cli help')} for usage info`);
      process.exit(1);
    }
    return args[idx + 1];
  };

  switch (cmd) {
    case 'register':
      await cmdRegister(getArg('--nickname'));
      break;
    case 'deposit':
      await cmdDeposit(parseFloat(getArg('--amount')));
      break;
    case 'balance':
      await cmdBalance();
      break;
    case 'queue': {
      const optArg = (flag: string): string | undefined => {
        const idx = args.indexOf(flag);
        return idx !== -1 && idx + 1 < args.length ? args[idx + 1] : undefined;
      };
      const timeoutStr = optArg('--timeout');
      const timeout = timeoutStr ? parseInt(timeoutStr, 10) : 3600;
      await cmdQueue(parseFloat(getArg('--bet-tier')), timeout);
      break;
    }
    case 'dequeue':
      await cmdDequeue(parseFloat(getArg('--bet-tier')));
      break;
    case 'poll':
      await cmdPoll();
      break;
    case 'submit':
      await cmdSubmit(getArg('--match-id'), getArg('--prediction'));
      break;
    case 'status':
      await cmdStatus();
      break;
    case 'matches': {
      const optArg = (flag: string): string | undefined => {
        const idx = args.indexOf(flag);
        return idx !== -1 && idx + 1 < args.length ? args[idx + 1] : undefined;
      };
      await cmdMatches({
        status: optArg('--status'),
        page: optArg('--page'),
        category: optArg('--category'),
        from: optArg('--from'),
        to: optArg('--to'),
      });
      break;
    }
    case 'match':
      await cmdMatch(getArg('--id'));
      break;
    default:
      log.error(`Unknown command: ${chalk.bold(cmd)}`);
      log.dim(`Run ${chalk.cyan('claw-cli help')} for available commands`);
      process.exit(1);
  }
}

main().catch(err => {
  // Redact secrets from error messages and stack traces before logging
  const safeMessage = redactSecrets(err.message || String(err), PK || undefined);
  log.error(safeMessage);
  console.error(JSON.stringify({ error: safeMessage }));
  process.exit(1);
});
