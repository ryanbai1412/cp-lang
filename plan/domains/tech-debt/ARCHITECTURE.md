# Architecture & tech-debt: the code as it is

Read from `main` at `4e2de3cd` (2026-11-22). Counts come from
`python3 plan/domains/tech-debt/measure.py` unless another command is named.
The cross-domain target design is [plan/ARCHITECTURE.md](../../ARCHITECTURE.md)
part A; this file is the measured present.

## Modules this lane owns

| Path | What it is for |
|---|---|
| `plan/ARCHITECTURE.md` | Cross-domain map: target layers (part A) and the migration (part B) |
| `scripts/debt-ratchet.py`, `scripts/debt-ratchet.json` | Gate ratchet (step in `scripts/check.sh`): files over 2,000 lines (allowlist), std TODO bodies, `#[ignore]`, pending tests |
| `scripts/test_debt_ratchet.py` | Unit tests for the ratchet, run by the fast gate |
| `plan/domains/tech-debt/measure.py` | Read-only counts behind [metrics.toml](metrics.toml); becomes ratchets in M2 |

## The crate graph (shipped `[dependencies]`, from each `crates/*/Cargo.toml`)

```
cpl-diag, cpl-json, cpl-judge, cpl-prover (dir crates/prover)    leaves
cpl-syntax  -> diag                         cpl-fmt  -> syntax diag
cpl-check   -> syntax diag                  cpl-doc  -> syntax diag json
cpl-ir      -> check syntax diag            cpl-lint -> check syntax diag prover
cpl-interp  -> ir diag json                 cpl-codegen -> ir diag
cpl-stress  -> check ir interp codegen judge syntax diag json
cpl-lsp     -> check ir fmt lint judge syntax diag json
cpl-contest -> check ir codegen lint stress judge syntax diag json
cpl-conform -> check ir interp codegen lint diag json      (spec conformance)
cpl-wasm    -> check ir interp codegen lsp stress judge diag json
cpl-cli     -> every crate except conform, wasm, prover
```

75 shipped edges. Back ends (`cpl-interp`, `cpl-codegen`) depend only on the
IR in shipped code, which is the intended layering; their tests use the
checker (`dev_crate_edges`: `codegen -> check`, `interp -> check`, and
`wasm -> cli` from `crates/cpl-wasm` tests). The one shipped violation is
`cpl-lint -> cpl-prover`: lints run their own prover queries instead of
reading facts the checker computed.

Source size by crate (`cat crates/<c>/src/**/*.rs | wc -l`): cpl-ir 40,872;
cpl-interp 28,052; cpl-check 25,671; cpl-codegen 16,280; cpl-lint 14,702;
cpl-syntax 11,178; cpl-stress 10,561; prover 7,313; cpl-cli 7,226; cpl-lsp
7,050; the other eight under 4,000 each.

## Interfaces other lanes use

| Interface | Where | Consumers | This lane's job |
|---|---|---|---|
| Crate edges | `crates/*/Cargo.toml` | every lane | Edge list frozen; a new edge lands with a `plan/ARCHITECTURE.md` change (M2) |
| Debt ratchet | `scripts/debt-ratchet.py --check`, baseline `.json` | gate (infra), every lane's commits | Counts only go down; a bump needs a `decisions.md` line |
| Shared core | [compiler/ARCHITECTURE.md](../compiler/ARCHITECTURE.md) | lint-fmt, tools, diagnostics, perf, debugger, stdlib | Review every interface change against all consumers |
| Cold-review findings | each lane's `worklog.md` | owning lane | Findings come before that lane's feature work |

## Invariants

- Back ends read the IR, never the checker or syntax (holds in shipped code today).
- No Rust file over 2,000 lines unless allowlisted in `scripts/debt-ratchet.json`; the allowlist only shrinks.
- `#[ignore]`, std TODO bodies and pending tests never increase (gate step "debt ratchet").
- A migration deletes what it replaces (`plan/ARCHITECTURE.md` B2).

## Known debt

