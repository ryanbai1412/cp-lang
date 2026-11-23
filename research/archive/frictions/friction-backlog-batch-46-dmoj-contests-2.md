# Friction Backlog — Scout Batch 46: DMOJ Contest Classics, Part 2 (`aac`, `yac`, `sac`)

Scout Batch 46 ported ten more problems from the `aac*` / `yac*` / `wac*` / `sac*` / `dmpg*` bucket against the locked v2 spec (`docs/one-page-cheat-sheet.md`, `3723dca`). Ports, runnable versions, and the harness are in `proto/ports/scout_batch_46/`; all ten runnable ports match the original AC C++ on 150 random small/medium inputs each under ASan + UBSan.

One caveat on the sample before the numbers. This bucket is nearly used up: no unported `wac*` or `dmpg*` problem remains, and the one clearly hard problem left (`sac22cc4p5`, interactive merge-insertion sort) uses a time-seeded RNG and cannot be diffed. The ten ported here are the hardest of the rest, but several are short. The LOC reductions below are therefore flattered by fixed per-file C++ boilerplate, and the friction list is short because the problems lean on few features. Read this batch as evidence about small everyday programs, not about hard ones.

## 1. Dual LOC

Non-blank, non-comment lines. `raw` is the whole C++ file. `stripped` removes the shared debug/macro header and reusable library code (the `SegTree` class in `sac22cc2p5`, stream-printing templates in `yac6p5`, the author's `stress()` driver in `yac1p3`) and keeps `main()` and all problem-specific code.

- `aac4p4`: 148 raw / 123 stripped -> 32 v2 (-78.4% / -74.0%), 64 today.
- `aac5p4`: 61 raw / 36 stripped -> 10 v2 (-83.6% / -72.2%), 14 today.
- `yac1p2`: 134 raw / 109 stripped -> 19 v2 (-85.8% / -82.6%), 43 today.
- `yac1p3`: 130 raw / 46 stripped -> 9 v2 (-93.1% / -80.4%), 16 today.
- `yac6p4`: 57 raw / 32 stripped -> 12 v2 (-78.9% / -62.5%), 18 today.
- `yac6p5`: 176 raw / 137 stripped -> 13 v2 (-92.6% / -90.5%), 38 today.
- `yac7p2`: 136 raw / 115 stripped -> 50 v2 (-63.2% / -56.5%), 84 today.
- `sac22cc2p3`: 62 raw / 37 stripped -> 11 v2 (-82.3% / -70.3%), 36 today.
- `sac22cc2p5`: 210 raw / 49 stripped -> 10 v2 (-95.2% / -79.6%), 32 today.
- `sac22cc3p3`: 63 raw / 38 stripped -> 8 v2 (-87.3% / -78.9%), 18 today.

Totals: 1,177 raw / 722 stripped -> **174 v2** (-85.2% vs raw, -75.9% vs stripped) and **363 today** (-69.2% vs raw, -49.7% vs stripped).

Two of the big reductions come from rethinking the algorithm, not from syntax. `yac6p5` drops a 50-line two-multiset running median for a sort and two prefix sums; `sac22cc3p3` replaces zero-padded string suffixes with `a % p`. The smallest reduction, `yac7p2`, is the one where v2 has nothing to offer: the program is a list of walk instructions and stays one.

## 2. Friction in the locked v2 spec

Ordered by how much I hesitated while writing. The first three are real; the rest are small spec gaps.

### F46.1 — Interactive output outside `ask()` has no stated flush rule (`yac1p3`)

The cheat sheet covers interaction only through `@interactor` paired with `ask("?", x, limit: 2)`. `yac1p3` has no query prefix: the program prints a move, then reads the opponent's. The natural v2 code is

```cp
for (x, y) in read_until((0, 0)):
    print(n + 1 - x, n + 1 - y)
```

and nothing says whether that `print` reaches the judge before the next `read`. If it does not, this deadlocks on the judge with no local symptom. The prototype shows the risk is real: `cp.py build` buffers all output and flushes once at exit, so the runnable port cannot be played live at all (the harness replays a recorded game into it instead). Batch 39's `wac6p6` was also only transcript-checked.

Suggested rule: every `read` flushes pending output first. It costs nothing on batch problems (reads finish before the first print in almost all of them) and removes the question.

### F46.2 — A transposed grid cannot share a binding with the original (`yac7p2`)

Collision: per-test domains × product-domain arrays × `g.transposed`. The construction needs an even side as the height, so the port builds an `h × w` grid and, when it swapped the sides, prints the transpose. `g` is `[Row, Col]Int` and `g.transposed` is `[Col, Row]Int`, which are different types, so there is no way to write "the grid to print":

```cp
if flip:
    let out = g.transposed
    for c in Col: print(out.row(c))
else:
    for r in Row: print(g.row(r))
```

The duplication is two lines here, but any solution that normalises orientation first and un-normalises at the end hits it. A `print(g)` that prints a 2D array row by row would hide it for output; the spec does not say what `print` does with a 2D array.

### F46.3 — Small counters used as indices need a cast at every use (`yac1p2`)

The DP table is indexed by "blacks so far" and "whites so far", both in `0..3`. Declared as a domain, every index sum needs wrapping, and the guard that makes the wrap safe is written separately:

```cp
for a in K for b in K if a + ub < 3 and b + uw < 3:
    down[u, K(a + ub), K(b + uw)] += down[v, a, b]
```

Declared as a plain `[V, 0..3, 0..3]Int` the casts go away, and that is what I would write in a contest. So the friction is a choice with a wrong-looking default: the spec's advice to name a domain for index safety makes this code worse, and nothing tells the writer that small arithmetic counters are the case for anonymous ranges. One sentence in section 3 would settle it.

### F46.4 — `Dir4` variants are never named (`yac7p2`)

The cheat sheet gives `Dir4`, `.cw`, `.ccw`, `.back`, `p.step(d)` and `Dir4.from("UDLR", ch)`, but not the four variant names. The port guesses `U`, `D`, `L`, `R`; `N`/`E`/`S`/`W` (used for the example enum in section 7) is as plausible. A program that only walks in fixed directions cannot be written from the cheat sheet alone.

### F46.5 — No last-match search (`aac4p4`)

`a.find(x)` returns the first index. The original wants the last column holding `f`:

```cp
let hit = first(c for c in Col.rev if arr[c] == f)
```

This works but took a moment to find, and it relies on a domain having `.rev` (stated for ranges, not for domains). Either choice was accepted by the original's construction here, so the cost was only hesitation.

### F46.6 — Smaller gaps

- `yac6p4`: no stated literal for a `const` array over a written range (`const GRUNDY: [0..=100]Int = [...]`), and `sum(it)` has no xor counterpart, so a nim-sum is a three-line loop.
- `aac4p4`: no stated way to print `Bool` as `0`/`1`; the port writes `"{Int(on[i, j])}"` and assumes `Int(Bool)` exists.
- `sac22cc2p5`: `Fenwick` has `.add` but no `.set`. A point assignment either reads the old value from a shadow array or switches to `SegTree`, which is what the port does.
- `sac22cc3p3`: the answer depends on `sorted(a, by:)` being stable; the spec does not say.
- `sac22cc2p3`: `@memo fn best(u)` recurses `2^19` deep on a chain-shaped tree. The spec promises a flat table but nothing about stack depth.

## 3. Prototype gaps seen in `today/`

These are `proto/cpc` limits, logged for whoever graduates features, not spec problems. `today/` is 363 lines against 174 for v2; the difference is almost all here.

- Output is flushed only at exit (F46.1), so interactive programs cannot run live.
- No `Rooted` in the `build` path: `yac1p2` hand-writes a BFS order and parent array.
- No generators or comprehensions as arguments: every `print` of a row builds a `List<Int>` in a loop, and `aac4p4`'s three "fill these cells until `l` are on" loops are written out instead of sharing one `fill`.
- No inline `if` expression and no tuple `let`, so `let (ub, uw) = if ...` becomes five lines.
- No `Fenwick`/`SegTree` in the `build` path used here: `sac22cc2p5` hand-rolls two Fenwick trees.
- The work estimator warns "this loop nest can run ~1.0e+11 times" on `yac7p2` because it multiplies the per-test bounds by the test count; the real bound is on the sum over tests, which a declared range cannot express.
