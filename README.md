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
    "max_token_amount": 0
  }
}
~~~

The compact JSON output includes the verdict, narration, findings,
intent_bound, policy_configured, transaction version, and structural counts.

## Operator policy

The host injects only this plugin's flat config through its sole config_read
permission:

| Key | Safe default | Meaning |
|---|---:|---|
| max_sol_lamports | 0 | Absolute native-transfer cap; zero denies SOL movement |
| max_token_amount | 0 | Absolute raw token-unit cap; zero denies token movement |
| allowed_recipients | empty | Comma-separated full base58 destination accounts |
| allowed_mints | empty | Comma-separated full base58 checked-transfer mints |
| reject_on_critical | true | Critical finding produces REJECT |
| hold_on_high | true | High finding produces HOLD |
| hold_on_medium | false | Optionally require review for medium findings |

Invalid integers or pubkeys produce POLICY_CONFIG_INVALID at CRITICAL severity.

## Safety signals

In addition to policy violations, the analyzer detects:

- System Assign/AssignWithSeed and durable nonce authority changes;
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
cargo test
cargo clippy --all-targets -- -D warnings
cargo build --target wasm32-wasip2 --release
cargo run --quiet --example generate_demo_fixture
~~~

Current local suite: 4 unit tests and 20 integration tests. The WASM component
has also passed the included end-to-end test through ZeroClaw 0.8.4's real
Wasmtime/Cranelift host.

## Repository layout

~~~text
src/core/       Solana wire decode, narration, and risk taxonomy
src/guard.rs    intent/policy binding and verdict engine
src/lib.rs      thin ZeroClaw WIT component shim
tests/          host tests over the same guard path used by WASM
examples/       deterministic non-broadcastable demo fixture generator
wit/v0/         pinned ZeroClaw plugin contract
showcase/       Telegram config, skill, SOP, threat model, fixtures, and scripts
manifest.toml   minimal tool capability with config_read only
~~~

## Security claim and limits

The component is offline and deterministic with no RPC, files, wallet, keys,
signing, or broadcast access. ALLOW means only that statically decoded fields
match the configured policy and supplied intent. It is not proof of runtime
behavior: CPI, account state, token ownership, balance deltas, lookup-table
contents, simulation results, signatures, and blockhash freshness remain out of
scope.

## License

MIT OR Apache-2.0
