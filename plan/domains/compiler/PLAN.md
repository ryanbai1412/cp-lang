# compiler plan

The detail behind [README.md](README.md). Numbers are from [metrics.toml](metrics.toml) (measured 2026-11-22, main `4b8f504f`).

## Where we are

Conformance 1617/1711 with 47 distinct causes; the top two (18 wrong outputs, 16 `E0302`) cluster in `core-04` (domains, keyed slices, products). The diff suite skips 94 cases, the same count conformance fails (a case is diffed only once it passes, inferred from the equal counts). Under the numbers, the architecture carries four debts that make every fix more expensive than it should be: two interpreter engines, std names matched in the compiler, edition branches, and 13k-line files ([ARCHITECTURE.md](ARCHITECTURE.md#known-debt)).

## Workstreams, in order

1. **Land the queue (M1).** `devin/wip-split-vm` first (pure move of `vm.rs` into `vm/`; everything else rebases on it), then `devin/wip-conformance-queue` (6 commits: `turn` in the prelude, `<<` in type bounds, `join` on a list view, `Prefix(a)` on a 2-D shape, top-level `let ... else` narrowing, domain values using domain methods), then `devin/wip-product-domain` once `p.step(d)` passes. Each: rebase, `scripts/check.sh --fast`, land, `git cherry` to delete superseded ones, worklog entry.
2. **Known bugs (M1).** `Seq.max()`/`min()` on a slice returns `?T` (`std-seq/slice-max-optional`, `diagnostics/max-of-a-slice-is-optional`); a fn that mints a domain mints a fresh one per call (`std-compress/wrapper-fn-fresh-domain`, `core-04/rebase-fresh-domain`).
3. **Products and keyed slices (M1 to M2).** One representation of a product position used by check, lowering and both back ends (Q1); then keyed-slice positions (`b1`/`b2`/`b5` wrong outputs) and the `E0302` cluster.
4. **The long tail (M2).** By cause, biggest first, re-running `--causes` after each landing: generic inference (`E0501` explicit type args, `E0204`), missing std methods that are compiler gaps (`E0201`/`E0203`; stdlib gaps go to the stdlib worklog), ownership (`E0608` in loops and branches), the 3 missing-diagnostic cases.
5. **One engine (M3).** Lower each fallback tag into the VM, most executed first (`CPL_VM_STATS=json` names them; `expr:Read` leads), until the fallback count is 0 on corpus and conformance; delete the tree-walker; enable the VM on wasm.
6. **No privilege, no editions, no giant files (M3).** Inline `is_v03()` branches (mechanical, 120 sites); move std-name matching into one intrinsics table; split `lower/call.rs` and `cpl-interp/src/lib.rs` along the seams the table and the VM leave. Each split is a pure move commit, then behaviour commits.

Contract tests run alongside all of it: each interface change adds or updates the test in its consumer's suite; first missing one is "parse round-trips every corpus file".

## Why each milestone

- **M1** removes merge risk (three branches touching `vm.rs`, `lower/call.rs`, `check/call.rs`) and the bugs Ryan already knows about. Cheap, unblocks everyone.
- **M2** is the v0.3 promise: the spec runs. It also closes the diff number, since every skipped diff case is a failing conformance case.
- **M3** is what keeps M2 true. With two engines every semantic fix is done twice and can disagree; with std names in the compiler, the stdlib can't change a type without a compiler change. Doing M3 after M2 is deliberate: conformance is the safety net for the refactors. Pieces of M3 that make an M2 fix cheaper (a VM tag the fix needs, one std name) are pulled forward.

## Alternatives considered

- **Refactor first, then conformance.** Cleaner code sooner, but refactors with 94 failing cases have a weaker net, and Ryan's v0.3 bar is behaviour. Rejected; refactor where it pays for the next fix.
- **Rewrite to the target query pipeline (`plan/ARCHITECTURE.md`) now.** The right end state, but a rewrite stalls the numbers for weeks. Rejected for v0.3; migrate by seams (facts pass, intrinsics table, one VM).
- **Keep the tree-walker as an oracle.** The C++ diff already is a second implementation; a third costs every feature twice. Recommend delete (Q2).
- **Fix std-name privilege by adding more names to `builtins.rs`.** Moves the problem; the goal says user libraries get the same powers. Table first, general mechanism second (tech-debt's builtin-floor question).
- **Product positions packed as one row-major `Int`.** Smallest storage, but neighbours cost a div/mod and `(r, c)` destructuring needs it everywhere. Recommend tuple (Q1).

## Risks

| Risk | Mitigation |
|---|---|
| A fix in check or lowering regresses another case | `cpl-conform all --check` in the fast gate; land one cause at a time |
| Branch conflicts after `wip-split-vm` moves `vm.rs` | land it first, rebase the others immediately |
| Failing cases are spec bugs, not compiler bugs | dispute in `plan/domains/spec/worklog.md` with the case; never edit the case |
| Long tail needs stdlib bodies (`E0201` on `std-graph`, `std-poly`) | triage per case; stdlib gaps go to the stdlib worklog, not into the compiler |
| VM work blocked by perf's C++ choices or the trace format | IR changes go through the interface table and consumers' contract tests |
| Two-core VM makes the full gate slow | `--fast` per commit; trains via `scripts/land.sh` |

## Phase 2, first actions

1. Rebase `devin/wip-split-vm` on main, `scripts/check.sh --fast`, `CPL_LANE=compiler scripts/land.sh`.
2. Rebase and land `devin/wip-conformance-queue` (6 commits, each gated), record the conformance delta.
3. Fix `p.step(d)` on `devin/wip-product-domain` (the remaining tuple-expecting path in check/lowering), add the regression, land.
4. `Seq.max()` on a slice as `?T`; fresh domains per `compress()` call. Then `--causes`, next biggest.
