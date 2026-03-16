import express, { Request, Response } from 'express';
import crypto from 'crypto';
import { ethers } from 'ethers';
import dotenv from 'dotenv';

dotenv.config();

// --- Types ---

export interface MatchWebhook {
  matchId: string;
  problem: {
    category: string;
    title: string;
    prompt: string;
    valueType: 'number' | 'string' | 'boolean';
  };
  deadline: string;
  submissionUrl: string;
}

export type PredictHandler = (
  problem: MatchWebhook['problem'],
  deadline: Date
) => Promise<string | number | boolean>;

export interface AgentOptions {
  privateKey: string;
  rpcUrl?: string;
  backendUrl?: string;
  contractAddresses?: {
    bank: string;
    registry: string;
    duelArena: string;
    multiArena: string;
    usdc: string;
  };
  port?: number;
  secret?: string;
  verbose?: boolean;
}

// --- Protocol Client ---

/**
 * ClawClient
 *
 * Handles all on-chain interactions: registration, deposits, and
 * signing combat attestations.
 */
export class ClawClient {
  public provider: ethers.JsonRpcProvider;
  public wallet: ethers.Wallet;
  public addresses: any;

  constructor(options: AgentOptions) {
    const rpc = options.rpcUrl || process.env.RPC_URL || 'https://polygon-rpc.com';
    this.provider = new ethers.JsonRpcProvider(rpc);
    this.wallet = new ethers.Wallet(options.privateKey, this.provider);
    this.addresses = options.contractAddresses || {
      bank: process.env.BANK_ADDRESS,
      registry: process.env.AGENT_REGISTRY_ADDRESS,
      duelArena: process.env.DUEL_ARENA_ADDRESS,
      multiArena: process.env.MULTI_ARENA_ADDRESS,
      usdc: process.env.USDC_ADDRESS,
    };
  }

  async register(nickname: string) {
    const registry = new ethers.Contract(this.addresses.registry, [
      "function registerAgent(string nickname) external",
      "function isRegistered(address agent) external view returns (bool)"
    ], this.wallet);

    const isReg = await registry.isRegistered(this.wallet.address);
    if (isReg) throw new Error("AGENT_ALREADY_REGISTERED");

    return registry.registerAgent(nickname);
  }

  async deposit(amountUsdc: number) {
    const amount = ethers.parseUnits(amountUsdc.toString(), 6);
    const usdc = new ethers.Contract(this.addresses.usdc, [
      "function approve(address spender, uint256 amount) external returns (bool)",
      "function allowance(address owner, address spender) external view returns (uint256)"
    ], this.wallet);

    const bank = new ethers.Contract(this.addresses.bank, [
      "function deposit(uint256 amount) external"
    ], this.wallet);

    const allowance = await usdc.allowance(this.wallet.address, this.addresses.bank);
    if (allowance < amount) {
      console.log("[SDK] Approving USDC for Bank...");
      const tx = await usdc.approve(this.addresses.bank, amount);
      await tx.wait();
    }

    return bank.deposit(amount);
  }

  async getBalances() {
    const bank = new ethers.Contract(this.addresses.bank, [
      "function balanceOf(address user) external view returns (uint256)",
      "function lockedBalanceOf(address user) external view returns (uint256)"
    ], this.provider);

    const liquid = await bank.balanceOf(this.wallet.address);
    const locked = await bank.lockedBalanceOf(this.wallet.address);

    return {
      liquid: ethers.formatUnits(liquid, 6),
      locked: ethers.formatUnits(locked, 6)
    };
  }

  async signDuelAttestation(betTierIndex: number, deadlineSeconds: number = 3600) {
    const duelArena = new ethers.Contract(this.addresses.duelArena, [
      "function nonces(address) external view returns (uint256)"
    ], this.provider);

    const nonce = await duelArena.nonces(this.wallet.address);
    const deadline = Math.floor(Date.now() / 1000) + deadlineSeconds;

    const domain = {
      name: "ClawArena",
      version: "1",
      chainId: (await this.provider.getNetwork()).chainId,
      verifyingContract: this.addresses.duelArena
    };

    const types = {
      JoinDuelAttestation: [
        { name: "agent", type: "address" },
        { name: "betTier", type: "uint256" },
        { name: "nonce", type: "uint256" },
        { name: "deadline", type: "uint256" }
      ]
    };

    const value = {
      agent: this.wallet.address,
      betTier: betTierIndex,
      nonce: nonce,
      deadline: deadline
    };

    const signature = await this.wallet.signTypedData(domain, types, value);

    return { attestation: value, signature };
  }

  async signArenaAttestation(arenaId: number, deadlineSeconds: number = 3600) {
    const multiArena = new ethers.Contract(this.addresses.multiArena, [
      "function nonces(address) external view returns (uint256)"
    ], this.provider);

    const nonce = await multiArena.nonces(this.wallet.address);
    const deadline = Math.floor(Date.now() / 1000) + deadlineSeconds;

    const domain = {
      name: "ClawArena",
      version: "1",
      chainId: (await this.provider.getNetwork()).chainId,
      verifyingContract: this.addresses.multiArena
    };

    const types = {
      JoinMultiAttestation: [
        { name: "agent", type: "address" },
        { name: "arenaId", type: "uint256" },
        { name: "nonce", type: "uint256" },
        { name: "deadline", type: "uint256" }
      ]
    };

    const value = {
      agent: this.wallet.address,
      arenaId,
      nonce,
      deadline
    };

    const signature = await this.wallet.signTypedData(domain, types, value);

    return { attestation: value, signature };
  }
}

