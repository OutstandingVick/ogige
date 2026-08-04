# Bounty scorecard

This is a conservative self-score against the supplied 100-point rubric, not a
claim about judge results.

## Current repository, before live Telegram recording: 76/100

| Category | Score | Evidence / deduction |
|---|---:|---|
| Use case | 20/30 | Real model-driven ZeroClaw ALLOW and adversarial REJECT turns plus host execution, but no recorded Telegram run yet |
| Safety | 23/25 | Rust-enforced intent + policy, minimal WASM grant, narrow risk profile, external-only durable checkpoint, injection regression |
| Craft | 17/20 | Strict tests, compact output, source host E2E; offline decoder cannot resolve state/CPI/ALTs |
| Reproducibility | 13/15 | Pinned host/toolchain, config, skill, SOP, fixture generator, exact commands; live credentials necessarily absent |
| Showcase | 3/10 | Reproducible CLI evidence, draft post, and video script exist, but no final channel video or Discord post |

Eligibility remains pending until the real channel showcase is recorded and
posted. A technically strong repository alone is not a valid final submission.

## Expected after the planned live run: about 84/100

| Category | Score | Evidence needed |
|---|---:|---|
| Use case | 24/30 | Real Telegram ALLOW → durable checkpoint → human decision plus REJECT flow |
| Safety | 23/25 | Same enforced boundary, visibly demonstrated |
| Craft | 17/20 | Same implementation and host proof |
| Reproducibility | 13/15 | Another operator can follow the pinned runbook |
| Showcase | 7/10 | Clear sub-three-minute real-channel video and concise Discord write-up |

## Highest-value remaining upgrades beyond 84

1. Add a provenance-backed devnet/mainnet unsigned transaction produced by a
   standard Solana client, not only the deterministic synthetic fixture.
2. Show daemon audit evidence that the same bytes reach the checkpoint and that
   a REJECT creates no approval run.
3. Add optional RPC simulation/state enrichment in a separate permissioned
   mode, while keeping the default offline tool minimal.
4. Replace or validate the purpose-built wire primitives against upstream
   modular Solana crate test vectors and publish a broader fixture corpus.
