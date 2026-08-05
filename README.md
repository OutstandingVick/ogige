# Ogige

**Policy-bound Solana transaction approval firewall for ZeroClaw agents.**

Ogige reviews **unsigned** Solana transactions against explicit user intent and operator-owned limits, then parks eligible proposals at a durable **human checkpoint**. It never signs or broadcasts.

[Documentation](docs/index.md) · [Getting started](docs/getting-started.md) · [Security model](docs/security.md) · [API reference](docs/api-reference.md)

---

## Why it exists

Agents that can touch Solana need a brake pedal. Ogige is that brake:

1. Decode the exact wire bytes (legacy or v0) inside a jailed WASM tool  
2. Bind recipient / mint / amount to **user intent** and **operator policy**  
3. Return `ALLOW` / `HOLD` / `REJECT` with narration and findings  
4. On ALLOW, open an out-of-band human SOP — the agent cannot self-approve  

```text
Telegram → ZeroClaw agent → solana_guard (WASM) → verdict
                              │
                    ALLOW ────┼──► durable human checkpoint
                    HOLD  ────┼──► incomplete offline proof
                    REJECT ───┴──► stop (no checkpoint)
```

## Quick start

```sh
git clone https://github.com/OutstandingVick/ogige.git
cd ogige
rustup target add wasm32-wasip2
make verify    # tests, clippy, WASM build, fixture checks
make install   # stage plugin + skill (does not write secrets)
```

Then merge [`showcase/telegram-firewall/config.fragment.toml`](showcase/telegram-firewall/config.fragment.toml) into your ZeroClaw config, set the Telegram bot token via masked prompts, and follow the [Getting started](docs/getting-started.md) guide.

## Product docs

| Guide | Description |
|---|---|
| [Overview](docs/overview.md) | Product model and loop |
| [Architecture](docs/architecture.md) | WASM guard, skill, SOP, bridges |
| [Security](docs/security.md) | Trust boundary and non-goals |
| [Configuration](docs/configuration.md) | Caps, allowlists, nonce mode |
| [API reference](docs/api-reference.md) | Request / response schema |
| [Getting started](docs/getting-started.md) | Install and first run |
| [Operator guide](docs/operator-guide.md) | Approve, review, RPC enrich |
| [Telegram agent](docs/telegram-agent.md) | Channel behavior |
| [Verification](docs/verification.md) | Repro and evidence |

## Safety claim (short)

- Permission surface: **`config_read` only**  
- Custody: **T0/T1** — no keys, no signing, no broadcast  
- Fail-closed defaults: zero caps and empty allowlists deny value movement  
- Prompt injection cannot change the Rust verdict  
- Optional RPC enrichment is **advisory only** and lives outside the jail  

Full detail: [Security model](docs/security.md).

## Repository layout

```text
docs/                 Product documentation (start here)
src/                  WASM tool: decode, narrate, policy verdict
tests/                Host-run tests over the same guard path
fixtures/sdk/         Official Solana SDK differential vectors
showcase/telegram-firewall/
  skills/             Agent skill bundle
  sops/               Durable human-checkpoint workflow
  bin/                install · verify · approve · review · rpc-enrich
  config.fragment.toml
manifest.toml         Minimal tool capability
```

## License

MIT OR Apache-2.0
