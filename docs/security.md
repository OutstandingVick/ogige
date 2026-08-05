# Security model

## Claim

Ogige is a **T0/T1** safety workflow:

- **T0** — read / decode / classify  
- **T1** — produce policy verdicts over unsigned bytes; never hold signing keys  

The WASM component cannot sign, broadcast, open sockets, read arbitrary files, or call RPC. ALLOW is a **static policy match**, not an execution safety certificate.

## Asset and trust boundary

| | |
|---|---|
| **Protected asset** | The operator’s approval decision over unsigned Solana transaction bytes |
| **Trusted** | Operator-owned plugin configuration; compiled WASM component; out-of-band operator principal |
| **Untrusted** | Telegram text; claimed intent description; transaction bytes; decoded memo content; model output; SOP payload fields |

The ZeroClaw agent risk profile should exclude shell, HTTP, wallet-adjacent tools, delegation, model switching, and `sop_approve`.

## Attacks and controls

| Attack | Control | Residual risk |
|---|---|---|
| Prompt injection in purpose/memo | Skill treats decoded text as data; core uses description only as a non-empty audit label; SOP guards suspicious payloads | Model can summarize poorly; cannot change the Rust verdict |
| Recipient substitution | Decoded recipient must equal intent and appear on the operator allowlist | Token recipients are token accounts, not wallet owners |
| Amount inflation | Decoded amount must be ≤ per-request cap **and** operator cap | Token decimals must be supplied correctly as raw units |
| Mint substitution | TransferChecked mint must match intent and allowlist | Plain SPL Transfer cannot prove mint offline → HOLD |
| Hidden v0 accounts | Any address lookup table → HIGH / HOLD | Offline mode cannot resolve table contents |
| Unknown / CPI behavior | Unknown programs → HIGH / HOLD | Offline parsing cannot predict CPI or state-dependent behavior |
| Dangerous authority primitive | Approvals, ownership/authority changes, upgrades, permanent delegates → HIGH/CRITICAL | Taxonomy is not exhaustive |
| Policy typo or omission | Invalid values → CRITICAL; zero caps / empty allowlists deny value movement | Operator can deliberately set an unsafe cap |
| Agent self-approval | `sop_approve` excluded; SOP uses `out_of_band_required` | Compromise of authorized Telegram/operator identity |
| Replay / stale bytes | SHA-256 of exact bytes; optional durable-nonce binding | Nonce state can change after review; signatures are not verified |
| Compromised RPC | `rpc-enrich` is separate, bounded, advisory-only; cannot upgrade verdict | Enrichment may be stale, censored, or false |
| Wrong-run approval | Bridge accepts a strict run ID, verifies pending, scopes continuation | Local operator principal remains trusted |

## Danger signals detected offline

In addition to policy violations, the analyzer flags:

- System Assign / AssignWithSeed and durable-nonce authority changes  
- Misplaced / multiple nonce advances; nonce account / authority / value mismatch  
- Unlimited or ordinary token delegate approvals  
- Mint, freeze, token-owner, and program-upgrade authority changes  
- BPF program upgrades  
- Token-2022 permanent delegates, transfer hooks, and non-transferable mints  
- Token mint, burn, freeze, thaw, and close operations  
- Unknown programs and unresolved v0 address-lookup tables  

Malformed, non-canonical, truncated, structurally inconsistent, or trailing transaction bytes fail decoding.

## Explicit non-goals

Ogige does **not**:

- Prove runtime CPI or account-state outcomes  
- Sign or broadcast  
- Guarantee ordinary blockhash freshness  
- Trust address lookup table contents  
- Treat advisory RPC simulation as authoritative  

## Related

- Operator policy keys: [Configuration](configuration.md)  
- Injection regression notes: [`showcase/telegram-firewall/PROMPT_INJECTION_TEST.md`](../showcase/telegram-firewall/PROMPT_INJECTION_TEST.md)  
- Canonical threat table source: this page (replaces the old standalone showcase-only narrative for product docs)
