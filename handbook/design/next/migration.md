# Migration

**Status:** 2026-11-25. Stage 1 has landed; stage 5's pipeline is
growing in `crates/cpl-next`. Ryan approved the order, and made two
calls on it: the lossless syntax tree is kept, and an old path that
runs beside a new one is debt that never gets deleted, so the plan may
not rely on one. Current numbers are in `plan/END-STATE.md`, not here.

## Measures

Three counts, each a gate that never goes down. Deleting the old
pipeline needs all three complete, plus a passing case for every
diagnostic code.

| Count | What it measures | Today |
|---|---|---|
| Conformance programs equal to their expected result | the language (4,447 programs, 232 syntax features) | not measured on the current slice; 196 of 332 accepted on the reviewed one |
| Corpus programs equal to expected output | what users write (1,570 programs, 171 features) | 30, each on 3 inputs |
| Std declarations compiled from unmodified `std/*.cp` | the library (43 modules, 56,059 lines) | 0 |

"Accepted" always means equal to the expected result, never only
"not refused".

## Rules

1. **A program goes through the new pipeline only when all of it is
   supported.** The new pipeline refuses by a closed list: every
   syntax form, field and decorator it has not implemented is a
   refusal, by an exhaustive match with no catch-all.
2. **The oracle is the spec.** Every test carries its expected result
   and the spec rule it comes from. Today's conformance suite
   describes today's compiler (it passes 4,199 of 4,213 cases), so
   where the design decides differently the case changes first, with
   the decision cited, and the code second.
3. **New code is in the workspace and the commit gate from its first
   commit.** The slice outside the repo stopped building against main
   twice.
4. **Every measure records the commit it was taken at** and is
   produced by a script in the repo. Three of five round-5 measures
   did not reproduce.

### No old path survives

The repository already has this debt: the tree-walker was to be
deleted for the VM and both remain. These rules are the guard.

5. **The old path is never a fallback.** No flag, no "if the new one
   refuses, use the old one". A shipped tool runs a program one way.
   The `cpl` binary cannot link both; a dependency rule fails the
   build if it does.
6. **Old code is compared against only inside the test harness.**
7. **Switch and delete are one landing.** A stage is not done when
   the new code passes; it is done when the old code it replaces is
   gone. Each stage's exit includes "lines of the replaced code
   remaining: 0".
8. **Each window has a forecast end, re-forecast at every audit.** A
   missed forecast never reverts the new work and never extends the old
   code's life: the old code stays frozen (rule 9) and the switch and
   its deletion stay one landing (rule 7). Each two-hourly audit
   re-forecasts the end from the measured burn-down and the remaining
   refactors (Ryan, 2026-11-27: the end state matters, not the
   intermediate dates).
9. **Old code waiting for deletion is frozen**: no fixes, no
   features. A fix goes to a conformance case, which binds the new
   code. The gate checks each commit, not only the line count: a
   commit that adds a line to the old crates fails, `frozen_edits` in
   `scripts/debt-ratchet.py`, and no trailer excuses it. The one
   commit that may is one putting each old-crate file it touches back
   as it was at `c7ac8761d^`, the commit the old pipeline's std is
   pinned to. The old pipeline reads its own std,
   `crates/cpl-check/frozen/std/`: a copy of `std/` at `c7ac8761d^`,
   counted in `old_pipeline_lines` and held byte-identical to that
   commit by `frozen_std_drift`, so a std change never reaches the
   old code. Nothing in cpl-next reads it; stage 5 deletes it with
   the old crates.
10. **Prefer no window.** A stage that can change today's code in
    place does, so there is never a second copy.
11. **A designed component is built in its full shape before anything
    consumes it.** No consumer is written against a stand-in for it (the
    old syntax tree, a whole-program pass, a name match), because every
    such consumer becomes a migration later.
12. **A new kind of debt gets a hard-zero gate when the code that could
    add it starts**, not when an audit finds it.
13. **Reuse of old code carries its deletion step in the same plan.**
    Old code reused as scaffolding names the landing that deletes it
    and the counter that reaches 0 there.
