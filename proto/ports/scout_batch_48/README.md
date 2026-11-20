# Scout Batch 48 — Hard BTOI, CEOI, EGOI & DP, Part 2

Ten more problems from the `dp*` / `btoi*` / `ceoi*` / `egoi*` / `apio*` / `joisc*` / `joi*` prefix bucket, solved against the locked v2 spec (`docs/one-page-cheat-sheet.md`). None of them appears in an earlier port, scout batch or blind-critic batch.

Each problem has two files. `<code>.cp` is the solution written in spec v2 as the cheat sheet defines it. `today/<code>.cp` is the same algorithm restricted to what `proto/cpc` compiles right now, and is the one that is built and run.

- `btoi06p5` — RLE Compression: DP over the current escape symbol with add-all / assign-all / set-one / argmin.
- `ceoi19pp1` — count equal gaps between two arrays with a hash map.
- `ceoi19pp2` — online ordered set that pops everything above the newest value.
- `egoi22p1` — SubsetMex: backward demand propagation.
- `egoi22p5` — Data Centers: subtract from a sorted prefix, then merge.
- `egoi22p8` — rotation-proof cards: rank and unrank canonical cards on a 3x3 dot grid (grader functions `BuildPattern` / `GetCardNumber`).
- `egoi23p1` — Inflation: lazy global offset over a price multiset.
- `egoi23p5` — Carnival General: incremental path insertion.
- `egoi23p6` — Candy: 3D swap-budget knapsack.
- `egoi23p8` — Guessing Game: two-phase communication using compositions of 11 as a codebook.

## Verifying

```
python3 proto/ports/scout_batch_48/check.py            # all ten
python3 proto/ports/scout_batch_48/check.py btoi06p5   # one problem
```

The script copies each `today/*.cp` to a temporary directory, builds it with `proto/cp.py build` and `g++ -std=c++17 -O2`, compiles the original submission from `corpus/submissions/`, and compares the outputs token for token on seeded random inputs (three small cases for every medium one):

- The eight stdin problems run 400 inputs each.
- `egoi22p8` wraps the original's `BuildPattern` / `GetCardNumber` in a small `main`, compares the patterns both build for 600 card numbers, and then compares both decoders on transformed copies of those patterns.
- `egoi23p8` runs phase 1 on 300 reveal orders, then erases two labels from the original's output four times and compares phase 2. The original scores 91 of 150 points: on some erasures it fails an assertion or indexes past its candidate list. Those inputs are skipped.

Generator limits that come from the original, not from the port: `egoi22p5` keeps counts inside 32 bits, and `egoi23p6` stops at `n = 30` because the original puts its DP table on the stack.

The dual LOC numbers and the friction found are in `research/friction-backlog-batch-48-olympiads-2.md`.
