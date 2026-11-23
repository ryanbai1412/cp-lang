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

## General fixes owed
- Slice `max()` typed separately in checker (`catalog_ret`) and IR (`ListView` arm). General fix: IR takes a call's type from the checker instead of re-deriving it; and spec R07.9.3 says every `Seq.min/max` is `?T`, while List/Str rows still give the trapping `T` (decision at decisions.md ~3913). Align the rows with spec once corpus is migrated. Owner: compiler.
- Member visibility (R11.1.4): E0207 on `p.y`/`p.secret()` is decided by `FileCtx::member_private` (cpl-check db/fields.rs) from the declaring module's AST. Still owed: a struct literal or positional constructor naming a private field outside its module, a private field in a pattern, and one per-member `public` bit on `Def::Struct` that IR, hover and completion read instead of re-walking the AST.
- LSP call hierarchy resolves calls by name in cpl-lsp. General fix: the checker records each fn's resolved callees, one call graph for callHierarchy, dead-code lint and explain. Owner: compiler.
- `cpl explain` whole-program peak memory, specializations and remaining checks need IR data. General fix: an IR-level explain query beside `Database::representation`. Owner: compiler.

Each local fix whose general mechanism is still owed: the instance, the general fix, the owner.

- **Retired spellings** (`some`/`none`, 2026-11-23, landed by diagnostics in 835de99a1 as `old_optional_spelling`): the check is per-name, and the parser still canonicalizes `Some` to an internal `some` that the IR and prover special-case. General fix: one table of renamed spellings (old, new, decision) that every name, literal and pattern goes through, so a future spec rename is one row and never a silent alias; syntax keeps the source spelling and IR names the variant `Some`. Owner compiler.

- Range `sum` closed form (row 1 of `plan/domains/tech-debt/std-intercepts.md`: `try_range_int_sum` in `cpl-ir/src/lower/call.rs`, `cpl::range_sum` in `cpl-codegen/src/runtime.rs`, `closed_form::range_sum` in `cpl-interp`): both copies compute in 128 bits and trap past an `Int` (untrapped C++ wraps through unsigned). General: `sum` written in std, in CPL, with checked `Int` arithmetic, found by method lookup ahead of the generic `Seq.sum`; then delete `RangeSum`. Missing first: (1) `(lo..hi).step(k)` lowers to a materialized `List<Int>`, so std needs a stepped-range `Seq` type; (2) a type's own method before a same-name extension (spec is writing it; `fn` overloading is E0702 today); (3) std methods on the core `Range`. Owner: spec for (2), then compiler.
- The other intercepts (rows 2-18 of `std-intercepts.md`): each row's general fix column is the move-to-std plan; delete a row when it lands. Owner: compiler, with stdlib where the row says.
- `lo..=hi` lowers to `lo..hi + 1` in C++, so `(0..=big).step(big)` at `Int` max is signed overflow in the generated C++ while the interpreter traps. General: inclusive ranges carry `hi` and test `x <= hi`, in one lowering. Owner: compiler.

- An array over an anonymous range (`[0..m]R`) has no domain of its own; each generic call that binds `E` from it gets one by call site or by fn (`bind_range_domain`), and a domain shared by fn can escape through a return type that names it only indirectly (`Graph<V, B>` with `B` holding `e: E`). General fix: an anonymous range in an array type is an anonymous domain at its declaration, as `Dsu<0..3>` already is (decision 255), so `E` binds to it like a named one. Owner: compiler.

- Field-trait satisfaction (R08.1.7, decision 361) now agrees in checker and IR and checks a field's type when the trait is not generic, but it is still written twice (`Checker::meets_by_shape`, IR `has_bound_by_shape`), and a generic trait's field type (`Heavy<W>: w: W`) is not matched against its arguments. General fix: one relation in cpl-check, `satisfies(ty, trait, args)`, that IR asks through the checked program instead of re-deriving it. Owner: compiler.
- A slice of a `[lo..=hi]T` (`Dim::IntFrom`) still copies into a `List`; `[0..n]T` and `[D]T` are key views. General fix: slices of every one-axis array are key views (needs `ListView` keyed from `lo`). Owner: compiler.
## Next actions

1. `dmopc21c5p2`: bisect the first differing value between `cpl run` and the C++, fix, regression.
2. `--causes` `wrong output`, one cause per commit.
3. `etsr23r4p2` internal trap to a named error.
