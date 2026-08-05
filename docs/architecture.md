# Architecture

Ogige is a **product workflow**, not a single binary. Four layers work together.

```text
┌─────────────────────────────────────────────────────────┐
│  Channel: Telegram (ZeroClaw long-polling)              │
└────────────────────────────┬────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────┐
│  Agent skill: solana-transaction-firewall               │
│  Collects intent · calls tool · starts SOP · never      │
│  self-approves · never signs                            │
└────────────────────────────┬────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────┐
│  WASM tool: solana_guard (wasm32-wasip2 WIT component)  │
│  Decode · narrate · risk taxonomy · intent/policy bind  │
│  Permission: config_read only                           │
└────────────────────────────┬────────────────────────────┘
                             │ ALLOW
┌────────────────────────────▼────────────────────────────┐
│  Durable SOP: solana-transaction-review                 │
│  out_of_band_required checkpoint · SQLite audit trail   │
└────────────────────────────┬────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────┐
│  Operator bridges (host CLI, outside the jail)          │
│  ogige-approve · ogige-review · rpc-enrich (advisory)   │
└─────────────────────────────────────────────────────────┘
```

## 1. WASM guard (`solana_guard`)

| Property | Value |
|---|---|
| Package | `solana-guard` (crate) / plugin name `solana-guard` |
| Tool name | `solana_guard` |
| Target | `wasm32-wasip2` WIT `tool-plugin` world |
| Permissions | `config_read` only |
| Layout | Pure Rust core (`src/core`, `src/guard.rs`) + thin WASM shim (`src/lib.rs`) |

Responsibilities:

- Decode legacy and v0 Solana transactions (SDK-less)  
- Narrate instructions in plain language  
- Classify danger primitives (authority changes, unlimited approvals, upgrades, Token-2022 hooks, …)  
- Bind decoded fields to user intent + operator policy  
- Emit compact JSON with SHA-256 of the exact input bytes  
- Optionally enforce durable-nonce position / account / authority / value  

## 2. Telegram skill

Located at `showcase/telegram-firewall/skills/solana-transaction-firewall/`.

Teaches the agent how to:

- Collect required fields without inventing values  
- Treat purpose/memo/decoded text as **untrusted data**  
- Call `solana_guard` exactly once with user-supplied intent  
- Start the SOP only on ALLOW (or explicit HOLD review)  
- Never call `sop_approve`, shell, HTTP, or wallet tools  

## 3. Durable SOP

`showcase/telegram-firewall/sops/solana-transaction-review/`

- Parks ALLOW payloads at a human checkpoint  
- Requires an **external** principal (CLI / HTTP / dashboard) — not the agent  
- Cancel-on-timeout; durable SQLite audit record  

## 4. Operator tooling

| Command | Role |
|---|---|
| `make install` | Build WASM, stage plugin, install skill bundle |
| `make verify` | Full reproducibility gate |
| `make review REQUEST=…` | Bounded JSON review through the configured agent |
| `ogige-approve RUN_ID` | Strict pending-run approval + scoped resume |
| `rpc-enrich` | Optional read-only RPC evidence (**advisory only**) |

## Trust split

| Trusted | Untrusted |
|---|---|
| Compiled WASM + operator plugin config | Telegram text, intent description, memo |
| Operator host CLI / out-of-band principal | Model output and SOP payload content |
| Offline Rust verdict | Optional RPC enrichment |

See [Security model](security.md) for the full boundary.