14. **The switch lands only at zero debt.** Stage 5's switching landing
    requires every gated `next_*` counter in `scripts/debt-ratchet.py`
    at 0 (`--zero` over them), so no shortcut survives into the end
    state as "fix it after".

## The joint landing

lane-j-check (the syntax tree, decls and check on it, and the driver) lands on `main` in
slices, not as one batch. Each slice builds alone on the `main` it lands on, deletes the
old-tree reads it replaces, and passes `scripts/land.sh`'s full gate. The seams were found by
overlaying the branch's crates on `main` and building the workspace:

| Slice | What lands | Builds alone on `main` |
|---|---|---|
| 0 | `cpl-next-syntax` (lossless tree, operator kinds, interpolation nodes, typed views), `cpl-next-db`'s item queries, `cpl-next-ir`'s ids and types, `cpl-diag` | 2 errors, both in `main`'s `cpl-next-decls/src/modules.rs` (`ItemId.file` and one call's arity), fitted in the slice |
| 1 | decls on forms and the next tree (the `Mark` readers and `cpl-syntax` gone from decls), check on the next tree, the `Cx` views core and its views (Shapes, RoleRows), and `cpl-syntax`'s changes | yes, as one slice. Decls alone breaks `main`'s check in 158 places across 18 files, and fitting the old check to the new decls would be a second copy (rule: no old path survives) |
| 2 | the driver on one tree (the `Front.main` old `Stmt` and `FnBody` reads deleted), lower, link, the tools' edits, and the driver's `cpl-syntax` dependency | to be probed on slice 1 |

`std/` and `spec/` commits land on `main` on their own, ahead of the slices.

The landing gate. Each item gives its value on lane-j-check. A slice lands only when every item
it owns is met:

| Item | Target | Now |
|---|---|---|
| `next_syntax_old_refs` | 0 | 0 |
| `next_old_syntax_analyses` | 0 | 0 |
| `next_whole_program_inputs` | at most 602 | 748 |
| old `tyconv` over `ast::Type` | deleted | deleted |
| `cpl-syntax` in `cpl-next-check/Cargo.toml` | gone | gone |
| G's `Lane-for` trailers | every commit | waiting on the confirmed table |
| commit tier | green | red: the ratchet (case counts, `next_check_name_keys` 3, `unwrap_input` 4, `allow_attrs` 25, `dream_unverified` 18, `decls.rs` near its cap) and check's clippy |

## Order

| Stage | Work | Second copy? | Exit |
|---|---|---|---|
| 1 | Measures and the slice into the repo and the gate | no | the three counts and the spec-verdict harness run in the gate at HEAD |
| 2 | Run and check half of the facade, over today's compiler | no | direct compiler dependencies in tool crates for run and check: 0 |
| 3 | The new pipeline's declarations and checker accept every program | no: `cpl-check` is not changed, and is deleted with `cpl-ir` in stage 5's switching landing | conformance, std and corpus cases refused by `crates/cpl-next`'s front half: 0 (its refusals by cause are `target/cpl-next/queue.txt`); library declarations it knows by name (`next_std_names`): 0; on files with injected errors every unaffected unit still types and lowers (contracts 3 and 4: total), and every diagnostic carries a source id, not a line string; an edit rechecks only the units it changes, p95 under the end-state editor bar on 5,000-line files (END-STATE #20) |
| 4 | Storage operations in the language, then `List` in CP Lang | no | `List` builtins in the compiler: 0; the 1.1x time and memory bars hold on the container benchmarks |
| 5 | New back half: generic IR, link, reference interpreter, C++ emitter | **yes, the one window** | all three counts at 100%; today's lowering, tree-walker and C++ emitter deleted in the switching landing; the C++ size (END-STATE #15) and memory (#13) bars hold on the factored optimized program; `cpl run`, trace, the debugger, the browser and stress run on the bytecode engine, equal to the reference interpreter on all three counts (see the engine gap below) |
| 6 | Remaining containers, stress, rest of the facade | no | compiler-privileged std behaviour: 0; `cpl-stress` lowering entry points used: 0 (today 7) |
| 7 | Bytecode engine, events, debugger, machine code | per engine, switch and delete together | today's VM deleted; every engine equal to the reference interpreter on all three counts |
| 8 | Lossless syntax tree, then lints and editor onto it | parser switch and delete together | lint matches on today's tree: 0 (today about 940); today's parser deleted |

