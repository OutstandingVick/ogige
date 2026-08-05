# API reference

## Tool

| | |
|---|---|
| Plugin name | `solana-guard` |
| Tool name | `solana_guard` |
| Permission | `config_read` |

## Request

```json
{
  "transaction": "<base64 legacy-or-v0 transaction>",
  "intent": {
    "description": "Pay an approved invoice",
    "expected_recipient": "<full base58 account>",
    "expected_mint": null,
    "max_lamports": 100000000,
    "max_token_amount": 0,
    "expected_nonce_account": null,
    "expected_nonce_authority": null,
    "expected_nonce_value": null
  }
}
```

A ready-to-copy template lives at [`showcase/telegram-firewall/request.example.json`](../showcase/telegram-firewall/request.example.json).

### Intent fields

| Field | Type | Required | Notes |
|---|---|---|---|
| `description` | string | yes | Non-empty audit label only; not a policy input |
| `expected_recipient` | base58 | yes | Must match decoded destination |
| `expected_mint` | base58 or null | conditional | Required for TransferChecked; null for native SOL |
| `max_lamports` | u64 | yes | Per-request SOL cap (0 if unused) |
| `max_token_amount` | u64 | yes | Per-request raw token units (0 if unused) |
| `expected_nonce_account` | base58 or null | if nonce mode | Must match advance account |
| `expected_nonce_authority` | base58 or null | if nonce mode | Required signer authority |
| `expected_nonce_value` | string/base58 or null | if nonce mode | Current nonce value from trusted builder |

Use **raw integer units**: lamports for SOL, raw token units for SPL (not UI decimals).

## Response

Compact JSON `GuardReport` (field names may expand slightly by version; treat unknown fields as forward-compatible):

| Field | Meaning |
|---|---|
| `verdict` | `ALLOW` \| `HOLD` \| `REJECT` |
| `summary` | One-line human summary |
| `narration` | Multi-line plain-language decode |
| `findings` | Ordered list of `{ code, severity, instruction_index, message }` |
| `intent_bound` | Decoded fields matched supplied intent |
| `policy_configured` | Operator policy was present and parseable |
| `tx_version` | `legacy` or `v0` |
| `instruction_count` / `account_count` | Structural counts |
| SHA-256 digest field | Identity of the exact serialized input bytes |
| Durable-nonce binding fields | Present when nonce checks apply |

### Severity → verdict (defaults)

| Max severity | Default outcome |
|---|---|
| CRITICAL | REJECT (`reject_on_critical=true`) |
| HIGH | HOLD (`hold_on_high=true`) |
| MEDIUM | ALLOW unless `hold_on_medium=true` |
| LOW / none | ALLOW (still subject to policy binding) |

Policy mismatches and cap breaches produce dedicated finding codes (e.g. `SOL_CAP_EXCEEDED`) and fail closed regardless of prose in `description`.

## Example finding codes

| Code | Typical severity |
|---|---|
| `SOL_CAP_EXCEEDED` / token-cap equivalents | CRITICAL / policy REJECT |
| `SYSTEM_ASSIGN` | CRITICAL |
| `TOKEN_APPROVE_MAX` | CRITICAL |
| `MINT_AUTHORITY_CHANGE` / `FREEZE_AUTHORITY_CHANGE` / `TOKEN_OWNER_CHANGE` | CRITICAL |
| `PROGRAM_UPGRADE` / `UPGRADE_AUTHORITY_CHANGE` | CRITICAL |
| `NONCE_AUTHORIZE` / nonce mismatch codes | HIGH |
| `UNKNOWN_PROGRAM` / `ALT_USED` | MEDIUM→HIGH / HOLD |
| `SOL_TRANSFER` / `TOKEN_TRANSFER` | LOW |

## Host-callable review

Operators can submit a filled request JSON without Telegram:

```sh
make review REQUEST=/absolute/path/request.json
```

This uses the bounded `ogige-review` bridge against the configured agent.
