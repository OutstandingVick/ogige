# Telegram firewall showcase pack

Operational assets for running Ogige as a ZeroClaw Telegram approval firewall.

**Product documentation lives in [`docs/`](../../docs/index.md)** — start there for overview, security, API, and getting started.

This directory holds the runnable pack: config fragment, skill, SOP, host test, fixtures, and operator scripts.

## Contents

| Path | Role |
|---|---|
| `config.fragment.toml` | Non-secret ZeroClaw merge fragment |
| `skills/solana-transaction-firewall/` | Agent skill |
| `sops/solana-transaction-review/` | Durable human-checkpoint SOP |
| `bin/` | `install`, `verify`, `ogige-approve`, `ogige-review`, `rpc-enrich` |
| `host-tests/ogige_e2e.rs` | Real Cranelift host proof |
| `fixtures/` | Demo transaction JSON |
| `request.example.json` | Bounded review template |
| `EVIDENCE.md` | Dated validation log (artifact) |
| `PROMPT_INJECTION_TEST.md` | Adversarial regression notes |
| `VIDEO_SCRIPT.md` | Demo recording outline |
| `SUBMISSION.md` | External submission draft |
| `THREAT_MODEL.md` | Showcase copy — canonical product page is [docs/security.md](../../docs/security.md) |
| `SCORECARD.md` | Internal self-score notes |

## Quick path

```sh
# from repo root
make verify
make install
# merge config.fragment.toml, set secrets, then:
# $ZEROCLAW daemon
```

Full steps: [Getting started](../../docs/getting-started.md) · [Operator guide](../../docs/operator-guide.md)
