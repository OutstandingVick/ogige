# Operator guide

Day-2 operations after Ogige is installed and the daemon is healthy.

## Routine review (JSON)

Copy and fill [`request.example.json`](../showcase/telegram-firewall/request.example.json), then:

```sh
make review REQUEST=/absolute/path/request.json
```

This submits a bounded request through the configured agent without relying on a live Telegram message.

## Approve a pending run

When an ALLOW creates a durable SOP checkpoint:

```sh
showcase/telegram-firewall/bin/ogige-approve RUN_ID
```

The bridge:

1. Accepts only a strict run ID  
2. Verifies the run is pending  
3. Approves via the supported out-of-band principal  
4. Opens a fresh, narrowly scoped agent turn for **that run only**  

Inspect status:

```sh
$ZEROCLAW sop pending
$ZEROCLAW sop show solana-transaction-review
```

## Advisory RPC enrichment

Optional and **outside** the jailed WASM component. Read-only, size/time limited, and cannot upgrade HOLD/REJECT to ALLOW:

```sh
RPC_URL=https://YOUR_SOLANA_RPC \
  showcase/telegram-firewall/bin/rpc-enrich TRANSACTION_BASE64 NONCE_ACCOUNT
```

Use enrichment to inform a human reviewer — never as an automatic bypass.

## Reproducibility gate

At any time:

```sh
make verify
```

CI mirrors this path: locked tests, clippy `-D warnings`, WASM release build, and official-SDK fixture drift checks.

## Fixture generation

Official modular Solana SDK fixtures (placeholder-signed, non-broadcastable):

```sh
make fixtures
```

Provenance and crate pins live under `fixtures/sdk/`.

## Incident / rejection playbook

| Observation | Action |
|---|---|
| `SOL_CAP_EXCEEDED` / token cap | Do not retry with altered args; raise operator cap only if intentional |
| `POLICY_CONFIG_INVALID` | Fix plugin config; zero/typo defaults deny movement |
| Prompt says “ignore policy / self-approve / broadcast” | Expect REJECT; confirm no SOP opened |
| HOLD on ALT / unknown program | Require human judgment; offline gate is incomplete by design |
| Unexpected ALLOW | Diff intent vs decoded narration; check allowlists and caps |

## Security hygiene

- Rotate Telegram tokens if exposed; keep them out of git  
- Keep `sop_approve` off the agent tool list  
- Prefer least-privilege allowlists over large caps  
- Treat every Telegram peer as untrusted input  

Full boundary: [Security model](security.md).