// --- Agent Server ---

/**
 * ClawAgent
 *
 * Receives match webhooks from the backend, calls your predict handler,
 * and submits the prediction to the backend before the deadline.
 *
 * Agent developers only need to implement: (problem, deadline) => prediction
 */
export class ClawAgent {
  public client: ClawClient;
  private app = express();
  private port: number;
  private secret: string | null;
  private verbose: boolean;
  private backendUrl: string;

  constructor(options: AgentOptions) {
    this.client = new ClawClient(options);
    this.port = options.port || 3333;
    this.secret = options.secret || process.env.CLAW_SECRET || null;
    this.verbose = options.verbose ?? true;
    this.backendUrl = options.backendUrl || process.env.BACKEND_URL || 'http://localhost:3000';

    this.app.use(express.json());
  }

  /**
   * Verify that a webhook came from the Claw Arena backend.
   */
  private verifyWebhookSignature(body: string, signature: string): boolean {
    if (!this.secret) return true; // No secret configured, skip
    const expected = crypto.createHmac('sha256', this.secret).update(body).digest('hex');
    return crypto.timingSafeEqual(Buffer.from(signature), Buffer.from(expected));
  }

  /**
   * Submit a prediction to the backend.
   */
  private async submitPrediction(
    submissionUrl: string,
    prediction: string | number | boolean
  ): Promise<boolean> {
    const timestamp = String(Date.now());
    const message = `ClawArena:auth:${this.client.wallet.address.toLowerCase()}:${timestamp}`;
    const signature = await this.client.wallet.signMessage(message);

    try {
      const res = await fetch(submissionUrl, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'X-Agent-Address': this.client.wallet.address,
          'X-Agent-Signature': signature,
          'X-Agent-Timestamp': timestamp,
        },
        body: JSON.stringify({ prediction: String(prediction) }),
      });

      if (!res.ok) {
        const err = await res.json().catch(() => ({ error: res.statusText }));
        console.error(`[SDK] Submission failed: ${res.status} — ${(err as any).error}`);
        return false;
      }

      if (this.verbose) console.log(`[SDK] Prediction submitted: ${prediction}`);
      return true;
    } catch (err) {
      console.error('[SDK] Submission error:', err instanceof Error ? err.message : err);
      return false;
    }
  }

  /**
   * Start the agent. Provide a handler that receives a problem and returns a prediction.
   *
   * Example:
   * ```ts
   * agent.start(async (problem, deadline) => {
   *   // Your prediction logic here
   *   return 67432.50;
   * });
   * ```
   */
  public start(onPredict: PredictHandler) {
    // Webhook endpoint — receives match notifications from backend
    this.app.post('/', async (req: Request, res: Response) => {
      // Verify webhook signature
      if (this.secret) {
        const sig = req.headers['x-claw-signature'] as string;
        if (!sig) {
          res.status(401).json({ error: 'Missing signature' });
          return;
        }
        const rawBody = JSON.stringify(req.body);
        if (!this.verifyWebhookSignature(rawBody, sig)) {
          res.status(401).json({ error: 'Invalid signature' });
          return;
        }
      }

      const webhook = req.body as MatchWebhook;
      if (!webhook.matchId || !webhook.problem || !webhook.deadline || !webhook.submissionUrl) {
        res.status(400).json({ error: 'Invalid webhook payload' });
        return;
      }

      const deadline = new Date(webhook.deadline);

      if (this.verbose) {
        console.log(`[SDK] Match ${webhook.matchId}: ${webhook.problem.title} (${webhook.problem.category})`);
        console.log(`[SDK] Deadline: ${webhook.deadline}`);
      }

      // Acknowledge webhook immediately — prediction runs async
      res.json({ ok: true });

      // Run prediction in background
      try {
        const prediction = await onPredict(webhook.problem, deadline);

        // Check deadline before submitting
        if (Date.now() > deadline.getTime()) {
          console.error(`[SDK] Match ${webhook.matchId}: prediction took too long, deadline passed`);
          return;
        }

        await this.submitPrediction(webhook.submissionUrl, prediction);
      } catch (err) {
        console.error(`[SDK] Match ${webhook.matchId}: prediction failed —`, err instanceof Error ? err.message : err);
      }
    });

    // Health check
    this.app.get('/status', (_req, res) => {
      res.json({
        status: 'OPERATIONAL',
        agent: this.client.wallet.address,
        version: '2.0.0'
      });
    });

    this.app.listen(this.port, () => {
      if (this.verbose) {
        console.log(`
  CLAW AGENT ONLINE
  -----------------
  Agent:    ${this.client.wallet.address}
  Port:     ${this.port}
  Backend:  ${this.backendUrl}
  -----------------
        `);
      }
    });
  }
}
