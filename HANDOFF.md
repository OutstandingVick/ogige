# HANDOFF — ogige (ZeroClaw Solana Guard Plugin)

**Date:** 2026-08-01  
**Repo:** https://github.com/OutstandingVick/ogige  
**Local path:** `/Users/macbook/macbook/ogige`  
**Status:** Scaffold complete, host tests green, `wasm32-wasip2` release build succeeds, pushed to GitHub `main`.  
**Author of this work:** prior Cursor agent (Grok) — session ending due to user monthly usage limit.

This document is the single source of truth for continuing the Superteam Brasil × ZeroClaw bounty submission.

---

## 1. Mission

Win (or place highly in) the bounty:

**[Build Solana-native plugins for Zeroclaw 🦞](https://superteam.fun/earn/listing/zeroclaw)**  
- Sponsor: Superteam Brasil  
- Prize pool: 5,000 USDG (1st 1,800 / 2nd 1,200 / 3rd 1,000 / 4×250 bonus)  
- Winner announcement: **August 21, 2026**  
- Skills: Backend + Blockchain  
- Contact called out in research: https://x.com/kauenet  

Deliverable: one or more **ZeroClaw WIT tool plugins** (`wasm32-wasip2` components) that give agents real Solana capability, submitted as a PR to  
https://github.com/zeroclaw-labs/zeroclaw-plugins  
(draft PRs + Discord engagement encouraged).

---

## 2. Product decision (what we are building)

### Chosen direction: **tight-scope safety gate, not a full Solana SDK**

Project/repo name: **`ogige`**  
Plugin name (manifest): **`ogige`**  
LLM tool name: **`solana_guard`**

**Job:** Agent passes a base64 Solana transaction → plugin returns:
1. Human-readable **narration**
2. Structured **risk findings**
3. Fail-closed verdict: **`ALLOW` / `HOLD` / `REJECT`**

**Never signs. Never broadcasts.** Custody model T0/T1 only.

### Strategy that was debated and then narrowed

Earlier research docs recommended:
- `solana-core` (full WASM-friendly Solana substrate — Track E) + `solana-guard`
- Or a suite: guard + stream + repro

**Prior agent recommendation that the user accepted:**

| Do | Don’t |
|---|---|
| One deeply polished plugin | Premature multi-plugin suite |
| Internal `core/` modules (extractable later) | Full tx construction / send / confirm SDK |
| Decode + narrate + verdict as the wow demo | Reinvent all of `zeroclaw-solana-core` |
| Heavy exploit-style fixtures | Breadth-over-depth |

Rationale: `solana-sdk` / `solana-client` do **not** compile cleanly for `wasm32-wasip2`. Building a full substrate is months of work (see Palinurus / Track C competitors). For a guard you only need **read/decode** primitives. There is already a crates.io crate [`zeroclaw-solana-core`](https://crates.io/crates/zeroclaw-solana-core) (read-focused RPC + account decode) — differentiate by **transaction decode + safety analysis**, optionally depend on that crate later for RPC enrichment rather than duplicating it.

---

## 3. ZeroClaw technical contract (must not violate)

Canonical reference plugin: `plugins/redact-text` in `zeroclaw-labs/zeroclaw-plugins`.

Hard requirements for judging:
- Layout matching reference: pure Rust core + thin `#[cfg(target_family = "wasm")]` shim
- Crate type: `["cdylib", "rlib"]`
- Host-runnable tests: plain `cargo test` (no WASM needed for core)
- Structured logging via WIT `log-record` (no raw stdout in the component)
- `manifest.toml`: kebab-case name, version, `wasm_path`, `capabilities`, minimal `permissions`
- Target: `wasm32-wasip2`
- WIT world: `tool-plugin` from vendored `wit/v0`
- Bindgen: `wit-bindgen` `0.46` with `features: ["plugins-wit-v0"]`

### WIT surface (tool plugin)

Exports:
- `plugin-info`: `plugin-name`, `plugin-version`
- `tool`: `name`, `description`, `parameters-schema`, `execute(args) -> result<tool-result, string>`

Imports:
- `logging.log-record`

`tool-result`: `{ success: bool, output: string, error: option<string> }`

Host injects jailed config as `__config` flat `string -> string` map in execute args when `config_read` permission is declared. **ogige currently declares `permissions = []`** and still works with defaults via empty map.

Docs:
- Plugin protocol: https://github.com/zeroclaw-labs/zeroclaw/blob/master/docs/book/src/developing/plugin-protocol.md
- Registry README: https://github.com/zeroclaw-labs/zeroclaw-plugins

---

## 4. What exists in the repo today

### Layout

```
ogige/
  Cargo.toml          # cdylib+rlib, wit-bindgen, serde; [workspace] standalone
  Cargo.lock          # committed
  manifest.toml       # name=ogige, wasm_path=ogige.wasm, capabilities=["tool"], permissions=[]
  README.md
  HANDOFF.md          # this file
  .gitignore
  src/
    lib.rs            # WASM shim (tool name solana_guard) + mod core/guard
    guard.rs          # analyze() → GuardReport (ALLOW/HOLD/REJECT)
    core/
      mod.rs
      base58.rs       # hand-rolled; fixed leading-zero / system-program roundtrip
      base64.rs       # hand-rolled encode/decode
      pubkey.rs
      programs.rs     # well-known program IDs + labels
      tx.rs           # legacy + v0 wire decode (compact-u16, ATTs)
      narrate.rs      # System / SPL / Token-2022 / BPF loader / compute budget
      risk.rs         # findings + severity taxonomy
  tests/guard.rs      # host integration fixtures (built txs)
  wit/v0/             # vendored from zeroclaw-plugins (tool, logging, types, plugin-info)
  fixtures/           # empty placeholder dir (not tracked)
```

### Public API (host-testable)

```rust
ogige::guard::analyze(transaction_base64: &str, &GuardConfig) -> Result<GuardReport, String>
ogige::guard::report_json(&GuardReport) -> String
```

`GuardReport` fields: `verdict`, `summary`, `narration`, `findings`, `tx_version`, `instruction_count`, `account_count`.

Config keys (via `__config` later):
- `reject_on_critical` (default true)
- `hold_on_high` (default true)
- `hold_on_medium` (default false)

### Risk codes already implemented (v0.1)

| Code | Severity |
|---|---|
| `SYSTEM_ASSIGN` | CRITICAL |
| `TOKEN_APPROVE_MAX` | CRITICAL |
| `MINT_AUTHORITY_CHANGE` / `FREEZE_AUTHORITY_CHANGE` / `TOKEN_OWNER_CHANGE` | CRITICAL |
| `PROGRAM_UPGRADE` / `UPGRADE_AUTHORITY_CHANGE` | CRITICAL |
| `NONCE_AUTHORIZE` | HIGH |
| `TOKEN_APPROVE` / `TOKEN_MINT_TO` | HIGH |
| `TOKEN_CLOSE_ACCOUNT` / `ALT_USED` / `UNKNOWN_PROGRAM` | MEDIUM |
| `SOL_TRANSFER` / `TOKEN_TRANSFER` | LOW |

### Verification already done

```bash
cd /Users/macbook/macbook/ogige
cargo test                          # 4 unit + 8 integration = all green
CARGO_TARGET_DIR=./target cargo build --target wasm32-wasip2 --release
# → target/wasm32-wasip2/release/ogige.wasm (~209KB)
```

Note: Cursor sandbox may redirect `CARGO_TARGET_DIR`; prefer explicit `./target` for local artifacts. `wasm32-wasip2` target is installed on this machine via rustup.

### Git / GitHub

- Remote: `origin` → `https://github.com/OutstandingVick/ogige.git`
- Branch: `main` tracking `origin/main`
- Initial commit: `d6b7740` — “Initial ogige: ZeroClaw Solana transaction safety gate.”
- GitHub account used: **OutstandingVick** (`gh` scopes: gist, read:org, repo)
- This handoff commit may land after that.

---

## 5. Conversation history summary (what the user brought)

### Discussion 1 — bounty overview + winning plan
User pasted a long summary covering:
- ZeroClaw as Rust agent runtime, WASM plugins, deny-by-default permissions
- Recommended `solana-guard` as flagship
- Hybrid suite idea (guard + stream + repro)
- Phased plan: fork plugins repo → draft PR → Discord

### Discussion 2 — “can’t-do-without” infrastructure take
Research pointing at Track E shared substrate + safety rails:
- WASM-friendly Solana core is foundational
- Agents lack spend gates / fail-closed behavior
- Proposed `solana-core` + `solana-guard`

### Agent pushback (accepted)
- Don’t build full `solana-core` SDK first
- One plugin, absurd test quality, great demo
- Watch differentiation vs existing `zeroclaw-solana-core` on crates.io

### User decisions
1. “yess” → start scaffolding  
2. Project name **`ogige`**, location **`~/macbook/ogige`** (resolves to `/Users/macbook/macbook/ogige`)  
3. Create GitHub repo, commit, push → done  
4. Stop + write this handoff (usage limit)

---

## 6. What is NOT done yet (priority order for next agent)

### P0 — submission readiness
1. **Open draft PR** to https://github.com/zeroclaw-labs/zeroclaw-plugins  
   - Copy/adapt this crate into `plugins/ogige/` (or `plugins/solana-guard/` — decide naming with user; repo is branded ogige, descriptive kebab name may score better for registry discoverability)  
   - Match their CI: `python3 tools/build-registry.py --check-metadata`, locked builds, etc.  
2. **Join / engage ZeroClaw Discord** for maintainer feedback early  
3. **Demo script + ≤3 min video**: agent proposes dangerous tx → guard REJECT with narration  
4. **Superteam Earn submission** before winner announcement window (Aug 21, 2026)

### P1 — differentiation / depth
5. Real **mainnet exploit / rug fixtures** as base64 corpus under `fixtures/` (authority hijacks, unlimited approve, upgrade authority grabs)  
6. **Token-2022** deeper analysis (transfer hooks, permanent delegate, freeze, non-transferable)  
7. Optional **`http_client` permission** + RPC enrichment:
   - `simulateTransaction` + balance deltas  
   - mint authority / freeze authority account lookups  
   - Consider depending on `zeroclaw-solana-core` instead of rewriting RPC  
8. Spend caps / mint allowlists via config  
9. Squads / nested CPI narration if feasible offline

### P2 — polish
10. README agent workflow examples with ZeroClaw `config.toml`  
11. Quantitative coverage reporting  
12. Self-audit of permission surface (keep minimal)  
13. Extract `core/` to a publishable crate only if it helps the bounty story — not required

---

## 7. How to continue in a new Cursor session

```bash
# Option A: open existing local clone
cd /Users/macbook/macbook/ogige

# Option B: fresh clone
git clone https://github.com/OutstandingVick/ogige.git
cd ogige

rustup target add wasm32-wasip2
cargo test
CARGO_TARGET_DIR=./target cargo build --target wasm32-wasip2 --release
```

**Workspace note:** User’s Cursor home-workspace rule says: for project work, call `cursor-app-control` MCP `move_agent_to_root` to `/Users/macbook/macbook/ogige` (or the clone path) **before** making edits.

Suggested first prompt for next model:

> Read `HANDOFF.md` and `README.md` in ogige. Continue P0: prepare a draft PR to zeroclaw-labs/zeroclaw-plugins for the ogige Solana guard plugin, then add real exploit fixtures and improve Token-2022 coverage.

---

## 8. Key external references

| Resource | URL |
|---|---|
| Bounty listing | https://superteam.fun/earn/listing/zeroclaw |
| ZeroClaw runtime | https://github.com/zeroclaw-labs/zeroclaw |
| Plugin registry (submit here) | https://github.com/zeroclaw-labs/zeroclaw-plugins |
| Reference plugin | `plugins/redact-text` in that registry |
| Existing WASM Solana core crate | https://crates.io/crates/zeroclaw-solana-core |
| Example competitor (DePIN / Track C) | https://github.com/RECTOR-LABS/palinurus |
| This project | https://github.com/OutstandingVick/ogige |

---

## 9. Design constraints / pitfalls already hit

1. **Base58 leading zeros:** naive encode/decode produced 33 `1`s for system program; fixed to match Solana’s 32-`1` encoding. Keep that fix.  
2. **No `solana-sdk` in WASM:** stick to hand-rolled or proven WASM-friendly crates only.  
3. **WIT vendoring:** `wit/v0` must stay compatible with host; re-sync from zeroclaw-plugins if upstream bumps.  
4. **Permissions:** don’t request `http_client` until RPC path exists and is tested; deny-by-default is a judging theme.  
5. **Cargo target dir under Cursor sandbox:** set `CARGO_TARGET_DIR=./target` when you need the `.wasm` on disk in-repo (gitignore keeps `*.wasm` / `target/` out of git — correct).  
6. **`gh` auth:** worked for OutstandingVick with full permissions; if `gh` says keyring token invalid, retry outside sandbox / `gh auth refresh`.

---

## 10. Suggested demo narrative (for judges)

1. Agent wants to “approve unlimited USDC spending for this DEX helper.”  
2. Calls `solana_guard` with the crafted base64 tx.  
3. Output: narration (“Approve MAX (unlimited) delegate → …”) + `TOKEN_APPROVE_MAX` CRITICAL + **`REJECT`**.  
4. Contrast with a simple 1 SOL transfer → **`ALLOW`** with LOW `SOL_TRANSFER` note.  
5. Emphasize: fail-closed, host-tested, sandbox-safe, no signing keys in plugin.

---

## 11. Open questions for the user (do not assume)

- Prefer registry folder name `ogige` vs `solana-guard`?  
- Public Discord handle / Superteam Earn account ready?  
- Any prior Solmite / stream-skill / airdrop code to reuse for fixtures?  
- Target demo chain: mainnet fixtures only, or also devnet live simulate?

---

**Bottom line for next model:** The scaffold is real and green. Do not restart architecture. Next value is **submission packaging (draft PR) + fixture depth + demo**. Time box toward August 21, 2026 winner announcement.
