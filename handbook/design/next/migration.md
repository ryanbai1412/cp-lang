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
8. **Each window has a fixed end**, set when the stage starts. If the
   new code is not passing by then, the new work is reverted; the
   window is not extended.
9. **Old code waiting for deletion is frozen**: no fixes, no
   features. A fix goes to a conformance case, which binds the new
   code.
10. **Prefer no window.** A stage that can change today's code in
    place does, so there is never a second copy.

## Order

| Stage | Work | Second copy? | Exit |
|---|---|---|---|
| 1 | Measures and the slice into the repo and the gate | no | the three counts and the spec-verdict harness run in the gate at HEAD |
| 2 | Run and check half of the facade, over today's compiler | no | direct compiler dependencies in tool crates for run and check: 0 |
| 3 | Today's checker made total, in place | no | corpus programs with every expression typed: 1,564 of 1,564 (today 470); errors found only in lowering: 0 (today 45); inference lines in lowering: 0 (today about 7,073) |
| 4 | Storage operations in the language, then `List` in CP Lang | no | `List` builtins in the compiler: 0; the 1.1x time and memory bars hold on the container benchmarks |
| 5 | New back half: generic IR, link, reference interpreter, C++ emitter | **yes, the one window** | all three counts at 100%; today's lowering, tree-walker and C++ emitter deleted in the switching landing |
| 6 | Remaining containers, stress, rest of the facade | no | compiler-privileged std behaviour: 0; `cpl-stress` lowering entry points used: 0 (today 7) |
| 7 | Bytecode engine, events, debugger, machine code | per engine, switch and delete together | today's VM deleted; every engine equal to the reference interpreter on all three counts |
| 8 | Lossless syntax tree, then lints and editor onto it | parser switch and delete together | lint matches on today's tree: 0 (today about 940); today's parser deleted |

**The stage 5 window.** The new back half cannot replace today's in
one commit; it grows until it accepts every program. While it grows:

- it is reachable from the test harness only (rule 5);
- its accepted counts are published and may only rise; the gate
  fails on a drop;
- its end date is set from the measured rate of its first week;
- the tree-walker's removal is part of its exit, not a later cleanup.

**Why this order.** Stage 3 comes before the back half because a
typed tree with 22,236 untyped expressions cannot feed it. Stage 4
comes before stage 5 because std uses `List` 2,877 times and every
other container is built on it; writing it needs storage without a
fill value, moving an element out, and literal and `for` support for
a declared type. The syntax tree is last because it is a parser
rewrite and nothing before it needs it; lints and the editor move
once, onto the new tree, not onto today's tree first.

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

- How large stage 3 is. The known part is 22,236 untyped expressions
  and about 7,073 lines of inference in lowering.
- Whether generated C++ stays under 64 KB on a 200-line program using
  six std modules. At slice scale it is 5.5 to 10.5 KB against 5.5 to
  12.4 KB from today's compiler.
- Speed against today's emitter and its 59 passes on the 58 `perf.cp`
  programs.
- Per-statement rechecking is designed and not built.
