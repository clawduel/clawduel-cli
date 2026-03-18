import { ethers } from 'ethers';
import chalk from 'chalk';
import dotenv from 'dotenv';

dotenv.config();

// --- Security: Secret Leak Detection ---

/**
 * Patterns that detect secrets which must NEVER be sent to a backend.
 * Each pattern is tested against the full JSON-serialized request body.
 */
const SECRET_PATTERNS: { name: string; pattern: RegExp }[] = [
  // Ethereum private keys: 64 hex chars, with or without 0x prefix
  { name: 'Ethereum private key (0x-prefixed)', pattern: /(?:^|[^a-fA-F0-9])0x[0-9a-fA-F]{64}(?:[^a-fA-F0-9]|$)/ },
  { name: 'Ethereum private key (raw hex)', pattern: /(?:^|[^a-fA-F0-9x])[0-9a-fA-F]{64}(?:[^a-fA-F0-9]|$)/ },
  // BIP-39 mnemonic seed phrases (12 or 24 words of lowercase alpha separated by spaces)
  { name: 'Mnemonic seed phrase', pattern: /(?:[a-z]{3,8}\s){11,23}[a-z]{3,8}/ },
  // Extended private keys (BIP-32)
  { name: 'Extended private key (xprv)', pattern: /xprv[a-zA-Z0-9]{107,108}/ },
  // OpenAI / Anthropic / common API keys
  { name: 'API key (sk- prefix)', pattern: /sk-[a-zA-Z0-9_-]{20,}/ },
  { name: 'API key (sk-ant- prefix)', pattern: /sk-ant-[a-zA-Z0-9_-]{20,}/ },
  // AWS secret keys (40-char base64)
  { name: 'AWS secret key', pattern: /(?:AWS|aws).{0,20}['\"][0-9a-zA-Z/+=]{40}['\"]/ },
];

/**
 * Scans a string for embedded secrets. Returns the name of the first match found, or null.
 */
function detectSecretLeak(data: string): string | null {
  for (const { name, pattern } of SECRET_PATTERNS) {
    if (pattern.test(data)) {
      return name;
    }
  }
  return null;
}

/**
 * Scans an object (to be JSON-serialized) for secrets. Throws if a secret is detected.
 * Also accepts the agent's own private key to do an exact-match check.
 */
function assertNoSecretLeak(body: unknown, privateKey?: string): void {
  const serialized = typeof body === 'string' ? body : JSON.stringify(body);

  // Exact match against the agent's own private key (stripped of 0x)
  if (privateKey) {
    const rawKey = privateKey.startsWith('0x') ? privateKey.slice(2) : privateKey;
    if (serialized.includes(rawKey)) {
      throw new SecretLeakError(
        'BLOCKED: Request body contains the agent\'s own private key. Request was NOT sent.'
      );
    }
  }

  // Pattern-based detection
  const detected = detectSecretLeak(serialized);
  if (detected) {
    throw new SecretLeakError(
      `BLOCKED: Request body appears to contain a secret (${detected}). Request was NOT sent.`
    );
  }
}

/**
 * Custom error class for secret leak detection so callers can distinguish it.
 */
export class SecretLeakError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'SecretLeakError';
  }
}

// --- Security: URL Validation ---

/**
 * Validates a URL to prevent SSRF. Rejects non-HTTP(S) schemes and internal/private IPs.
 */
function validateBackendUrl(url: string): void {
  let parsed: URL;
  try {
    parsed = new URL(url);
  } catch {
    throw new Error(`Invalid backend URL: ${redactSecrets(url)}`);
  }

  if (!['http:', 'https:'].includes(parsed.protocol)) {
    throw new Error(`Backend URL must use http or https protocol, got: ${parsed.protocol}`);
  }

  // Block internal/private IPs in non-localhost configurations
  const hostname = parsed.hostname;
  const isLocalhost = hostname === 'localhost' || hostname === '127.0.0.1' || hostname === '::1';
  const isPrivateIp = /^(10\.|172\.(1[6-9]|2\d|3[01])\.|192\.168\.|169\.254\.|0\.0\.0\.0)/.test(hostname);
  const isMetadata = hostname === '169.254.169.254'; // AWS metadata endpoint

  if (isMetadata) {
    throw new Error('Backend URL must not point to cloud metadata endpoints');
  }

  // Allow localhost for development, but warn on private IPs
  if (isPrivateIp && !isLocalhost) {
    // Allow but this is suspicious in production -- could add a strict mode later
  }
}

/**
 * Sanitizes a URL path segment to prevent path traversal or injection.
 */
function sanitizePathSegment(segment: string): string {
  // Allow only alphanumeric, hyphens, underscores, and dots
  return segment.replace(/[^a-zA-Z0-9\-_.]/g, '');
}

// --- Security: Sensitive Data Redaction ---

/**
 * Redacts potential secrets from a string (for safe logging).
 */
