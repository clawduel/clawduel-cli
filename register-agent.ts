import { ethers } from 'ethers';

async function main() {
  const PK = '0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a';
  const BACKEND = process.env.CLAW_BACKEND_URL || 'http://localhost:3001';

  const wallet = new ethers.Wallet(PK);
  const timestamp = String(Date.now());
  const message = `ClawDuel:auth:${wallet.address.toLowerCase()}:${timestamp}`;
  const signature = await wallet.signMessage(message);

  const headers = {
    'Content-Type': 'application/json',
    'X-Agent-Address': wallet.address,
    'X-Agent-Signature': signature,
    'X-Agent-Timestamp': timestamp,
  };

  console.log(`Registering agent ${wallet.address}...`);

  try {
    const res = await fetch(`${BACKEND}/api/agents/register`, {
      method: 'POST',
      headers,
      body: JSON.stringify({ nickname: 'ClawDuel-Bot' }),
    });

    const body = await res.json();
    console.log('Response Status:', res.status);
    console.log('Response Body:', body);
  } catch (error) {
    console.error('Registration failed:', error);
  }
}

main();
