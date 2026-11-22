# Cold skill eval, round 3 (CP Lang v0.3)

A model solved the 20 dry-run problems in `research/dryrun/`, learning the
language only from `skills/cp-lang/` (`SKILL.md`, then `io.md`,
`domains-and-cuts.md`, `idioms.md`, `ownership.md`,
`functions-and-generators.md`, `errors.md`, `debugging-and-stress.md`, and the
stdlib pages `README`, `seq`, `graph`, `grid`, `tree`, `dsu`, `prefix`,
`fenwick`, `segtree`, `lazy`, `algebra`, `mod`, `strings`, `geometry`).
Problem-side reads: each `pNN_*.tests/` sample, `pNN_*_gen.py`,
`p06_labyrinth_check.py` and `p19_guess_interact.py`. Each problem was
identified from its file name and sample; no statement was fetched.

Forbidden reads: none. No `.cp` file outside this directory, no `_ref.cpp`,
`LOG*.md`, `spec/`, `crates/`, `web/`, `docs/` or `plan/` file was opened
(directory listings that show such names were seen). `AGENTS.md` was shown by
the environment; its pointers into `spec/` and `plan/` were not followed.

Compiler: `cargo build --release -p cpl-cli` from this branch's base.

## Method

Per problem, the `research/dryrun/v03/loop.sh` stages, run by hand:
`cpl check`; `cpl test` on the samples; `cpl stress --brute pNN_*_ref.cpp
--gen pNN_*_gen.py --cases 200 --size 20` (plus `--checker
p06_labyrinth_check.py` for p06); `cpl lint`; `cpl build` and
`g++ -std=c++17 -O2`. The ref was only run as the stress brute.

p19 is interactive; see finding 3 for why its stages ran with
`repro/p19_interact_stdio.py` (the same judge in the protocol `cpl`
uses). The shipped interactor was also run directly against the built binary
on 200 generator cases: 0 failures.

Columns: stages are check / test / stress / lint (no errors) / build + g++ on
the final file. **First** = no edit at all after the first full run (lint-only
edits count as edits). **Stages-first** = every stage except lint passed on
the first run. **Workaround** = code written only because the skill, stdlib
or compiler did not do what the skill says. **Lint** = warnings kept in the
final file. Minutes are wall-clock per problem, approximate: problems were
written and run in batches of five to ten after about 15 minutes of reading
the skill (not included).

## Results

