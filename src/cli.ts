#!/usr/bin/env node
import { ClawAgent } from './index';
import OpenAI from 'openai';
import dotenv from 'dotenv';

dotenv.config();

/**
 * No-code agent entry point.
 * Runs a default OpenAI-powered prediction agent.
 */
async function main() {
  const privateKey = process.env.AGENT_PRIVATE_KEY;
  const openaiApiKey = process.env.OPENAI_API_KEY;
  const port = parseInt(process.env.AGENT_PORT || '3333');
  const secret = process.env.CLAW_SECRET;
  const model = process.env.AGENT_MODEL || 'gpt-4o';

  if (!privateKey || !openaiApiKey) {
    console.error(`
  Missing required environment variables:
    AGENT_PRIVATE_KEY  — Your Ethereum private key
    OPENAI_API_KEY     — Your OpenAI API key

  Optional:
    CLAW_SECRET   — Webhook signing secret
    AGENT_PORT    — Port to listen on (default: 3333)
    AGENT_MODEL   — OpenAI model (default: gpt-4o)
    BACKEND_URL   — Claw Arena backend URL
    `);
    process.exit(1);
  }

  const openai = new OpenAI({ apiKey: openaiApiKey });
  const agent = new ClawAgent({ privateKey, port, secret });

  console.log(`[CLI] Starting agent with model ${model}...`);

  agent.start(async (problem, deadline) => {
    const timeLeft = deadline.getTime() - Date.now();

    const response = await openai.chat.completions.create({
      model,
      messages: [
        {
          role: 'system',
          content: `You are a competitive prediction agent in the Claw Arena. You must predict a real-world value. You have ${Math.floor(timeLeft / 1000)} seconds. Be precise. Return ONLY the predicted value, nothing else.`
        },
        {
          role: 'user',
          content: `Category: ${problem.category}\nType: ${problem.valueType}\n\n${problem.prompt}`
        }
      ],
      temperature: 0.3,
      max_tokens: 100,
    });

    const raw = response.choices[0]?.message?.content?.trim() ?? '';

    // Parse based on value type
    if (problem.valueType === 'number') {
      const match = raw.match(/-?\d+\.?\d*/);
      return match ? parseFloat(match[0]) : raw;
    }

    return raw;
  });
}

main().catch(err => {
  console.error('[CLI] Fatal:', err);
  process.exit(1);
});
