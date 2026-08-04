# HANDOFF — ogige bounty submission

Date: 2026-08-04

Repository: https://github.com/OutstandingVick/ogige

Local path: /Users/macbook/macbook/ogige

## Current direction

The old “plugin-only registry submission” direction was ineligible under the
actual bounty brief. The strongest pivot is now implemented:

**Telegram Solana transaction approval firewall**

A real ZeroClaw agent receives unsigned Solana transaction bytes and explicit
intent over Telegram. The solana_guard WASM component compares decoded
recipient/mint/amount fields against both the user's limits and operator-owned
caps/allowlists, returning ALLOW, HOLD, or REJECT. Eligible proposals enter a
durable SOP with an out-of-band human checkpoint.

The component is T0/T1 only and can never sign or broadcast.

The prohibited draft registry PR
https://github.com/zeroclaw-labs/zeroclaw-plugins/pull/151 was closed on
2026-08-04. Its fork branch was preserved.

## Implemented

- Policy-bound SOL and TransferChecked token verification.
- Fail-closed zero defaults, invalid configuration, missing intent, recipient
  mismatch, mint mismatch, and amount-cap enforcement.
- Existing dangerous instruction taxonomy retained and expanded into the
  policy verdict.
- Compact tool output and required structured intent schema.
- 4 unit tests and 20 integration tests, including hostile-purpose invariance.
- ZeroClaw WIT v0 logging contract synced to pinned upstream.
- Version bumped to 0.2.0.
- Deterministic non-broadcastable fixture generator and checked-in fixture.
- Telegram skill with explicit untrusted-data and no-self-approval rules.
- Durable supervised SOP with a human checkpoint.
- Risk profile that excludes shell, HTTP, wallet-adjacent tools, delegation,
  model switching, and sop_approve.
- Threat model, injection test, submission draft, and three-minute video script.
- Reproducible runbook pinned to ZeroClaw commit
  707e0870df3988ab80a46759c50fae680ca3ccd9 and Rust 1.96.0.

## Verification evidence

Local component:

~~~text
cargo test                                      PASS (4 + 20)
cargo clippy --all-targets -- -D warnings       PASS
cargo build --target wasm32-wasip2 --release   PASS
git diff --check                                PASS
~~~

Pinned ZeroClaw 0.8.4 source host:

~~~text
cargo +1.96.0 build --release \
  --features plugins-wasm-cranelift,channel-telegram    PASS
zeroclaw plugin list / info                             PASS
zeroclaw skills audit / list --agent                    PASS
zeroclaw sop validate / show                            PASS
zeroclaw security status --agent solana_firewall       PASS, no warnings
ogige_e2e through real Cranelift host                   PASS
~~~

The host E2E test loaded the real WASM, read its metadata, returned ALLOW for a
fully bound 1 SOL fixture, then returned REJECT with SOL_CAP_EXCEEDED for the
same bytes plus hostile self-approval/broadcast prose under a 0.1 SOL operator
cap.

## What is still required

Two operator-owned credentials are intentionally not in the repository:

1. a Telegram bot token;
2. a working ZeroClaw model-provider credential/profile.

With those available:

1. merge showcase/telegram-firewall/config.fragment.toml into the live config;
2. set the Telegram token through ZeroClaw's masked config prompt;
3. start the daemon and perform ALLOW/checkpoint, REJECT, and injection flows;
4. record and trim the video to under three minutes;
5. add the final video URL to SUBMISSION.md;
6. post the showcase in the bounty-designated Discord channel.

Do not reopen a registry PR during the bounty.

## Key files

- README.md — project overview and safety claim
- showcase/telegram-firewall/README.md — exact operator runbook
- showcase/telegram-firewall/config.fragment.toml — non-secret config
- showcase/telegram-firewall/skills/.../SKILL.md — agent behavior
- showcase/telegram-firewall/sops/... — durable workflow
- showcase/telegram-firewall/THREAT_MODEL.md — security boundary
- showcase/telegram-firewall/PROMPT_INJECTION_TEST.md — adversarial case
- showcase/telegram-firewall/host-tests/ogige_e2e.rs — real host proof
- showcase/telegram-firewall/SUBMISSION.md — Discord post draft
- showcase/telegram-firewall/VIDEO_SCRIPT.md — recording plan