| # | problem | check | test | stress | lint | build+g++ | first | stages-first | workaround | lint kept | min | fixes after first run |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| p01 | ordinary numbers | ok | ok | ok | ok | ok | yes | yes | no | 0 | 2 | |
| p02 | movie festival | ok | ok | ok | ok | ok | yes | yes | no | 0 | 2 | |
| p03 | forest queries (2-D prefix) | ok | ok | ok | ok | ok | no | no | no | 0 | 3 | `Prefix([R, C] ...)` is E0302; 2-D needs `prefix(...)` (F2) |
| p04 | factory machines | ok | ok | ok | ok | ok | yes | yes | no | 1 | 2 | L0506 kept (F6) |
| p05 | subarray sums I | ok | ok | ok | ok | ok | yes | yes | no | 1 | 3 | L0506 kept (F6) |
| p06 | labyrinth | ok | ok | ok (checker) | ok | ok | no | yes | no | 0 | 4 | L0509: `find` -> `first` (F7) |
| p07 | shortest routes I | ok | ok | ok | ok | ok | yes | yes | no | 0 | 1 | |
| p08 | road construction | ok | ok | ok | ok | ok | yes | yes | no | 0 | 2 | |
| p09 | course schedule | ok | ok | ok | ok | ok | yes | yes | no | 0 | 2 | |
| p10 | tree matching | ok | ok | ok | ok | ok | yes | yes | no | 0 | 2 | |
| p11 | tree distances II | ok | ok | ok | ok | ok | yes | yes | no | 0 | 2 | |
| p12 | range updates and sums | ok | ok | ok | ok | ok | no | no | no | 0 | 5 | `Affine` not in prelude (E0201), then `affine` tag is `(Int, Int)` not `Affine` (E0302) (F1); L0506 -> `type Val` |
| p13 | dynamic range sum | ok | ok | ok | ok | ok | no | yes | no | 0 | 2 | L0506 -> `type Val` |
| p14 | elevator rides | ok | ok | ok | ok | ok | no | yes | no | 0 | 3 | L0506 -> `[n]1..=x` |
| p15 | book shop | ok | ok | ok | ok | ok | no | yes | no | 1 | 3 | L0101 -> `c.offset(-price[i])`; L0506 kept (F6) |
| p16 | increasing subsequence | ok | ok | ok | ok | ok | yes | yes | no | 0 | 1 | |
| p17 | string matching | ok | ok | ok | ok | ok | yes | yes | no | 0 | 1 | |
| p18 | binomial coefficients | ok | ok | ok | ok | ok | no | yes | no | 0 | 2 | L0506 -> `const MAX_A` |
| p19 | guessing game (interactive) | ok | ok | ok (interactor) | ok | ok | yes | no | no | 0 | 4 | none to the solution; harness mismatch (F3) |
| p20 | point location test | ok | ok | ok | ok | ok | no | no | no | 0 | 3 | `turn` not in prelude (E0201); third point widened to `Int` because the generator exceeds the statement's bounds (F9) |

## Totals

| metric | value |
|---|---|
| pass (all stages, final) | 20 / 20 |
| first-attempt pass (no edit after first run) | 12 / 20 |
| stages-first (no correctness/compile fix needed) | 16 / 20 |
| workarounds | 0 |
| lint warnings kept | 3 (all L0506) |
| contestant source | 184 lines in all |

Every first-run failure was a compile error (3) or a harness issue (2); no
solution was ever wrong on a sample or in stress. Compile errors were each
fixed in one edit from the error's `help:` line or wording.

## Skill findings, ranked by impact

**F1. The stock `affine` action's tag type is not stated, and the page puts
`struct Affine` next to it (p12; repro `repro/affine_tag.cp`).**
`stdlib/algebra.md`: "`const affine: AffineAction = AffineAction()` — Stock
action: `x -> a * x + b` on `+` folds (span `n`: `a * s + b * n`)", and
further down "`struct Affine<T>(a: T, b: T)`". Writing
`st.apply(r, Affine(1, x))` after `SegTree(+, a, act: affine)` first needs a
`use` and then fails with E0302 "`st.apply` takes an (Int, Int) for `f`".
The tag is a plain `(a, b)` tuple. `segtree.md` never shows `act:` with a
stock action, so the tag type of `add`, `assign` and `affine` must be guessed.
Either the docs should say "tag `(a, b)`" (and one example), or `affine`
should accept `Affine`, which is the type the same page offers for the job.

**F2. `SKILL.md` teaches `Prefix(a)` as the prefix-sum spelling; it is 1-D
only (p03).** `SKILL.md`: "| `pre[i+1] = pre[i] + a[i]` | `let p = Prefix(a)`;
`p.fold(l..=r)` |" and the opening example `let p = Prefix(a)`. On a
`[R, C]` grid this gives E0302 "`Prefix` takes a [R]Int". Only
`stdlib/prefix.md` says "Call `prefix(a)` on a `[D]T`, `[R, C]T`, or
`List<T>`", and its listed `fold` is the 1-D `fold(self, r: Range<D>)`; the
2-D `fold(rows, cols)` that worked is not documented anywhere read. Fix:
show `prefix(a)` in `SKILL.md` (it covers both shapes) and document the 2-D
`fold`.

