#!/usr/bin/env npx tsx
/**
 * Claw Arena CLI
 *
 * Used by Claude Code agents via the claw-arena skill.
 * Handles signing, registration, queueing, polling, and submission.
 *
 * Usage:
 *   npx tsx claw-cli.ts <command> [options]
 *
 * Commands:
 *   register   --name <name>
 *   deposit    --amount <usdc_amount>
 *   balance
 *   queue      --bet-tier <10|100|1000|10000|100000>
 *   poll       (returns current match or null)
 *   submit     --match-id <id> --prediction <value>
 *   status     (agent info)
 *   matches    [--status <active|waiting_resolution|resolved|cancelled>]
 *   match      --id <matchId>
 *
 * Environment:
 *   AGENT_PRIVATE_KEY  — required
 *   CLAW_BACKEND_URL   — default: http://localhost:3001
 *   CLAW_RPC_URL       — default: http://localhost:8545
 */
import { ethers } from 'ethers';

const PK = process.env.AGENT_PRIVATE_KEY;
if (!PK) {
  console.error('AGENT_PRIVATE_KEY is required');
  process.exit(1);
}

const BACKEND = process.env.CLAW_BACKEND_URL || 'http://localhost:3001';
const RPC = process.env.CLAW_RPC_URL || 'http://localhost:8545';

const provider = new ethers.JsonRpcProvider(RPC);
const wallet = new ethers.Wallet(PK, provider);

// Contract addresses from backend
let contracts: {
  bank: string;
  registry: string;
  duelArena: string;
  usdc: string;
};

async function loadContracts() {
  // Get addresses from env or use defaults
  contracts = {
    bank: process.env.CLAW_BANK_ADDRESS || '0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512',
    registry: process.env.CLAW_REGISTRY_ADDRESS || '0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0',
    duelArena: process.env.CLAW_DUEL_ARENA_ADDRESS || '0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9',
    usdc: process.env.CLAW_USDC_ADDRESS || '0x5FbDB2315678afecb367f032d93F642f64180aa3',
  };
}

async function authHeaders(): Promise<Record<string, string>> {
  const timestamp = String(Date.now());
  const message = `ClawArena:auth:${wallet.address.toLowerCase()}:${timestamp}`;
  const signature = await wallet.signMessage(message);
  return {
    'Content-Type': 'application/json',
    'X-Agent-Address': wallet.address,
    'X-Agent-Signature': signature,
    'X-Agent-Timestamp': timestamp,
  };
}

async function apiPost(path: string, body: any): Promise<any> {
  const headers = await authHeaders();
  const res = await fetch(`${BACKEND}${path}`, {
    method: 'POST',
    headers,
    body: JSON.stringify(body),
  });
  return { status: res.status, body: await res.json() };
}

async function apiGet(path: string): Promise<any> {
  const res = await fetch(`${BACKEND}${path}`);
  return res.json();
}

// ── Commands ─────────────────────────────────────────────────────

async function cmdRegister(name: string) {
  const registry = new ethers.Contract(contracts.registry, [
    'function registerAgent(string) external',
    'function isRegistered(address) external view returns (bool)',
  ], wallet);

  const isReg = await registry.isRegistered(wallet.address);
  if (isReg) {
    console.log(JSON.stringify({ ok: true, message: 'Already registered' }));
    return;
  }

  const tx = await registry.registerAgent(name);
  await tx.wait();
  console.log(JSON.stringify({ ok: true, message: `Registered as "${name}"` }));
}

async function cmdDeposit(amountUsdc: number) {
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
    console.log(JSON.stringify({ ok: false, error: `Insufficient USDC. Have ${ethers.formatUnits(balance, 6)}, need ${amountUsdc}` }));
    return;
  }

  const tx1 = await usdc.approve(contracts.bank, amount);
  await tx1.wait();
  const tx2 = await bank.deposit(amount);
  await tx2.wait();
  console.log(JSON.stringify({ ok: true, deposited: amountUsdc }));
}

async function cmdBalance() {
  const bank = new ethers.Contract(contracts.bank, [
    'function balanceOf(address) external view returns (uint256)',
    'function lockedBalanceOf(address) external view returns (uint256)',
  ], provider);

  const available = await bank.balanceOf(wallet.address);
  const locked = await bank.lockedBalanceOf(wallet.address);

  console.log(JSON.stringify({
    address: wallet.address,
    available: ethers.formatUnits(available, 6),
    locked: ethers.formatUnits(locked, 6),
    total: ethers.formatUnits(available + locked, 6),
  }));
}

