# Scout batch 49: hard IOI classics, part 2

Ten accepted C++ submissions from the `ioi*` prefix only, re-solved against
the locked v2 sheet (`docs/one-page-cheat-sheet.md`).

| problem | original | what it is |
| --- | --- | --- |
| `ioi00p2` | `4682555.cpp` | Car Parking: cycle-chasing rounds under a driver limit (constructive) |
| `ioi01p3` | `4806167.cpp` | Twofive: rank and unrank 5x5 grids by counting staircase fillings |
| `ioi99p6` | `4899527.cpp` | A Strip of Land: widest flat rectangle, column window times row two-pointer |
| `ioi14p4` | `4684567.cpp` | Gondola: validate, reconstruct and count replacement sequences mod 1e9+9 |
| `ioi15p5` | `5723358.cpp` | Sorting: binary search on rounds, swaps chosen on values to commute with the adversary |
| `ioi17p2` | `5724153.cpp` | Wiring: prefix DP with bracket-matched blocks keyed by colour balance |
| `ioi21p1` | `4698803.cpp` | Distributing Candies: sweep over boxes, segment tree of unclamped walks with a descent |
| `ioi22p4` | `4783516.cpp` | Digital Circuit: per-source weights from prefix/suffix products, range-flip lazy tree |
| `ioi12p2` | `4689474.cpp` | Parachute Rings: incremental union-find with four "graph without this ring" copies |
| `ioi18p2` | `4731161.cpp` | Seats: corners and dents per prefix, range add with count-of-minimum |

## Layout

- `<problem>.cp` is the v2-spec port: what the solution should read like.
  These files are **not compiled**; the prototype does not implement all of v2.
- `today/<problem>.cp` is the same algorithm in the subset `proto/cpc` builds
  now. These are the files that are verified.
- `check.py` builds every `today/` file (on a copy), compiles the emitted
  C++ with ASan/UBSan and diffs it against the original submission.
- `loc.py` prints the dual LOC table.

```
python3 proto/ports/scout_batch_49/check.py            # all ten, 300 cases each
python3 proto/ports/scout_batch_49/check.py ioi18p2 --cases 40
python3 proto/ports/scout_batch_49/loc.py
```

## What the check covers

Last run: 3000 cases pass (300 per problem: 200 small, 100 medium), port
side under ASan and UBSan.

Seven of the ten originals are grader functions with no `main`
(`ioi14p4`, `ioi15p5`, `ioi17p2`, `ioi21p1`, `ioi22p4`, `ioi12p2`,
`ioi18p2`). `check.py` appends a small stdin/stdout `main` to each; the
`today/` file reads the same stdin format and prints the same thing.

On top of the diff against the original:

- `ioi17p2`, `ioi21p1`, `ioi12p2` and `ioi18p2` are compared with an
  independent brute force (all inputs for `ioi21p1` and `ioi18p2`, the small
  ones for the other two).
- `ioi99p6` checks that the printed rectangle is flat enough and, on small
  grids, that its area is the exhaustive maximum.
- `ioi00p2`, `ioi15p5` and `ioi14p4` (task 2) replay the port's own output:
  the cars end sorted within the driver limit, the row ends sorted in the
  fewest possible rounds, the replacement sequence reproduces the input.

Four ports deliberately differ from the original's method:

- `ioi17p2` is the O(n) balance DP. The original keeps two deques, a
  `last_p` table and an auxiliary `dp2` chain for the same recurrence.
- `ioi21p1` keeps one tree of (end, highest, lowest) walks. The original
  keeps two max-subarray trees, one on the values and one on their negation.
- `ioi12p2` tracks, per candidate, the degrees and components of the graph
  without it. The original keeps global degree histograms and a per-ring
  count of high-degree neighbours, and walks the cycle to measure it.
- `ioi22p4` uses the guarantee that a gate's parent has a smaller number, so
  both tree passes are index loops. The original recurses.

## Not tested

- The v2 files at the top level are uncompiled sketches.
- No max-size inputs: nothing here measures time or memory at the limits.
- The grader problems are compared through the stdin/stdout wrapper, not an
  official grader or checker.
- `ioi01p3` and `ioi22p4` are verified against the original only (no
  independent solver).
- Input formats and ranges were inferred from the accepted code and the
  published IOI tasks, not from statements in the corpus. The generators
  only produce inputs the tasks allow: `w >= 2` for `ioi00p2`, valid
  sequences for `ioi14p4` task 2, `m = 3n` swaps for `ioi15p5`, distinct
  points for `ioi17p2`, parents numbered below children for `ioi22p4`, and
  no repeated or self links for `ioi12p2`.