**F3. `--interactor` protocol is underspecified, and the dry-run interactor
uses a different one (p19).** `debugging-and-stress.md`: "`--interactor
PROG` for interactive problems (the interactor talks to the solution over
stdin/stdout and its exit code is the verdict)". `cpl test --help` adds
"judge by talking to `PROG <input>`". `p19_guess_interact.py` instead
expects `PROG <input> <solution command...>` and spawns the solution itself,
so `cpl test` and `cpl stress` with it report WA with a Python traceback and
"got: (nothing)" for a correct solution. This cost the most time of any
issue, and the traceback was cut off, so the cause was found by running the
interactor by hand. Fix: state the argv contract in the skill and make the
dry-run interactor follow it (or have `cpl` support both); show the
interactor's stderr in full on failure.

**F4. Core control-flow syntax is not documented in the pages read.**
None of `SKILL.md`, `io.md`, `idioms.md` or `functions-and-generators.md`
shows `elif`/`else if`, `not`/`!`, `max=` (only `min=` appears; `max=` is seen once, in a `segtree.md` doc line), whether
`if` is an expression, or list comprehensions outside `seq.md`. Python-style
guesses (`elif`, `not`, `max=`, `[f(x) for x in xs]`) all compiled, so no
time was lost, but a model without Python priors would have nothing to go on.

**F5. "Which names need `use`" is easy to miss (p12, p20).** Each stdlib page
has an "In prelude" line, but `SKILL.md` never says to check it, and its
algorithm list ("`Prefix`, `SegTree`, `Dsu`, `Graph`, ...") reads as if the
stdlib is ambient. `turn` (geometry) and `Affine` (algebra) were not in the
prelude; `smallest_topo` was caught by reading. The E0201 `help:` line
("add `use std.geometry.{turn}`") made each fix one edit.

**F6. L0506 flags equal literals that are different facts.** "`1e9` is
written on 2 lines ... name it once": p04 (`t` and each `k_i`), p05 (`x` and
each `a_i`) and p15 (`n`, each price, each page count) have independent
statement bounds that happen to be equal; one shared name would be wrong if
one bound changed. Kept as 3 warnings. Where the literal was one fact (p12,
p13, p14, p18) the hint improved the code. The skill does not say when to
keep a lint warning.

**F7. Search-name vocabulary is not in `SKILL.md` (p06).** `list.find(pred)`
compiled and worked, but lint L0509 says "`find` is the index of a value" and
asks for `.first(..)`. The quick tables in `SKILL.md` never mention
`first`/`find`/`position`; the lint caught it.

**F8. No documented way to share setup across `@!multitest` tests (p18).**
`io.md`: "The compiler wraps the entire top-level body in a `t`-iteration
loop". A `Comb` table (or sieve) built once for all tests has no spelling, so
p18 reads its own count and loops instead of using `@!multitest`. Not a
workaround in this problem (the plain loop is natural), but the skill should
say how to hoist shared work or that one should not use `@!multitest` then.

**F9. (Not a skill issue.) `p20_point_gen.py` violates its statement's
bounds.** Collinear third points `p1 + k * (p2 - p1)` with `|k| <= 3` reach
about 7e9, outside the statement's `-1e9..=1e9`; a solution typed as the
statement says traps with R0502 in stress (the trap itself is correct
behaviour). The third point is read as `Int` to match the generator.

What the skill got right, for calibration: typed reads with tuple/record
shapes, domains, `bfs`/`dijkstra`/`Graph.directed`/`Rooted`/`Dsu`,
`first_true`/`last_true` on ranges, `lis`, `find_all`, `Comb`, `Fenwick`,
and the `none:` print option were all used as documented on the first try;
17 of 20 files compiled on the first `cpl check`.

## Repros

- `repro/affine_tag.cp`: F1, E0302 on an `Affine` tag for `act: affine`.
- `repro/p19_interact_stdio.py`: the p19 judge in the `PROG <input>`
  protocol, used for p19's `cpl test` and `cpl stress` runs (F3).
