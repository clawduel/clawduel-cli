# Project Retrospective

## Milestone: v1.0 — Agent Skill

**Shipped:** 2026-03-18
**Phases:** 3 | **Plans:** 4

### What Was Built
- Global `clawduel-cli` binary via npm link with TypeScript compilation
- Non-interactive key management (`init --non-interactive`, `CLAW_KEY_PASSWORD`)
- Multi-agent keystore directory with `--agent` flag
- Queue `--timeout` flag for attestation deadline control
- 138-line skill.md with complete agent instructions

### What Worked
- Coarse granularity (3 phases) kept overhead low for a focused project
- Research before planning caught the tsconfig rootDir issue early (Phase 1)
- Sequential wave execution for same-file modifications prevented merge conflicts (Phase 2)
- Single-plan phases for straightforward work avoided unnecessary splitting

### What Was Inefficient
- Phase 2 research took longer than needed for what turned out to be surgical changes
- Validation strategies (VALIDATION.md) were somewhat redundant for a project with no test framework

### Patterns Established
- `optArg` pattern for scoped optional flag parsing in switch cases
- Global flag parsing with `args.splice()` before command dispatch
- Keystore naming with lowercase address including 0x prefix
- agentskills.io-compliant frontmatter with metadata block

### Key Lessons
- For CLI-only projects, build verification via `npm run build` + grep is sufficient without a test framework
- Documentation-only phases (Phase 3) execute fastest when research extracts all source material first
- Brownfield codebase mapping is valuable — caught the existing stale skill.md early

### Cost Observations
- Model mix: opus for research/planning/execution, sonnet for verification/checking
- Sessions: 1 continuous session
- Notable: 4 plans executed with minimal deviation (only 1 auto-fix across entire milestone)

---

## Cross-Milestone Trends

| Metric | v1.0 |
|--------|------|
| Phases | 3 |
| Plans | 4 |
| Feature commits | 7 |
| Deviations | 1 |
| Timeline | 2 days |
