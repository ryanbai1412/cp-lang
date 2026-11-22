# Scout batch 45: hard long-tail corpus classics

Ten accepted C++ submissions from the prefixes no other scout has claimed
(`shoi`, `ucrpc`, `iati`, `neerc`, `rpeng`, `noimps`), re-solved against the
locked v2 sheet (`docs/one-page-cheat-sheet.md`).

| problem | original | what it is |
| --- | --- | --- |
| `shoi15p2` | `5450923.cpp` | range-assign lazy tree with longest-run nodes and a descent |
| `shoi16p1` | `5445691.cpp` | tree DP over "reach" `-k..=k` (guards covering marked nodes) |
| `shoi16p2` | `5445692.cpp` | counting lattice squares by inclusion-exclusion over deleted points |
| `shoi16p5` | `5445695.cpp` | segment tree of affine maps mod 1e9+7 under point updates |
| `ucrpc21h` | `3950286.cpp` | Dijkstra, then a 4-way partition feasibility DP |
| `iati21a12` | `5071843.cpp` | small-to-large heaps with a lazy offset up a weighted tree |
| `iati21a23` | `5072008.cpp` | constructive shot sequence, checked by simulation |
| `neerc10k` | `5325567.cpp` | Brooks-style colouring with the least odd `k >= max degree` |
| `rpeng2` | `4810287.cpp` | IOI 98 Polygon, circular interval DP on (least, most) |
| `noimps22p5` | `5448138.cpp` | level-by-level greedy on a bracket forest with a sorted bag |

## Layout

- `<problem>.cp` is the v2-spec port: what the solution should read like.
  These files are **not compiled**; the prototype does not implement all of v2.
- `today/<problem>.cp` is the same algorithm in the subset `proto/cpc` builds
  now. These are the files that are verified.
- `check.py` builds every `today/` file (on a copy), compiles the emitted
  C++ with ASan/UBSan and diffs it against the original submission.
- `loc.py` prints the dual LOC table.

```
python3 proto/ports/scout_batch_45/check.py            # all ten, 150 cases each
python3 proto/ports/scout_batch_45/check.py rpeng2 --cases 40
python3 proto/ports/scout_batch_45/loc.py
```

## What the check covers

Last run: 1498 cases pass (150 per problem; `iati21a23` loses 2 inputs with
`s = c = 0`, which the original itself rejects).

Half of the cases are small, half medium. `shoi16p1`, `shoi16p2`, `shoi16p5`
and `ucrpc21h` are also compared against an exhaustive solver on the small
half, so the port and the original are not just agreeing with each other.
`neerc10k` and `iati21a23` have many valid answers: both the port's and the
original's output go through a validator instead of a token diff.

Two ports deliberately differ from the original's method:

- `neerc10k` colours in reverse BFS order from a vertex of degree `< k`. The
  original uses an ad-hoc stack order that asserts when it runs out of colours.
- `rpeng2` runs one circular DP in O(n^3). The original re-runs a linear
  interval DP for every cut edge, O(n^4).

## Not tested

- The v2 files at the top level are uncompiled sketches.
- No max-size inputs: nothing here measures time or memory at the limits.
- `iati21a12`, `noimps22p5`, `shoi15p2` and `rpeng2` are verified against the
  original only (no independent solver); the input formats and ranges for
  all ten were inferred from the accepted code, not from the statements.
- `noimps22p5` with `x = y = 0` prints nothing in the original; the generator
  never produces it.
