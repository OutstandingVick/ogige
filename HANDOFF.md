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
- 4 unit tests and 27 integration/property tests, including hostile-purpose
  invariance and official-SDK differential decoding.
- ZeroClaw WIT v0 logging contract synced to pinned upstream.
- Version bumped to 0.3.0.
- Official modular Solana SDK ordinary and durable-nonce fixtures with pinned
  crate provenance and reproducible generation.
- Durable nonce position/account/signer/value binding plus exact-byte SHA-256.
- Strict pending-run approval/resume bridge, routine JSON review command, and
  optional bounded advisory-only RPC enrichment.
- GitHub Actions plus one-command install and full verification workflows.
- Deterministic non-broadcastable fixture generator and checked-in fixture.
- Telegram skill with explicit untrusted-data and no-self-approval rules.
- Durable supervised SOP with a human checkpoint.
- Risk profile that excludes shell, HTTP, wallet-adjacent tools, delegation,
  model switching, and sop_approve.
- Threat model, injection test, submission draft, and three-minute video script.
- Reproducible runbook pinned to ZeroClaw commit
  707e0870df3988ab80a46759c50fae680ca3ccd9 and Rust 1.96.0.
- Real model-driven ZeroClaw turns using the existing Codex subscription:
  ALLOW persisted at the explicit checkpoint; hostile over-cap REJECT created
  no SOP run.

## Verification evidence

Local component:

~~~text
cargo test                                      PASS (4 + 27)
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

Live operator preflight after creating `@ogige_bot`:

~~~text
replacement token verified by Telegram getMe           PASS
masked ZeroClaw token secret present and encrypted      PASS
repository Telegram-token scan                          PASS, clean
local Telegram peer allowlist                           PASS, one numeric peer
zeroclaw plugin list / info                             PASS
zeroclaw skills audit / list --agent                    PASS
zeroclaw sop validate / show                            PASS
zeroclaw security status --agent solana_firewall       PASS, no warnings
zeroclaw channel doctor                                PASS, Telegram healthy
~~~

The host E2E test loaded the real WASM, read its metadata, returned ALLOW for a
fully bound 1 SOL fixture, then returned REJECT with SOL_CAP_EXCEEDED for the
same bytes plus hostile self-approval/broadcast prose under a 0.1 SOL operator
cap.

The full agent loop was then exercised with the installed skill and pinned
host. Its ALLOW run persisted the exact bytes, intent, and tool report with
status waiting_approval/current_step 1. The hostile REJECT run stopped with
SOL_CAP_EXCEEDED and the isolated SOP store contained zero runs.

The real `@ogige_bot` transport was then exercised through the live daemon.
The ALLOW case created run `run-1785855514930587000-0001`, which was cleared by
ZeroClaw's supported out-of-band CLI principal and completed at step 2/2 with
two recorded step results. Normal and hostile-purpose over-cap Telegram turns
both returned `SOL_CAP_EXCEEDED`. The durable store remained at one completed
run and zero nonterminal runs after both rejections.

## What is still required

The Telegram bot `@ogige_bot` now exists, and its token is stored in the
operator's local ZeroClaw config through the masked config prompt. The token is
not in this repository. The token was rotated after setup, the replacement was
verified against Telegram's `getMe` endpoint, and the masked local secret was
updated. The working model-provider profile is also configured locally and has
already driven the validation turns described above.

Remaining operator and submission work:

1. record and trim the video to under three minutes;
2. add the final video URL to SUBMISSION.md;
3. post the showcase in the bounty-designated Discord channel.

Do not reopen a registry PR during the bounty.

## Key files

- README.md — project overview and safety claim
- showcase/telegram-firewall/README.md — exact operator runbook
- showcase/telegram-firewall/config.fragment.toml — non-secret config
- showcase/telegram-firewall/skills/.../SKILL.md — agent behavior
- showcase/telegram-firewall/sops/... — durable workflow
- showcase/telegram-firewall/THREAT_MODEL.md — security boundary
- showcase/telegram-firewall/PROMPT_INJECTION_TEST.md — adversarial case
- showcase/telegram-firewall/EVIDENCE.md — real host and agent-loop evidence
- showcase/telegram-firewall/host-tests/ogige_e2e.rs — real host proof
- showcase/telegram-firewall/SUBMISSION.md — Discord post draft
- showcase/telegram-firewall/VIDEO_SCRIPT.md — recording plan
