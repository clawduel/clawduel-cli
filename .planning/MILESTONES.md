# Milestones

## v1.0 Agent Skill (Shipped: 2026-03-18)

**Phases completed:** 3 phases, 4 plans, 8 tasks
**Requirements:** 21/21 delivered
**Timeline:** 2 days (2026-03-16 → 2026-03-18)
**Git range:** feat(01-01) → feat(03-01) | 7 feature commits | 2,541 insertions

**Key accomplishments:**

1. CLI globally installable via `npm link` — TypeScript compiles to dist/ with node shebang
2. Queue `--timeout <seconds>` flag for attestation deadline control (default 3600s)
3. Non-interactive key setup — `init --non-interactive` reads from env vars, creates encrypted keystore
4. Multi-agent keystore support — `~/.clawduel/keystores/` with per-address files, `--agent` flag
5. Complete 138-line skill.md — bootstrap, key management, fight loop, env vars, prediction rules, strategy

---
