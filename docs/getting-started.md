# Getting started

This guide gets Ogige running against a source-built ZeroClaw host with Telegram.

Pinned runtime used for validation: ZeroClaw commit
`707e0870df3988ab80a46759c50fae680ca3ccd9`, Rust `1.96.0`, features
`plugins-wasm-cranelift,channel-telegram`.

## Prerequisites

- Rust toolchain with `wasm32-wasip2`  
- A working ZeroClaw **model provider** config  
- Telegram bot token (stored only in ZeroClaw’s masked secrets)  

Prebuilt ZeroClaw releases may omit the WASM execution backend — build the host from source:

```sh
git clone https://github.com/zeroclaw-labs/zeroclaw.git
cd zeroclaw
git checkout 707e0870df3988ab80a46759c50fae680ca3ccd9
rustup toolchain install 1.96.0 --profile minimal
cargo +1.96.0 build --release \
  --features plugins-wasm-cranelift,channel-telegram
```

## 1. Clone and verify Ogige

```sh
git clone https://github.com/OutstandingVick/ogige.git
cd ogige
rustup target add wasm32-wasip2
make verify
```

`make verify` runs the locked test suite, clippy, WASM release build, fixture drift checks, and skill audit hooks used in CI.

## 2. Install the plugin + skill

```sh
make install
```

This builds the component and stages it for ZeroClaw. It does **not** edit your config or request secrets.

Manual equivalent (adjust paths):

```sh
cargo build --target wasm32-wasip2 --release
mkdir -p /ABS/PATH/TO/zeroclaw-plugins/solana-guard
cp manifest.toml /ABS/PATH/TO/zeroclaw-plugins/solana-guard/
cp target/wasm32-wasip2/release/solana_guard.wasm \
  /ABS/PATH/TO/zeroclaw-plugins/solana-guard/solana_guard.wasm
```

The staged directory must contain exactly the manifest and the WASM file named by `wasm_path`.

Install the skill into ZeroClaw’s confined shared-bundle directory:

```sh
mkdir -p /ABS/ZEROCLAW_CONFIG/shared/skills/ogige
cp -R showcase/telegram-firewall/skills/solana-transaction-firewall \
  /ABS/ZEROCLAW_CONFIG/shared/skills/ogige/
```

Leave `directory` unset under `[skill_bundles.ogige]` so ZeroClaw resolves
`<install>/shared/skills/ogige`. External absolute skill paths are rejected by
the workspace-containment validator.

## 3. Merge configuration

1. Copy [`showcase/telegram-firewall/config.fragment.toml`](../showcase/telegram-firewall/config.fragment.toml)  
2. Merge into an existing ZeroClaw config that already has a model provider  
3. Replace placeholders and absolute paths  
4. Set recipient/mint allowlists and caps — see [Configuration](configuration.md)  

Set secrets through masked prompts:

```sh
/ABS/PATH/TO/zeroclaw/target/release/zeroclaw \
  config set channels.telegram.firewall.bot_token
```

**Do not** add `sop_approve` to the agent’s tools.

## 4. Preflight

```sh
ZEROCLAW=/ABS/PATH/TO/zeroclaw/target/release/zeroclaw
$ZEROCLAW config list --filter plugins
$ZEROCLAW plugin list
$ZEROCLAW skills audit \
  /ABS/PATH/TO/ogige/showcase/telegram-firewall/skills/solana-transaction-firewall
$ZEROCLAW skills list --agent solana_firewall
$ZEROCLAW sop validate solana-transaction-review
$ZEROCLAW sop show solana-transaction-review
$ZEROCLAW channel doctor
$ZEROCLAW security status --agent solana_firewall
```

Do not start the daemon until plugin, skill, SOP, channel, and agent all resolve cleanly.

Optional host-level WASM proof (copy into the pinned ZeroClaw checkout):

```sh
cp showcase/telegram-firewall/host-tests/ogige_e2e.rs \
  /ABS/PATH/TO/zeroclaw/crates/zeroclaw-plugins/tests/ogige_e2e.rs
cd /ABS/PATH/TO/zeroclaw
cargo +1.96.0 test -p zeroclaw-plugins \
  --features plugins-wasm-cranelift --test ogige_e2e -- --nocapture
```

## 5. Run

```sh
$ZEROCLAW daemon
```

Telegram uses long polling — no webhook or public inbound port is required.

Send the bot:

- exact base64 transaction  
- purpose  
- full base58 recipient  
- max lamports / raw token units  
- mint when applicable  

Expected:

1. **REJECT** — malformed input, dangerous primitive, over-cap, mismatch, bad config  
2. **HOLD** — unresolved ALTs, unknown programs, unchecked mint  
3. **ALLOW** — opens durable `solana-transaction-review` SOP  

Clear a checkpoint with the operator bridge:

```sh
showcase/telegram-firewall/bin/ogige-approve RUN_ID
```

The transaction bytes remain **unchanged, unsigned, and unbroadcast**.

## Next

- [Operator guide](operator-guide.md)  
- [Telegram agent](telegram-agent.md)  
- [API reference](api-reference.md)  