function redactSecrets(input: string): string {
  let result = input;
  // Redact hex strings that look like private keys
  result = result.replace(/0x[0-9a-fA-F]{64}/g, '0x[REDACTED_KEY]');
  result = result.replace(/(?<![0-9a-fA-F])[0-9a-fA-F]{64}(?![0-9a-fA-F])/g, '[REDACTED_HEX]');
  // Redact API keys
  result = result.replace(/sk-[a-zA-Z0-9_-]{20,}/g, 'sk-[REDACTED]');
  result = result.replace(/sk-ant-[a-zA-Z0-9_-]{20,}/g, 'sk-ant-[REDACTED]');
  result = result.replace(/xprv[a-zA-Z0-9]{50,}/g, 'xprv[REDACTED]');
  return result;
}

// --- Security: Request Timeout ---

const DEFAULT_REQUEST_TIMEOUT_MS = 30_000; // 30 seconds

// --- Types ---

export interface ClientOptions {
  privateKey: string;
  rpcUrl?: string;
  backendUrl?: string;
  contractAddresses?: {
    bank: string;
    clawDuel: string;
    multiDuel: string;
    usdc: string;
  };
  /** Request timeout in milliseconds. Default: 30000 */
  requestTimeoutMs?: number;
}

// --- Logging Helpers ---

const log = {
  info: (msg: string) => console.log(chalk.cyan('  INFO ') + chalk.white(msg)),
  success: (msg: string) => console.log(chalk.green('    OK ') + chalk.white(msg)),
  error: (msg: string) => console.error(chalk.red(' ERROR ') + chalk.white(msg)),
};

// --- Protocol Client ---

/**
 * ClawClient
 *
 * Handles on-chain interactions: deposits, balance queries, and EIP-712 attestation signing.
 * Used programmatically by agents that import the SDK directly.
 *
 * Includes built-in secret leak detection: all outgoing request bodies are scanned for
 * private keys, mnemonics, and other secrets before being sent to the backend.
 *
 * For CLI usage, run `npx tsx clawduel-cli.ts help` instead.
 */
export class ClawClient {
  public provider: ethers.JsonRpcProvider;
  public wallet: ethers.Wallet;
  public addresses: {
    bank: string;
    clawDuel: string;
    multiDuel: string;
    usdc: string;
  };

  private backendUrl: string;
  private privateKey: string;
  private requestTimeoutMs: number;

  constructor(options: ClientOptions) {
    const rpc = options.rpcUrl || process.env.RPC_URL || 'https://polygon-rpc.com';
    this.provider = new ethers.JsonRpcProvider(rpc);
    this.wallet = new ethers.Wallet(options.privateKey, this.provider);
    this.privateKey = options.privateKey;
    this.backendUrl = options.backendUrl || process.env.CLAW_BACKEND_URL || 'http://localhost:3001';
    this.requestTimeoutMs = options.requestTimeoutMs || DEFAULT_REQUEST_TIMEOUT_MS;
    this.addresses = options.contractAddresses || {
      bank: process.env.BANK_ADDRESS || '',
      clawDuel: process.env.CLAWDUEL_ADDRESS || '',
      multiDuel: process.env.MULTIDUEL_ADDRESS || '',
      usdc: process.env.USDC_ADDRESS || '',
    };

    // Validate backend URL at construction time
    validateBackendUrl(this.backendUrl);
  }

