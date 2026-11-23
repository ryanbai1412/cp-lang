# Scout Batch 31: grids, BFS, flood fill, simulation and messy implementation

Ten corpus problems that had no port anywhere under `proto/ports/`, solved
against the locked v2 spec (`docs/one-page-cheat-sheet.md`). The corpus is
close to exhausted for grid problems, so the batch takes every unported one
that fits and fills the rest with state-space BFS and simulation.

Each problem has two files:

- `<problem>.cp` is the idiomatic v2 solution: `p.adj4`, `Dir4`, `[Row, Col]T`,
  `Str`, parallel tuple assignment and the stdlib searches. These are design
  sketches; places where the spec had no obvious answer carry a `// FRICTION`
  marker that matches an item in
  `research/archive/frictions/friction-backlog-batch-31-grids-sim.md`.
- `today/<problem>.cp` is the same algorithm in the subset the prototype runs
  now, checked against the original C++.

## Problems and dual LOC

Non-blank, non-comment lines. "C++ full" is the whole accepted submission
with its template; "C++ stripped" starts at the first line that belongs to
this problem (no debug macros, modint or I/O template).

```
problem          what it exercises                           C++ full  C++ stripped  v2  today
ucrpc21f         grid Dijkstra, cost on the entered cell           79            45   8     26
ucrpc21d         flood fill on a (2n+1) x (2m+1) ASCII maze       100            69  15     36
dmopc19c4p2      BFS over (token cell, hole cell)                  84            55  11     41
sac22cc1p3       falling blocks and settling water                105            74  31     45
coci25c3p1       brush walk driven by a Str of commands            29            27  14     25
coci20c6p1       2 x 2 stamp with edge clamping                    58            29  11     14
imo22p6checker   path-count DP in value order over adj4           265            46  11     28
dmopc21c6p4      BFS over (city, 2-bit visited set)               120            89  18     51
ucrpc21g         reachable (cell, coins) states per tick           95            64  25     37
dmopc21c7p1      fill blanks so rows and columns increase         103            73  17     31
total                                                            1038           571 161    334
```

The v2 forms are 84% shorter than the full submissions and 72% shorter than
the stripped ones; the forms that run today are 68% and 42% shorter.

Two more ports are kept outside the ten and outside the totals. Scout batch
27 landed the same two problems while this batch was in progress, so they
are not new coverage, but they are checked like the rest and three backlog
items (31.4 to 31.6) come from them:

```
cco19p2          components, gap graph, Dijkstra, redraw          100            90  25     54
noi10p3          Dijkstra over (junction, entry direction)        151           125  34     47
```

## Running the differential tests

```
python3 proto/ports/scout_batch_31/check.py              # every port, 150 cases each
python3 proto/ports/scout_batch_31/check.py cco19p2 500  # one port, 500 cases
```

`check.py` compiles each original from `corpus/submissions/`, builds each
`today/` port with `proto/cp.py build` on a temporary copy, and compares
output on seeded random inputs: two small cases, then one medium case,
repeating. `coci25c3p1` walks a `Str`, which `proto/cpc` cannot iterate yet,
so it runs on the interpreter through `proto/cp.py lib-run`.

All twelve ports match their originals on 150 cases each.

## Notes on the originals

- `ucrpc21d` (3944584) prints four region sizes unconditionally, so it reads
  past the end of its list on a maze with fewer than four regions. The
  generator only emits mazes with at least four.
- `coci20c6p1` (4188008) writes the whole 2 x 2 stamp without a bounds check,
  so a `#` in the last row or column is undefined behaviour. The generator
  keeps the last row and column clear; the ports clamp to the picture.
- `ucrpc21g` (3944717) indexes a fixed table of 105 ticks by event time, so
  the generator keeps event times in 0..=104.
- `noi10p3` (5672180) packs a state as `((x + 1) * (n + 1) + (y + 1)) * 8 + dir`
  while `y + 1` reaches `n + 1`, so `(x, -1)` and `(x - 1, n)` share a slot.
  The colliding states never carry the same direction as an exit state, so the
  answer is unaffected; the ports index a true 3D array instead.
