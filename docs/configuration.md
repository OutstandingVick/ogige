# Configuration

The host injects only this plugin’s flat config section when the manifest declares `config_read`. The plugin cannot read global config or another plugin’s section.

Absent or empty keys fall back to **fail-closed** defaults.

## Policy keys

| Key | Safe default | Meaning |
|---|---:|---|
| `max_sol_lamports` | `0` | Absolute native-transfer cap; zero denies SOL movement |
| `max_token_amount` | `0` | Absolute raw token-unit cap; zero denies token movement |
| `allowed_recipients` | empty | Comma-separated full base58 destination accounts |
| `allowed_mints` | empty | Comma-separated full base58 checked-transfer mints |
| `require_durable_nonce` | `false` | Require a valid advance instruction at index 0 |
| `allowed_nonce_accounts` | empty | Operator-approved durable nonce accounts |
| `allowed_nonce_authorities` | empty | Operator-approved nonce signer authorities |
| `reject_on_critical` | `true` | Critical finding → REJECT |
| `hold_on_high` | `true` | High finding → HOLD |
| `hold_on_medium` | `false` | Optionally HOLD on medium findings |

Invalid integers or pubkeys produce `POLICY_CONFIG_INVALID` at CRITICAL severity.

## Example fragment

Merge the non-secret fragment from the showcase into your ZeroClaw config after replacing placeholders and absolute paths. Plugin policy lives under the plugin entry config:

```toml
[[plugins.entries]]
name = "solana-guard"

[plugins.entries.config]
max_sol_lamports = "100000000"
max_token_amount = "1000000"
allowed_recipients = "<RECIPIENT_ACCOUNT_1>,<RECIPIENT_ACCOUNT_2>"
allowed_mints = "<MINT_ACCOUNT_1>"
require_durable_nonce = "false"
reject_on_critical = "true"
hold_on_high = "true"
hold_on_medium = "true"
```

Full mergeable template (channels, risk profile, skill bundle, SOP paths):  
[`showcase/telegram-firewall/config.fragment.toml`](../showcase/telegram-firewall/config.fragment.toml)

## Secrets

Telegram bot tokens and model-provider credentials belong in ZeroClaw’s **masked / encrypted** config prompts — never in this repository.

```sh
zeroclaw config set channels.telegram.firewall.bot_token
```

## Durable-nonce profile

For offline durable-nonce reviews:

1. Set `require_durable_nonce=true`  
2. Populate `allowed_nonce_accounts` and `allowed_nonce_authorities`  
3. Require `expected_nonce_account`, `expected_nonce_authority`, and `expected_nonce_value` in every intent  

See [API reference](api-reference.md).

## Fail-closed checklist

Before going live, confirm:

- [ ] Caps are non-zero only for assets you intend to allow  
- [ ] Recipient allowlist contains the **decoded destination account** (for SPL, the destination **token account**)  
- [ ] Mint allowlist is set for TransferChecked flows  
- [ ] Agent tools do **not** include `sop_approve`  
- [ ] SOP mode is `out_of_band_required`  