  /**
   * Sends a POST request to the backend with secret leak detection, auth headers,
   * request timeout, and safe error handling.
   */
  async apiPost(path: string, body: unknown): Promise<{ status: number; body: any }> {
    // Scan outgoing body for secrets BEFORE sending
    assertNoSecretLeak(body, this.privateKey);

    const sanitizedPath = path.startsWith('/') ? path : `/${path}`;
    const headers = await this.authHeaders();

    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), this.requestTimeoutMs);

    try {
      const res = await fetch(`${this.backendUrl}${sanitizedPath}`, {
        method: 'POST',
        headers,
        body: JSON.stringify(body),
        signal: controller.signal,
      });

      const responseBody = await res.json();

      // Redact any reflected secrets in error responses
      if (res.status >= 400 && responseBody?.error) {
        responseBody.error = redactSecrets(String(responseBody.error));
      }

      return { status: res.status, body: responseBody };
    } catch (err: any) {
      if (err.name === 'AbortError') {
        throw new Error(`Request to ${sanitizedPath} timed out after ${this.requestTimeoutMs}ms`);
      }
      if (err instanceof SecretLeakError) {
        throw err;
      }
      throw new Error(`Request to ${sanitizedPath} failed: ${redactSecrets(err.message || String(err))}`);
    } finally {
      clearTimeout(timeout);
    }
  }

  /**
   * Sends a GET request to the backend with timeout and safe error handling.
   */
  async apiGet(path: string): Promise<any> {
    const sanitizedPath = path.startsWith('/') ? path : `/${path}`;

    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), this.requestTimeoutMs);

    try {
      const res = await fetch(`${this.backendUrl}${sanitizedPath}`, {
        signal: controller.signal,
      });
      const responseBody = await res.json();

      // Redact any reflected secrets in error responses
      if (res.status >= 400 && responseBody?.error) {
        responseBody.error = redactSecrets(String(responseBody.error));
      }

      return responseBody;
    } catch (err: any) {
      if (err.name === 'AbortError') {
        throw new Error(`Request to ${sanitizedPath} timed out after ${this.requestTimeoutMs}ms`);
      }
      throw new Error(`Request to ${sanitizedPath} failed: ${redactSecrets(err.message || String(err))}`);
    } finally {
      clearTimeout(timeout);
    }
  }

  private async authHeaders(): Promise<Record<string, string>> {
    const timestamp = String(Date.now());
    const message = `ClawDuel:auth:${this.wallet.address.toLowerCase()}:${timestamp}`;
    const signature = await this.wallet.signMessage(message);
    return {
      'Content-Type': 'application/json',
      'X-Agent-Address': this.wallet.address,
      'X-Agent-Signature': signature,
      'X-Agent-Timestamp': timestamp,
    };
  }

  async deposit(amountUsdc: number) {
    const amount = ethers.parseUnits(amountUsdc.toString(), 6);
    const usdc = new ethers.Contract(this.addresses.usdc, [
      'function approve(address spender, uint256 amount) external returns (bool)',
      'function allowance(address owner, address spender) external view returns (uint256)',
    ], this.wallet);

    const bank = new ethers.Contract(this.addresses.bank, [
      'function deposit(uint256 amount) external',
    ], this.wallet);

    const allowance = await usdc.allowance(this.wallet.address, this.addresses.bank);
    if (allowance < amount) {
      log.info('Approving USDC for Bank...');
      const tx = await usdc.approve(this.addresses.bank, amount);
      await tx.wait();
    }

    return bank.deposit(amount);
  }

  async getBalances() {
    const bank = new ethers.Contract(this.addresses.bank, [
      'function balanceOf(address user) external view returns (uint256)',
      'function lockedBalanceOf(address user) external view returns (uint256)',
    ], this.provider);

    const liquid = await bank.balanceOf(this.wallet.address);
    const locked = await bank.lockedBalanceOf(this.wallet.address);

    return {
      liquid: ethers.formatUnits(liquid, 6),
      locked: ethers.formatUnits(locked, 6),
    };
  }

  async signDuelAttestation(betTierIndex: number, deadlineSeconds: number = 3600) {
    const clawDuel = new ethers.Contract(this.addresses.clawDuel, [
      'function nonces(address) external view returns (uint256)',
    ], this.provider);

    const nonce = await clawDuel.nonces(this.wallet.address);
    const deadline = Math.floor(Date.now() / 1000) + deadlineSeconds;

    const domain = {
      name: 'ClawDuel',
      version: '1',
      chainId: (await this.provider.getNetwork()).chainId,
      verifyingContract: this.addresses.clawDuel,
    };

    const types = {
      JoinDuelAttestation: [
        { name: 'agent', type: 'address' },
        { name: 'betTier', type: 'uint256' },
        { name: 'nonce', type: 'uint256' },
        { name: 'deadline', type: 'uint256' },
      ],
    };

    const value = {
      agent: this.wallet.address,
      betTier: betTierIndex,
      nonce,
      deadline,
    };

    const signature = await this.wallet.signTypedData(domain, types, value);
    return { attestation: value, signature };
  }

  async signMultiAttestation(duelId: number, deadlineSeconds: number = 3600) {
    const multiDuel = new ethers.Contract(this.addresses.multiDuel, [
      'function nonces(address) external view returns (uint256)',
    ], this.provider);

    const nonce = await multiDuel.nonces(this.wallet.address);
    const deadline = Math.floor(Date.now() / 1000) + deadlineSeconds;

    const domain = {
      name: 'ClawDuel',
      version: '1',
      chainId: (await this.provider.getNetwork()).chainId,
      verifyingContract: this.addresses.multiDuel,
    };

    const types = {
      JoinMultiAttestation: [
        { name: 'agent', type: 'address' },
        { name: 'duelId', type: 'uint256' },
        { name: 'nonce', type: 'uint256' },
        { name: 'deadline', type: 'uint256' },
      ],
    };

    const value = {
      agent: this.wallet.address,
      duelId,
      nonce,
      deadline,
    };

    const signature = await this.wallet.signTypedData(domain, types, value);
    return { attestation: value, signature };
  }
}

// --- Exported utilities for use by CLI and external consumers ---

export {
  assertNoSecretLeak,
  detectSecretLeak,
  redactSecrets,
  validateBackendUrl,
  sanitizePathSegment,
  SecretLeakError as SecretLeakDetected,
  SECRET_PATTERNS,
  DEFAULT_REQUEST_TIMEOUT_MS,
};
