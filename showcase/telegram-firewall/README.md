# Telegram Solana transaction approval firewall

This is the bounty showcase: a real ZeroClaw agent receives an unsigned Solana
transaction in Telegram, verifies its decoded bytes against explicit user
intent and operator-owned limits inside a jailed WASM component, then parks an
eligible proposal at a durable human checkpoint. It never signs or broadcasts.

## Pinned runtime

The configuration and commands below target ZeroClaw commit
707e0870df3988ab80a46759c50fae680ca3ccd9. Prebuilt ZeroClaw releases do not
include the WASM execution backend, so build the host from source:

~~~sh
git clone https://github.com/zeroclaw-labs/zeroclaw.git
cd zeroclaw
git checkout 707e0870df3988ab80a46759c50fae680ca3ccd9
rustup toolchain install 1.96.0 --profile minimal
cargo +1.96.0 build --release \
  --features plugins-wasm-cranelift,channel-telegram
~~~

## 1. Build and stage the plugin

~~~sh
cd /ABS/PATH/TO/ogige
rustup target add wasm32-wasip2
cargo test
cargo clippy --all-targets -- -D warnings
cargo build --target wasm32-wasip2 --release

mkdir -p /ABS/PATH/TO/zeroclaw-plugins/solana-guard
cp manifest.toml /ABS/PATH/TO/zeroclaw-plugins/solana-guard/
cp target/wasm32-wasip2/release/solana_guard.wasm \
  /ABS/PATH/TO/zeroclaw-plugins/solana-guard/solana_guard.wasm
~~~

The staged directory must contain exactly the manifest and the WASM file named
by wasm_path. The manifest grants only config_read.

## 2. Configure ZeroClaw

Merge config.fragment.toml into an existing config that already has a working
model-provider entry. Replace all placeholders and absolute paths. The
recipient allowlist must contain the actual decoded destination account; for
SPL transfers this is the destination token account.

Install the skill inside ZeroClaw's confined shared-bundle directory. For a
config directory at /ABS/ZEROCLAW_CONFIG this is:

~~~sh
mkdir -p /ABS/ZEROCLAW_CONFIG/shared/skills/ogige
cp -R showcase/telegram-firewall/skills/solana-transaction-firewall \
  /ABS/ZEROCLAW_CONFIG/shared/skills/ogige/
~~~

Leave directory unset in [skill_bundles.ogige]; ZeroClaw then resolves the
bundle to <install>/shared/skills/ogige. External absolute skill paths are
rejected by the workspace-containment validator.

Set secrets through masked prompts:

~~~sh
/ABS/PATH/TO/zeroclaw/target/release/zeroclaw \
  config set channels.telegram.firewall.bot_token
~~~

Do not add sop_approve to the agent's tools. The configured
out_of_band_required mode ensures the model cannot clear its own checkpoint.

## 3. Validate before starting

~~~sh
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
~~~

Every command loads and validates the config before acting. Do not start the
daemon until the plugin, skill, SOP, channel, and agent all resolve.

For the same component-level proof used during development, copy the included
host test into the pinned ZeroClaw checkout and run it:

~~~sh
cp showcase/telegram-firewall/host-tests/ogige_e2e.rs \
  /ABS/PATH/TO/zeroclaw/crates/zeroclaw-plugins/tests/ogige_e2e.rs
cd /ABS/PATH/TO/zeroclaw
cargo +1.96.0 test -p zeroclaw-plugins \
  --features plugins-wasm-cranelift --test ogige_e2e -- --nocapture
~~~

Adjust the WASM absolute path constant in the copied test if the ogige checkout
is elsewhere.

## 4. Run the real channel

~~~sh
$ZEROCLAW daemon
~~~

Telegram uses long polling, so no webhook or public inbound port is required.
Send the bot the exact base64 transaction plus purpose, full recipient, maximum
lamports/raw token units, and mint when applicable.

Expected behavior:

1. malformed input, a dangerous primitive, an over-cap amount, a mismatched
   recipient/mint, or invalid operator config returns REJECT;
2. unresolved lookup tables, unknown programs, or unchecked token mint returns
   HOLD;
3. a fully bound transaction returns ALLOW and opens the durable
   solana-transaction-review SOP;
4. the Telegram human approval clears the checkpoint, after which the agent
   returns only the unchanged unsigned proposal.

Inspect the audit trail:

~~~sh
$ZEROCLAW sop pending
$ZEROCLAW sop show solana-transaction-review
~~~

## 5. Demo evidence

Capture:

- daemon startup showing solana-guard loaded;
- Telegram request and compact verdict;
- a human checkpoint approve/deny interaction;
- a policy rejection that opens no checkpoint;
- the prompt-injection regression test;
- plugin/skill/SOP inventory and durable run status.

Use VIDEO_SCRIPT.md for a sub-three-minute cut. THREAT_MODEL.md defines the
security claim precisely; PROMPT_INJECTION_TEST.md provides the adversarial
case. Replace claims with recorded evidence only after the live Telegram run.

## Current external requirements

The repository supplies the component, policy, skill, SOP, tests, and runbook.
A live recording still requires two operator-owned secrets that are
intentionally absent from source control: a Telegram bot token and credentials
for an existing ZeroClaw model provider.