**The stage 5 window.** The new back half cannot replace today's in
one commit; it grows until it accepts every program. While it grows:

- it is reachable from the test harness only (rule 5);
- its accepted counts are published and may only rise; the gate
  fails on a drop;
- its end date is set from the measured rate of its first week:
  **2026-11-27, end of day**, a pace target, not a quality trade (Ryan, 2026-11-26: the end state is what counts; first set to 2026-12-06 from the measured rate);
- today's checker, lowering, interpreter and emitter are frozen by
  the `old_pipeline_lines` ratchet, which may only fall (rule 9);
- the tree-walker's removal is part of its exit, not a later cleanup.

**The switching landing's checklist.** Each item is a std or tool line
kept only because the frozen old pipeline still reads it; the switch
does not land while one is unchanged, and an item is added in the
commit that keeps one.

- [ ] `std/seq.cp` `write_seq` (the `Show` of `Rev`, `Slice`, `Filtered`, `Mapped`,
  `Rebased`, `Row`, `Col` and `Rect`) writes elements with `write_to`, not `write_fmt`:
  importing `std.show.{Fmt}` breaks the old `cpl`. Switch it to `write_fmt` and add a
  case printing a view of `F64` with `precision:` and of a counted domain under `io.base`.
- [ ] `cpl-syntax` depends on `cpl-next-syntax` for the shared lexer,
  `Token`, `split_items`, `literal` and `doc` only, nothing parser-level
  (S20). The edge goes with `cpl-syntax`, and so does its copy of the
  contextual words, `crates/cpl-syntax/src/words.rs`.
- [ ] `std/mod.cp` `impl Read for Mod<P>` keeps `fn read()`: the old
  `cpl` has no `Input`, so `read<Mod<P>>(from: s)` reads standard
  input. It becomes `fn read(mut input: Input)`, reading
  `input.read<Str>()`, with a `read(from:)` case for `Mod`.
- [ ] `crates/cpl-cli/src/laws.rs` reads each law argument's type through
  `cpl_facade::today::Ty` to pick its samples: the old checker does not resolve
  `impl<D: Domain> Sample for D`, so `samples_of<Color>()` fails there. It emits
  `for x in samples_of<T>():` for `x: T` (`samples_of<Int>()` when no type is
  written), with no type read, in the landing that moves `cpl test` onto cpl-next.
- [ ] `crates/cpl-rt/src/lib.rs` keeps `pub use cpl_prim::PrimOp as Prim` only for the
  frozen `cpl-ir` and `cpl-check`, which name the primitive operations `Prim`. Core's
  `Prim` is `cpl-next-ir`'s; the alias goes with those crates (coordinator 22:22).
- [ ] `tests/old-parser-accepts.txt` lists the rejected-form cases only
  the new parser reports (each citing its decision; coordinator 19:39).
  It is deleted with the old parser, and `next_old_parser_accepts`
  goes with it.

**Switching `cpl` onto cpl-next.** The shipped `cpl` runs only the old
pipeline. It moves by putting cpl-next behind `cpl-facade`'s queries,
not by rewriting a subcommand: every subcommand already reads
`cpl-facade` (contract 9), so the switch is one landing that changes
what answers those queries and deletes the old answerers. Until then,
each query below gets a cpl-next answer that only the harness reaches
(rule 5), with a case comparing it to the old one. A query is ready
when its blockers are gone and its cases agree. The switch lands when
every row is ready. A subcommand that needs no compiler (`setup`,
`new`, `fetch`, `judges`, `vendor`, `locate`, `migrate`) has no row.

