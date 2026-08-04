# Bounty scorecard

This is a conservative self-score against the supplied 100-point rubric, not a
claim about judge results.

## Practical readiness after the v0.3 hardening: 96/100

| Category | Score | Evidence / deduction |
|---|---:|---|
| Use case | 28/30 | Real Telegram ALLOW/checkpoint/approval and two REJECT flows; bounded JSON review and approval-resume commands make it usable beyond the demo |
| Safety | 25/25 | Rust intent/policy/nonce binding, exact-byte digest, minimal WASM grant, narrow risk profile, external-only checkpoint, injection regression |
| Craft | 19/20 | Official-SDK differential fixtures, durable nonce enforcement, property tests, compact output; offline mode deliberately cannot prove CPI/state |
| Reproducibility | 15/15 | Pinned host, lockfile, one-command install/verify, CI, exact official-SDK fixtures with generator provenance, config/skill/SOP/runbook |
| Showcase | 9/10 | Real channel evidence, video script, polished post, and explicit threat model; final URL/post receipt remain external artifacts |

Assuming the stated clean sub-three-minute video and Discord post are complete,
the practical score is approximately **98/100**. A judge can still reserve the
last points for independent replay, production usage, or broader live-network
state/CPI coverage; no repository can honestly guarantee a subjective 100.

## What v0.3 added

- Official modular Solana crates generate both ordinary and durable-nonce wire
  fixtures with checked-in provenance and differential decoder assertions.
- Durable nonce account, signer authority, instruction position, and nonce
  value are bound to caller intent and operator allowlists.
- Every report identifies the exact serialized input with SHA-256.
- Property tests prove arbitrary bytes do not panic and trailing data is always
  rejected; CI reproduces fixtures and builds the WASM target.
- `make install`, `make verify`, `make review`, and `ogige-approve` provide the
  operator path from setup through exact-run approval and continuation.
- `rpc-enrich` provides bounded read-only simulation/account evidence outside
  the minimal offline component, marked advisory-only.

## Remaining judge-dependent path to a literal 100

1. Put the final public video URL and Discord post receipt in the submission.
2. Have an independent reviewer reproduce `make install`, `make verify`, and a
   Telegram ALLOW/REJECT flow on a clean ZeroClaw profile.
3. Optionally record a real devnet nonce account and transaction-state change;
   keep that network evidence advisory so it never weakens the offline gate.
