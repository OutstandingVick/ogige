# Ogige documentation

**Ogige** is a policy-bound Solana transaction approval firewall for [ZeroClaw](https://github.com/zeroclaw-labs/zeroclaw) agents.

It reviews **unsigned** Solana transactions against explicit user intent and operator-owned limits, then parks eligible proposals at a **human checkpoint**. It never signs or broadcasts.

| Guide | Description |
|---|---|
| [Product overview](overview.md) | What Ogige is, who it is for, and the ALLOW / HOLD / REJECT model |
| [Architecture](architecture.md) | WASM guard, Telegram skill, durable SOP, and operator bridges |
| [Security model](security.md) | Trust boundaries, attacks & controls, explicit non-goals |
| [Configuration](configuration.md) | Operator policy keys and fail-closed defaults |
| [API reference](api-reference.md) | Tool input schema, intent fields, and report shape |
| [Getting started](getting-started.md) | Build, install, configure, and run the first review |
| [Operator guide](operator-guide.md) | Approve runs, inspect SOPs, advisory RPC enrichment |
| [Telegram agent](telegram-agent.md) | Channel behavior and request format |
| [Verification](verification.md) | Tests, CI, reproducibility, and validation evidence |

**Repository:** https://github.com/OutstandingVick/ogige  
**License:** MIT OR Apache-2.0
