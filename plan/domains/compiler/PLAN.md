# compiler plan

The detail behind [README.md](README.md). Numbers are from [metrics.toml](metrics.toml) (measured 2026-11-22, main `4b8f504f`).

## Where we are

Conformance 1617/1711 with 47 distinct causes; the top two (18 wrong outputs, 16 `E0302`) cluster in `core-04` (domains, keyed slices, products). The diff suite skips 94 cases, the same count conformance fails (a case is diffed only once it passes, inferred from the equal counts). Under the numbers, the architecture carries four debts that make every fix more expensive than it should be: two interpreter engines, std names matched in the compiler, edition branches, and 13k-line files ([ARCHITECTURE.md](ARCHITECTURE.md#known-debt)).

## Workstreams, in order

Ordered by [Milestone 1](../../product/MILESTONE-1.md) (deadline about 09:30 PST 2026-11-23): what blocks the friend's solve/test/submit/compete steps comes first, the metric milestones below after. Done since the packet: `wip-split-vm` and `wip-conformance-queue` landed; `Seq.max()` on a slice is `?T` and the `compress()` wrapper case passes; the two P0 wrong answers (`**` on an `Int` parameter, a comprehension's stale range bound) landed with diff regressions.

1. **Interpreter == C++ (Milestone 1).** `cpl-conform diff` to 0 mismatches: `corpus/dmopc21c5p2` first. Each fix adds a `crates/cpl-cli/tests/programs/` pair that both back ends run.
2. **No silent wrong answers (Milestone 1).** The 11 `wrong output` causes in `--causes`, `corpus/aac5p4`. A construct that can't be made right in the time gets a named error instead (spec R-rule: compile-time error, then trap, never wrong).
3. **Gaps named (Milestone 1).** Internal traps (`etsr23r4p2`: `expected closure`) become errors naming the unsupported construct; the corpus `error` cases split by owner, compiler causes fixed biggest first; the `--causes` list goes to spec for `spec/STATUS.md`.
4. **`cpl run` and stress speed (Milestone 1, only where slow).** Time every runnable corpus sample; VM work only for one over 1 s.
5. **Products and keyed slices (M2).** `wip-product-domain` (tuple positions, Q1 default) and the `E0302` cluster (29 cases, top cause).
6. **The long tail (M2).** By cause: `matrix/index` methods (`E0203`, 21 cases), generic inference (`E0501`), lends (`E0603`), `Gen` for `core-06/gen-*` with stdlib.
7. **Deferred, in parallel when nothing above is open.** Interpreter snapshot/restore for trace replay (time-travel can wait; the shared value-register stack is parked locally), one engine (M3), std-name table, edition branches, file splits.

Contract tests run alongside all of it: each interface change adds or updates the test in its consumer's suite.

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

## Next actions

1. `dmopc21c5p2`: bisect the first differing value between `cpl run` and the C++, fix, regression.
2. `--causes` `wrong output`, one cause per commit.
3. `etsr23r4p2` internal trap to a named error.
