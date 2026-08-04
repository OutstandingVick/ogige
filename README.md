# ogige — policy-bound Solana approval firewall

Ogige is a real ZeroClaw agent workflow for reviewing unsigned Solana
transactions over Telegram. A jailed Rust/WASM tool decodes the transaction,
checks it against explicit user intent and operator-owned caps/allowlists, and
returns ALLOW, HOLD, or REJECT before a durable human approval checkpoint.

It never signs or broadcasts. This is deliberately a T0/T1 safety workflow, not
a wallet.

Built for the Superteam Brasil × ZeroClaw bounty. The complete showcase is in
[showcase/telegram-firewall](showcase/telegram-firewall/README.md).

## What makes the verdict policy-bound

A value transfer is eligible only when all three views agree:

1. the recipient, mint, and raw amount decoded from the transaction bytes;
2. the user's explicit expected recipient/mint and per-request maximum;
3. the operator's jailed recipient/mint allowlists and absolute caps.

Missing intent, zero/empty policy, malformed config, a mismatch, or an exceeded
cap fails closed. Plain SPL Transfer is held because it does not carry a mint;
TransferChecked can be fully bound offline.

The tool input is:

~~~json
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
~~~

The compact JSON output includes the verdict, narration, findings,
intent_bound, policy_configured, durable-nonce binding state, a SHA-256 identity
for the exact serialized bytes, transaction version, and structural counts.

## Operator policy

The host injects only this plugin's flat config through its sole config_read
permission:

| Key | Safe default | Meaning |
|---|---:|---|
| max_sol_lamports | 0 | Absolute native-transfer cap; zero denies SOL movement |
| max_token_amount | 0 | Absolute raw token-unit cap; zero denies token movement |
| allowed_recipients | empty | Comma-separated full base58 destination accounts |
| allowed_mints | empty | Comma-separated full base58 checked-transfer mints |
| require_durable_nonce | false | Require a valid advance instruction at index 0 |
| allowed_nonce_accounts | empty | Operator-approved durable nonce accounts |
| allowed_nonce_authorities | empty | Operator-approved nonce signer authorities |
| reject_on_critical | true | Critical finding produces REJECT |
| hold_on_high | true | High finding produces HOLD |
| hold_on_medium | false | Optionally require review for medium findings |

Invalid integers or pubkeys produce POLICY_CONFIG_INVALID at CRITICAL severity.

## Safety signals

In addition to policy violations, the analyzer detects:

- System Assign/AssignWithSeed and durable nonce authority changes;
- misplaced/multiple nonce advances and nonce account/authority/value mismatch;
- unlimited or ordinary token delegate approvals;
- mint, freeze, token-owner, and program-upgrade authority changes;
- BPF program upgrades;
- Token-2022 permanent delegates, transfer hooks, and non-transferable mints;
- token mint, burn, freeze, thaw, and close operations;
- unknown programs and unresolved v0 address-lookup tables.

Unknown programs, lookup tables, and unresolved token mints default to HOLD.
Malformed, non-canonical, truncated, structurally inconsistent, or trailing
transaction bytes fail decoding.

## Build and verify

~~~sh
rustup target add wasm32-wasip2
make verify
~~~

Current local suite includes unit/integration policy tests, official-Solana-SDK
differential fixtures, and property tests over arbitrary/trailing wire bytes.
CI reproduces the fixtures, runs clippy, and builds the WASM component. The
component has also passed the included end-to-end test through ZeroClaw 0.8.4's
real Wasmtime/Cranelift host.

## Repository layout

~~~text
src/core/       Solana wire decode, narration, and risk taxonomy
src/guard.rs    intent/policy binding and verdict engine
src/lib.rs      thin ZeroClaw WIT component shim
tests/          host tests over the same guard path used by WASM
examples/       synthetic and official-Solana-SDK fixture generators
fixtures/sdk/   reproducible official SDK transaction vectors + provenance
wit/v0/         pinned ZeroClaw plugin contract
showcase/       Telegram config, skill, SOP, threat model, fixtures, and scripts
manifest.toml   minimal tool capability with config_read only
~~~

## Security claim and limits

The component remains offline and deterministic with no RPC, files, wallet,
keys, signing, or broadcast access. ALLOW means only that statically decoded
fields match the configured policy and supplied intent. An optional external
`rpc-enrich` helper can fetch bounded read-only account/simulation evidence; it
is explicitly advisory and can never upgrade the Rust verdict. CPI, token
ownership, balance deltas, lookup-table trust, signatures, and ordinary
blockhash freshness remain outside the offline ALLOW claim.

## License

MIT OR Apache-2.0