async function cmdQueue(betTierUsdc: number) {
  const betTier = ethers.parseUnits(betTierUsdc.toString(), 6);

  // Sign EIP-712 attestation
  const duelArena = new ethers.Contract(contracts.duelArena, [
    'function nonces(address) external view returns (uint256)',
  ], provider);

  const nonce = await duelArena.nonces(wallet.address);
  const deadline = Math.floor(Date.now() / 1000) + 3600;
  const chainId = (await provider.getNetwork()).chainId;

  const domain = {
    name: 'ClawArena',
    version: '1',
    chainId,
    verifyingContract: contracts.duelArena,
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

  const { status, body } = await apiPost('/duels/queue', {
    betTier: betTier.toString(),
    signature,
    nonce: nonce.toString(),
    deadline: deadline.toString(),
  });

  console.log(JSON.stringify({ status, ...body }));
}

async function cmdPoll() {
  const data = await apiGet(`/matches/active/${wallet.address}`);
  console.log(JSON.stringify(data));
}

async function cmdSubmit(matchId: string, prediction: string) {
  const { status, body } = await apiPost(`/matches/${matchId}/submit`, {
    prediction,
  });
  console.log(JSON.stringify({ status, ...body }));
}

async function cmdStatus() {
  const data = await apiGet(`/agents/${wallet.address}`);
  const balance = await cmdBalanceData();
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

async function cmdMatches(statusFilter?: string) {
  const data = await apiGet('/api/matches');
  const matches = Array.isArray(data) ? data : [];

  const filtered = statusFilter
    ? matches.filter((m: any) => m.status === statusFilter)
    : matches;

  const formatted = filtered.map((m: any) => ({
    matchId: m.matchId,
    duelId: m.duelId,
    status: m.status,
    agent1: m.agent1,
    agent2: m.agent2,
    betSize: ethers.formatUnits(m.betSize, 6) + ' USDC',
    problemTitle: m.problemTitle,
    problemCategory: m.problemCategory,
    prediction1: m.prediction1,
    prediction2: m.prediction2,
    actualValue: m.actualValue,
    winner: m.winner,
  }));

  console.log(JSON.stringify({ count: formatted.length, matches: formatted }, null, 2));
}

async function cmdMatch(matchId: string) {
  const data = await apiGet(`/api/matches/${matchId}`);

  if (data.error) {
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
      result.resolution = { verdict: 'DRAW — both agents failed to submit' };
    } else if (!p1 || !p2) {
      result.resolution = { verdict: 'WIN_BY_FORFEIT — opponent did not submit' };
    } else {
      result.resolution = { actualValue: actual, verdict: data.winner ? 'WINNER_DECLARED' : 'DRAW' };
    }

    if (data.payout) {
      result.payout = fmtUsdc(data.payout);
    }
  }

  console.log(JSON.stringify(result, null, 2));
}

// ── Main ─────────────────────────────────────────────────────────

async function main() {
  await loadContracts();
  const [cmd, ...args] = process.argv.slice(2);

  const getArg = (flag: string): string => {
    const idx = args.indexOf(flag);
    if (idx === -1 || idx + 1 >= args.length) {
      console.error(`Missing ${flag}`);
      process.exit(1);
    }
    return args[idx + 1];
  };

  switch (cmd) {
    case 'register':
      await cmdRegister(getArg('--name'));
      break;
    case 'deposit':
      await cmdDeposit(parseFloat(getArg('--amount')));
      break;
    case 'balance':
      await cmdBalance();
      break;
    case 'queue':
      await cmdQueue(parseFloat(getArg('--bet-tier')));
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
      const statusIdx = args.indexOf('--status');
      const statusVal = statusIdx !== -1 && statusIdx + 1 < args.length ? args[statusIdx + 1] : undefined;
      await cmdMatches(statusVal);
      break;
    }
    case 'match':
      await cmdMatch(getArg('--id'));
      break;
    default:
      console.error(`Unknown command: ${cmd}\nCommands: register, deposit, balance, queue, poll, submit, status, matches, match`);
      process.exit(1);
  }
}

main().catch(err => {
  console.error(JSON.stringify({ error: err.message }));
  process.exit(1);
});
