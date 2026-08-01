# ogige

ZeroClaw **WIT tool plugin** — a Solana transaction safety gate for autonomous agents.

Pass a base64-encoded transaction. Get back:

1. A **human-readable narration** of what the transaction does
2. Structured **risk findings** (authority changes, unlimited approvals, program upgrades, …)
3. A fail-closed verdict: **ALLOW** / **HOLD** / **REJECT**

Never signs. Never broadcasts. Custody tier **T0/T1** only — the agent proposes, a human (or ZeroClaw approval gate) decides.

Built for the [Superteam Brasil × ZeroClaw bounty](https://superteam.fun/earn/listing/zeroclaw).

## Why it exists

Agents that can touch Solana need a brake pedal. `ogige` is that brake: decode → narrate → classify → verdict, all inside the `wasm32-wasip2` sandbox with no `solana-sdk`.

## Tool surface

| | |
|---|---|
| Plugin name | `ogige` |
| Tool name | `solana_guard` |
| Input | `{ "transaction": "<base64>" }` |
| Output | JSON `GuardReport` (verdict, summary, narration, findings, …) |

### Example verdict

```json
{
  "verdict": "REJECT",
  "summary": "REJECT — dangerous primitive detected (TOKEN_APPROVE_MAX)",
  "narration": "Solana legacy transaction · …\n1. [SPL Token] Approve MAX (unlimited) delegate → …",
  "findings": [
    {
      "code": "TOKEN_APPROVE_MAX",
      "severity": "CRITICAL",
      "instruction_index": 0,
      "message": "Approve with u64::MAX — unlimited spending delegate"
    }
  ]
}
```

## Danger primitives (v0.1)

| Code | Severity | Trigger |
|---|---|---|
| `SYSTEM_ASSIGN` | CRITICAL | System Program Assign / AssignWithSeed |
| `TOKEN_APPROVE_MAX` | CRITICAL | SPL Approve with `u64::MAX` |
| `MINT_AUTHORITY_CHANGE` / `FREEZE_AUTHORITY_CHANGE` / `TOKEN_OWNER_CHANGE` | CRITICAL | Token SetAuthority |
| `PROGRAM_UPGRADE` / `UPGRADE_AUTHORITY_CHANGE` | CRITICAL | BPF Upgradeable Loader |
| `NONCE_AUTHORIZE` | HIGH | Durable nonce authority change |
| `TOKEN_APPROVE` / `TOKEN_MINT_TO` | HIGH | Delegates / minting |
| `ALT_USED` / `UNKNOWN_PROGRAM` | MEDIUM | Hidden accounts / unrecognized programs |
| `SOL_TRANSFER` / `TOKEN_TRANSFER` | LOW | Normal transfers (ALLOW by default) |

## Config keys

Injected via the plugin's jailed `__config` section when `config_read` is granted (not required today — defaults are fail-closed):

| Key | Default | Meaning |
|---|---|---|
| `reject_on_critical` | `true` | Critical findings → REJECT |
| `hold_on_high` | `true` | High findings → HOLD |
| `hold_on_medium` | `false` | Medium findings → HOLD |

## Layout

```
src/core/     # SDK-less Solana decode / narrate / risk (no wasm deps)
src/guard.rs  # analyze() → GuardReport
src/lib.rs    # thin #[cfg(target_family = "wasm")] WIT shim
tests/        # host-run fixtures (cargo test)
wit/v0/       # vendored ZeroClaw tool-plugin contract
manifest.toml
```

## Build and test

```bash
cargo test
rustup target add wasm32-wasip2
cargo build --target wasm32-wasip2 --release
cp target/wasm32-wasip2/release/ogige.wasm ogige.wasm
```

## Roadmap

- [ ] Optional RPC enrichment (`simulateTransaction`, mint/authority lookups) behind `http_client`
- [ ] Token-2022 transfer-hook / permanent-delegate detection
- [ ] Squads / multisig CPI surface narration
- [ ] Fixture corpus from real exploit txs

## License

MIT OR Apache-2.0
