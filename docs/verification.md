# Verification

How to prove Ogige behaves as documented.

## One-command gate

```sh
make verify
```

Typical local result:

```text
unit tests:                         PASS
policy integration tests:           PASS
SDK differential/property tests:    PASS
clippy -D warnings:                 PASS
wasm32-wasip2 release build:        PASS
official-SDK fixture drift:         CLEAN
ZeroClaw skill audit:               PASS
```

## What the suite covers

| Layer | Coverage |
|---|---|
| Unit | Encoding, core helpers |
| Integration / policy | Caps, allowlists, intent binding, hostile-purpose invariance |
| Differential | Official `solana-transaction` deserialization vs SDK-less decoder |
| Property | Arbitrary / trailing bytes do not panic; trailing data rejected |
| Host E2E | Real Wasmtime/Cranelift load of release WASM (`ogige_e2e.rs`) |
| CI | GitHub Actions reproduces fixtures, clippy, WASM build |

## Host E2E

Copy the included host test into a pinned ZeroClaw checkout:

```sh
cp showcase/telegram-firewall/host-tests/ogige_e2e.rs \
  /ABS/PATH/TO/zeroclaw/crates/zeroclaw-plugins/tests/ogige_e2e.rs
cargo +1.96.0 test -p zeroclaw-plugins \
  --features plugins-wasm-cranelift --test ogige_e2e -- --nocapture
```

Expected: ALLOW under a matching 1 SOL policy; REJECT with `SOL_CAP_EXCEEDED` for the same bytes under a 0.1 SOL cap with hostile prose.

## Validation evidence log

A dated operator validation log (daemon, agent loop, live Telegram runs) is kept as an **evidence artifact**, not product narrative:

→ [`showcase/telegram-firewall/EVIDENCE.md`](../showcase/telegram-firewall/EVIDENCE.md)

## Security regression

Prompt-injection / hostile-purpose invariance:

→ [`showcase/telegram-firewall/PROMPT_INJECTION_TEST.md`](../showcase/telegram-firewall/PROMPT_INJECTION_TEST.md)

## Versioning

| Artifact | Version |
|---|---|
| Crate / plugin | `0.3.0` (see `Cargo.toml`, `manifest.toml`) |
| Validated ZeroClaw host | commit `707e0870df3988ab80a46759c50fae680ca3ccd9` |
| Rust for host build | `1.96.0` |
