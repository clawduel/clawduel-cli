# Phase 9: Multi-Duel Lobby Commands - Context

**Gathered:** 2026-03-20
**Status:** Ready for planning

<domain>
## Phase Boundary

Agent can create, join, list, and inspect multi-duel lobbies via CLI with EIP-712 multi-duel attestation signing. All lobby commands support `--output json` for machine-parseable output.

</domain>

<decisions>
## Implementation Decisions

### Lobby subcommand structure
- Nested subcommands under `lobby`: `clawduel lobby create`, `lobby join`, `lobby list`, `lobby status`
- Follows existing `wallet` subcommand pattern (clap `Subcommand` derive on `LobbyArgs`)
- Lobby ID is a positional argument for `join` and `status` (not `--id` flag)
- All lobby subcommands in a single `commands/lobby.rs` file (like `wallet.rs`)

### EIP-712 multi-duel signing
- Same EIP-712 domain as regular duels: name "ClawDuel", version "1"
- Different verifying contract: MultiDuel contract (separate from ClawDuel contract)
- JoinMultiAttestation struct has same fields as JoinDuelAttestation: agent, betTier, nonce, deadline
- New env var `CLAW_MULTIDUEL_ADDRESS` with placeholder default (contract not yet deployed)
- MultiDuel address added to `ContractAddresses` struct in `contracts.rs`
- Nonce generation reuses existing `generate_nonce` pattern (check `usedNonces` on-chain)

### Lobby creation parameters
- `--bet-size` is required (positional, like queue's `bet_tier`)
- `--max-participants` is optional with a sensible default (e.g., 5)
- `lobby create` auto-joins the creator — returns lobby ID and creator is first participant
- Since create auto-joins, it requires EIP-712 attestation signing (same as join)
- Create sends both lobby params and JoinMultiAttestation signature in one request

### Backend API endpoints
- Standard REST pattern: POST /lobbies, POST /lobbies/:id/join, GET /lobbies, GET /lobbies/:id
- Join payload mirrors queue: `{ betTier, signature, nonce, deadline }`
- Create payload: lobby params + attestation fields (betTier, maxParticipants, signature, nonce, deadline)
- `lobby list` returns all open lobbies — no filters in v1
- `lobby status` response: lobbyId, betSize, maxParticipants, currentParticipants (addresses/nicknames), status (open/full/started), createdAt

### Claude's Discretion
- Table output formatting for lobby list and status (use tabled like other commands)
- Error message wording for lobby-specific failures
- Whether to add `IMultiDuel` interface in contracts.rs or just use the address for signing

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

No external specs — requirements fully captured in decisions above.

### Existing code patterns to follow
- `src/commands/queue.rs` — EIP-712 signing flow (domain, attestation struct, sign_hash, POST body)
- `src/commands/wallet.rs` — Nested subcommand pattern (WalletArgs with Subcommand derive)
- `src/contracts.rs` — Contract addresses, `resolve_addresses()`, `sol!` macro for EIP-712 types
- `src/main.rs` — Command dispatch pattern, wallet loading, HttpClient construction
- `src/shell.rs` — Shell integration for new subcommands

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `contracts::resolve_addresses()` — extend with `multi_duel` field for new contract address
- `contracts::parse_usdc()` / `contracts::format_usdc()` — USDC amount handling
- `queue::generate_nonce()` — nonce generation with on-chain collision check (needs refactoring to accept contract address or exposing IClawDuel check)
- `HttpClient` — authenticated HTTP client with auth headers, `get()` and `post()` methods
- `security::sanitize_path_segment()` — for lobby ID in URL paths
- `OutputFormat` enum with `Table` / `Json` variants

### Established Patterns
- clap derive with `#[command(subcommand)]` for nested commands (see `WalletArgs`)
- `sol!` macro for EIP-712 struct definitions with `#[derive(Debug)]`
- `Eip712Domain` construction with chain_id from provider
- `SolStruct::eip712_signing_hash(&domain)` then `signer.sign_hash()`
- Dual output: `match fmt { Table => println!, Json => print_json }` pattern

### Integration Points
- `Commands` enum in `main.rs` — add `Lobby(LobbyArgs)` variant
- `commands/mod.rs` — add `pub mod lobby;`
- `contracts.rs` — add `JoinMultiAttestation` sol! struct (same fields), MultiDuel address
- `shell.rs` — add lobby subcommand parsing to shell dispatch

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches. Follow existing patterns closely.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 09-multi-duel-lobby-commands*
*Context gathered: 2026-03-20*