Ranked; the top 10 is the same list as the [README](README.md#top-10-architectural-debt-today-ranked-detail-in-architecturemd).

1. **Stdlib privilege in lowering** (compiler + stdlib). `crates/cpl-ir/src/lower/call.rs` (13,357 lines) string-matches 89 distinct names in match arms (`measure.py --list ir_builtin_names`): library shapes such as `prefix`, `first_ge`, `adjacency`, `counts`, `runs`, `choose2`, `laws_hold`, `AddAction`, `HashMap`, beside true primitives (`print`, `read`, `popcount`). They lower to `enum Builtin` in `crates/cpl-ir/src/lib.rs:674` (74 variants). A user library cannot get the same treatment, contrary to "no privileged stdlib".
2. **Two interpreter engines** (compiler). The tree-walker `fn eval` in `crates/cpl-interp/src/lib.rs` (5,615 lines) still runs, selectable with `CPL_TRACE_ENGINE=tree`, and the VM in `crates/cpl-interp/src/vm.rs` (15,367 lines) falls back to it for IR it has not lowered (comment at `vm.rs:1445`). Every interpreter feature is two implementations.
3. **Dead 0.2 edition switch** (compiler). 92 references to `is_v03`/`Edition::V02`/`CPL_EDITION` in 12 files; 32 in `crates/cpl-check/src/check/expr.rs`, 14 in `check/call.rs`, the switch in `crates/cpl-check/src/edition.rs`. Removal is written on branch `devin/drop-edition` (21 unlanded commits).
4. **Two bound analyses** (lint-fmt + compiler). `crates/cpl-lint/src/rules/bounds.rs` (1,089 lines) calls `cpl-prover` directly while the checker and codegen (`crates/cpl-codegen/src/ranges.rs`) derive ranges separately; the same fact is computed twice and can disagree.
5. **Monolith files** (owners vary; this lane does pure moves). 24 files over 2,000 lines (`measure.py --list rs_over_cap`): `cpl-interp/src/vm.rs` 15,367, `cpl-ir/src/lower/call.rs` 13,357, `lower/stmt.rs` 6,470, `lower/mod.rs` 6,123, `lower/expr.rs` 6,107, `cpl-interp/src/lib.rs` 5,615, `lower/mono.rs` 4,570, `cpl-lint/src/rules/taste.rs` 4,493, `cpl-check/src/check/expr.rs` 4,323. The `vm.rs` split is on `devin/wip-split-vm` (1 commit).
6. **Library type names in IR and emitter** (compiler + perf). 17 sites name std types that are not core intrinsics (`--list backend_std_names`): `HashMap` in `lower/call.rs:660`, `lower/stmt.rs:892`, `lower/mono.rs:3020`; `Prefix` in `lower/mod.rs:24`; `Bitset` in `cpl-codegen/src/expr.rs:2780` and `scope.rs:325`.
7. **Std member catalog kept twice** (compiler + tools). `crates/cpl-check/src/builtins.rs` (321 lines) lists receiver members for checking, hover and docs, while lowering matches the same names in `lower/call.rs`; the cross-check test `crates/cpl-cli/tests/builtin_catalog.rs:32` is `#[ignore]` for 9 stale v0.2 rows.
8. **Browser lints compiled out** (lint-fmt). Checker-backed lints sit behind `#[cfg(not(target_arch = "wasm32"))]` (`crates/cpl-lint/src/lib.rs:954`, `rules/bounds.rs:680`, `:1058`), so the playground and the CLI report different lints for one program.
9. **Repository leftovers** (this lane, with stdlib and idioms). The library implementation lives at `spec/std/` (40 files, 151 references to the path); 877 v0.2 `corpus/problems/*/port.cp` files remain; 57 of 68 `origin/devin/*` branches hold commits not on `main`; 13 direct `python3` calls in the gate and `justfile` bypass `uv`.
10. **Copied helpers** (tools, lint-fmt, compiler). `measure.py --list dup_fn_bodies`: `collect` (19 lines) in `crates/cpl-fmt/src/cli.rs:133` and `crates/cpl-lint/src/cli.rs:229` copies `crates/cpl-doc/src/cli.rs:163`; `refutable` in `cpl-ir/src/lower/stmt.rs:6429` copies `cpl-check/src/check/stmt.rs:3159`; `prefix_shape_struct` in `cpl-ir/src/lower/mod.rs:22` copies `cpl-check/src/check.rs:1036`; `block_hash` twice in `crates/cpl-conform`.

Also: 19 `#[ignore]` tests; `plan/ARCHITECTURE.md` part B1 describes `stdlib/` and other pre-restructure paths that no longer match the tree; `research/` holds 1,367 tracked files.
