# Product overview

## What Ogige is

Ogige turns a ZeroClaw agent into a **policy-bound review desk** for unsigned Solana transactions.

A user (typically over Telegram) submits:

1. the exact base64-encoded transaction bytes, and  
2. explicit intent (recipient, amount caps, mint when applicable).

A jailed Rust/WASM tool (`solana_guard`) decodes the wire bytes offline, compares them to both the per-request intent and the operator’s jailed allowlists/caps, and returns a structured verdict:

| Verdict | Meaning |
|---|---|
| **ALLOW** | Decoded fields match intent and operator policy. Eligible for a durable human checkpoint. |
| **HOLD** | Offline proof is incomplete (e.g. unknown program, address lookup tables, unchecked mint). |
| **REJECT** | Policy violation or dangerous primitive. Final for that request — no checkpoint. |

The component has **only** `config_read`. It has no RPC, filesystem, wallet, keys, signing, or broadcast capability.

## Who it is for

- Operators running self-hosted ZeroClaw agents that must touch Solana safely  
- Teams that need a fail-closed approval gate before any human offline-signing step  
- Security-minded workflows where the model must not be able to self-approve or bypass policy  

## What it is not

- Not a wallet  
- Not a signer or broadcaster  
- Not a full Solana SDK inside WASM  
- Not a proof of runtime CPI / balance outcomes (see [Security model](security.md))  

**ALLOW means:** “these static decoded fields match this policy and intent.”  
**ALLOW does not mean:** “safe to execute on-chain.”

## Core product loop

```text
Telegram user
    │  base64 tx + explicit intent
    ▼
ZeroClaw agent + Ogige skill
    │  calls solana_guard (WASM)
    ▼
Policy verdict (ALLOW / HOLD / REJECT)
    │
    ├─ REJECT → stop, report findings
    ├─ HOLD   → explain gap; optional human review on request
    └─ ALLOW  → durable SOP → out-of-band human approval → resume
                (bytes remain unsigned / unbroadcast)
```

## Why policy is binding

A value transfer is eligible only when **all three** views agree:

1. **Decoded bytes** — recipient, mint, and raw amount from the transaction  
2. **User intent** — expected recipient/mint and per-request maximums  
3. **Operator policy** — absolute caps plus recipient/mint allowlists  

Missing intent, zero/empty policy, malformed config, mismatch, or exceeded cap **fails closed**. Prose in the purpose field cannot change the Rust verdict.

## Next steps

- [Getting started](getting-started.md) — install and run  
- [Configuration](configuration.md) — set operator caps  
- [API reference](api-reference.md) — request / response shapes  