| Order | Subcommands | Queries | Blocked on |
|---|---|---|---|
| 1 | `check`, `check --json`, `fix` | diagnostics, verdict, fixes | stage 3's exit (cpl-next today: 2,674 passing, 1,518 wrong, 165 masked); `fix` needs every fix the old checker offers carried on cpl-next's `Diag` |
| 2 | `run` with fixed input, `test`, `watch`, `bench` | run with `RunInput`, `Stop`, `Limits`; entry roots (`@test`, `@alt`, `@check`, interactor) | the engine runs `&str` to an `Outcome`, with no `Stop` (time limit, cancel), no stack limit, no streamed output and no interactive input; cpl-next has no entry roots, only library `Roots` |
| 3 | `build`, `judge`, `submit` | C++ with `Checks` and a profile; size bar | `cpl-next-cpp` emits one program with no `Checks` levels and no profile; the C++ size bar (END-STATE #15) on its output |
| 4 | `run --trace`, `explain`, `interact`, replay, `dap` | events, windows, replay of one call, frames | the engine's events (the engine gap: these ship in the switching landing); ring buffer and window (`last_steps`, `from_step`); frames for DAP; locals at a trap for `explain` |
| 5 | `profile`, `cover` | line counts | per-line counts from the engine |
| 6 | `stress` | run, trace, C++ | rows 2 to 4; its 7 lowering entry points (stage 6 removes them, but the switch deletes that lowering, so stress reads only the facade by then) |
| 7 | `lsp`, `doc`, `lint` | hover, definition, completion, signatures, the built-in catalog | lane I's `Scope` and `Signatures` views; `cpl-lsp` and `cpl-lint` still name `cpl_check` or `cpl_facade::today` (about 35 lines); lint's tree matching moves in stage 8, but anything it asks the checker must be a facade query first |
| - | `fmt` | none | nothing in stage 5: it reads only the syntax tree, which stage 8 replaces |

Rows 1 to 3 are the milestone's path (write, run, submit) and go
first. Row 4 decides the switch date, since the old VM cannot outlive
today's lowering. `facade_today_uses` counts the tool lines that still
reach a compiler type through the facade; it reaches 0 before the
switch, so no tool notices what sits behind the facade.

**Why this order.** Stage 3 was first written as finishing today's
checker in place. It is the new pipeline's own front half instead
(decided 2026-11-25): today's checker is keyed on library names
(`BUILTIN_MEMBERS`, `OPTIONAL_MEMBERS`, `std_family`), so a back half
fed from it would need a second translation layer and would inherit
the privilege the design removes. Work on it is work on code stage 5
deletes. Stage 4
comes before stage 5 because std uses `List` 2,877 times and every
other container is built on it; writing it needs storage without a
fill value, moving an element out, and literal and `for` support for
a declared type. The syntax tree is last because it is a parser
rewrite and nothing before it needs it; lints and the editor move
once, onto the new tree, not onto today's tree first.

**The engine gap (decided by Ryan, 2026-11-26).** Today's VM reads today's lowering (`cpl-interp` depends on `cpl-ir`), so stage 5's deletion would leave `cpl run`, trace, the debugger, the browser and stress no engine until stage 7; the reference interpreter is for tests only (decision 20). The bytecode engine over canonical IR, with events, therefore moves into stage 5 and ships in the switching landing. No interim engine.

**Not yet in the plan.** Pinned start commits and sizes for stages 3
and 4. Neither has an estimate anyone has measured.

## Cost per language feature

The design has more forms than the repository's earlier target
(`handbook/design/architecture.md` chose one IR and no separate
reference interpreter, for the cost of keeping them in step). The
cost is measured, not assumed: for each feature added to the slice,
count the layers and lines touched. If the median is above about five
files, forms are merged (typed tree with generic IR, or raw with
canonical) before widening further.

## What is not known

- How large stage 3 is. Its known part is the new pipeline's 4,130
  refused cases, 868 of them waiting on `use` (modules).
- Whether generated C++ stays under 64 KB on a 200-line program using
  six std modules. At slice scale it is 5.5 to 10.5 KB against 5.5 to
  12.4 KB from today's compiler.
- Speed against today's emitter and its 59 passes on the 58 `perf.cp`
  programs.
- Per-statement rechecking is designed and not built.
