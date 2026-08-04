# Threat model

## Asset and trust boundary

The asset is the operator's approval decision over unsigned Solana transaction
bytes. Telegram text, the claimed intent, the transaction itself, decoded memo
content, model output, and SOP payload are untrusted. The operator-owned plugin
configuration and the compiled WASM component are trusted.

The component has only config_read. It has no network, filesystem, wallet,
key, signing, simulation, or broadcast capability. The ZeroClaw risk profile
also removes shell, HTTP, signing-adjacent, delegation, model-switching, and
agent-side SOP approval tools.

## Attacks and controls

| Attack | Control | Residual risk |
|---|---|---|
| Prompt injection in purpose/memo | Skill treats decoded text as data; core uses the description only to require a non-empty audit label; SOP guard blocks suspicious untrusted payloads | A model can summarize poorly, but cannot change the Rust verdict |
| Recipient substitution | Decoded recipient must equal the explicit intent and appear in the operator allowlist | Token recipients are token accounts, not wallet owners |
| Amount inflation | Decoded amount must be below both the per-request cap and operator cap | Raw token decimals must be supplied correctly |
| Mint substitution | TransferChecked mint must match intent and allowlist | Plain SPL Transfer cannot prove mint offline and is held |
| Hidden v0 accounts | Any address lookup table produces HIGH/HOLD | RPC-free mode cannot resolve table contents |
| Unknown/CPI behavior | Unknown programs produce HIGH/HOLD | Offline parsing cannot predict CPI or account-state-dependent behavior |
| Dangerous authority primitive | Known approvals, ownership/authority changes, upgrades, permanent delegates produce HIGH/CRITICAL | Taxonomy is not exhaustive |
| Policy typo or omission | Invalid values become CRITICAL; zero caps and empty allowlists deny value movement | Operator can deliberately configure an unsafe cap |
| Agent self-approval | sop_approve excluded; SOP uses out_of_band_required, durable SQLite, and cancel-on-timeout | Compromise of authorized Telegram/operator identity |
| Replay/stale bytes | Human sees a durable run tied to exact input | No blockhash freshness or signature verification is performed |

## Explicit non-goals

This T0/T1 firewall does not prove runtime behavior, fetch account state,
simulate, sign, broadcast, estimate balance deltas, resolve lookup tables, or
guarantee blockhash freshness. ALLOW means “the decoded static fields match
this policy and intent,” never “safe to execute.”
